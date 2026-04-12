# Development Plan: Notebook Reports

**Target command:** `report render analysis.rlnb`  
**Current phase:** design  
**Status:** planning  

---

## Overview

Add a notebook-style report system that lets users write `.rlnb` files —
Markdown documents with fenced RustLab code blocks. A `report` command
executes the blocks in sequence, captures plot outputs as inline Plotly
divs, and renders the whole thing into a self-contained HTML report.

**Use case:** generating shareable DSP analysis reports with interleaved
narrative, code, computed results, and interactive plots.

### Example `.rlnb` file

````markdown
# Filter Analysis Report

## Lowpass FIR Design

We design a 64-tap lowpass filter with cutoff at 0.3π:

```rustlab
h = fir1(64, 0.3);
[H, w] = freqz(h, 1, 512);
plotdb(w/pi, H)
title("Lowpass Filter Response")
xlabel("Normalized Frequency (×π rad/sample)")
```

The passband ripple is well within spec. Now test with noisy input:

```rustlab
x = sin(2*pi*0.1*(0:255)) + 0.5*randn(1, 256);
y = filter(h, 1, x);
subplot(2,1,1); plot(x); title("Input")
subplot(2,1,2); plot(y); title("Filtered Output")
```
````

### Generated output

```
report/
  index.html          ← single self-contained HTML report
```

---

## Phase 1 — Notebook Parser (`rustlab-plot`)

**Goal:** Parse `.rlnb` files into a sequence of blocks.

Add a new module `crates/rustlab-plot/src/report.rs` with:

```rust
pub enum Block {
    Markdown(String),       // raw markdown text
    Code(String),           // rustlab source code
}

pub fn parse_notebook(src: &str) -> Vec<Block>
```

**Parsing rules:**
- Split on ` ```rustlab ` / ` ``` ` fences (standard CommonMark fenced code blocks)
- Everything outside fences → `Block::Markdown`
- Everything inside ` ```rustlab ``` ` → `Block::Code`
- Ignore other fenced blocks (```python, etc.) — treat as markdown

This is simple string splitting, no markdown AST needed.

**Files to modify:**
- New: `crates/rustlab-plot/src/report.rs`
- Edit: `crates/rustlab-plot/src/lib.rs` (add `pub mod report;`)

---

## Phase 2 — HTML Report Renderer (`rustlab-plot`)

**Goal:** Render a sequence of executed blocks into a single HTML file.

Add to `report.rs`:

```rust
pub struct RenderedBlock {
    pub kind: RenderedKind,
}

pub enum RenderedKind {
    Markdown(String),                   // raw markdown
    Code {
        source: String,                 // the rustlab code
        output: String,                 // captured text output (printed values, etc.)
        plot_html: Option<String>,      // Plotly div+script if a plot was generated
    },
}

pub fn render_report_html(title: &str, blocks: &[RenderedBlock]) -> String
```

**HTML template design:**
- Single self-contained HTML file
- Same dark theme as existing Plotly export (#1e1e2e / catppuccin)
- Plotly.js 2.35.0 from CDN (loaded once in `<head>`)
- Lightweight markdown→HTML: convert `#` headings, `**bold**`, `*italic*`,
  `\n\n` → `<p>`, `` `code` `` → `<code>`. No external dependency — a
  ~50-line function handles the subset we need.
- Code blocks rendered in `<pre><code>` with monospace font
- Text output rendered in a muted `<pre>` block below code
- Plot divs get unique IDs (`plot-1`, `plot-2`, ...) and are initialized
  via `Plotly.newPlot()` calls at the bottom of the page
- Responsive layout, max-width ~900px centered, readable typography

**Key reuse:** Extract the trace/layout generation from `html.rs` into a
helper `fn render_figure_plotly_div(fig: &FigureState, div_id: &str) -> String`
that returns just the `<div>` + `<script>` fragment (no full HTML wrapper).
The existing `render_figure_state_html` can be refactored to call this
helper wrapped in the full-page template.

**Files to modify:**
- Edit: `crates/rustlab-plot/src/html.rs` (extract div-rendering helper)
- Edit: `crates/rustlab-plot/src/report.rs` (add renderer)

---

## Phase 3 — Report Execution Engine (`rustlab-cli`)

**Goal:** Wire up the `report` REPL command that parses, executes, and renders.

### Execution strategy

The evaluator (`Evaluator`) already persists variable state across
`exec_stmt` calls — this is exactly what `run <file>` does. For notebooks:

1. Parse `.rlnb` → `Vec<Block>`
2. For each `Block::Code`:
   a. Tokenize + parse the source
   b. Reset figure state (`clf` equivalent)
   c. Capture stdout (text output from `disp`, `ans =`, etc.)
   d. Execute statements via `ev.exec_stmt()`
   e. Snapshot the current `FIGURE` state if it has series data
   f. Build a `RenderedBlock` with source, captured output, and optional plot div
3. For each `Block::Markdown`: pass through as-is
4. Call `render_report_html()` with all rendered blocks
5. Write to output path

### Stdout capture

Wrap code block execution with a capture mechanism. Two options:
- **Simple:** Redirect prints to a `Vec<String>` buffer via a thread-local
  `CAPTURE_BUFFER` in the evaluator (similar pattern to `HTML_FIGURE_PATH`)
- The evaluator's display/print paths already go through a small number of
  `println!`/`eprintln!` calls that could be routed to a buffer

### REPL command

Add as a REPL-only direct command (Pattern 3 — same as `run`, `pwd`, `cd`):

```
report render analysis.rlnb                   → renders to report/index.html
report render analysis.rlnb output.html       → renders to custom path
```

**Parsing:** `strip_prefix("report ")` in the REPL loop, then match subcommand.

### Help entry

```rust
HelpEntry {
    name: "report",
    brief: "Render a .rlnb notebook to an HTML report",
    detail: "report render <file.rlnb>               — render to report/index.html\n\
             report render <file.rlnb> <output.html>  — render to custom path\n\n\
             Notebook files (.rlnb) are Markdown with ```rustlab fenced code blocks.\n\
             Each code block executes in sequence, sharing variables. Plots are\n\
             captured as interactive Plotly charts in the output HTML.",
}
```

Add `"report"` to a new help category `"Reports"` or append to `"Plotting"`.

**Files to modify:**
- Edit: `crates/rustlab-cli/src/commands/repl.rs` (add command dispatch + help)

---

## Phase 4 — Polish & Extras

**Optional enhancements after the core works:**

- **Syntax highlighting** in code blocks: embed a minimal highlighter that
  colorizes keywords, numbers, strings, and comments. Could be a simple
  regex-based pass in the HTML renderer (~30 lines).
- **Table of contents:** auto-generate from `#` headings in markdown blocks,
  rendered as a sticky sidebar or top nav.
- **Error handling:** if a code block fails, render the error inline in red
  and continue with remaining blocks (don't abort the whole report).
- **`report watch`:** file-watcher mode that re-renders on `.rlnb` save
  (stretch goal, not Phase 1).

---

## Files Summary

| File | Action | Phase |
|------|--------|-------|
| `crates/rustlab-plot/src/report.rs` | New — parser + HTML renderer | 1–2 |
| `crates/rustlab-plot/src/lib.rs` | Add `pub mod report` + re-exports | 1 |
| `crates/rustlab-plot/src/html.rs` | Extract div-rendering helper | 2 |
| `crates/rustlab-cli/src/commands/repl.rs` | Add `report` command + help | 3 |

---

## Verification

1. Create a test notebook `examples/reports/demo.rlnb` with markdown + code blocks
2. Run `report render examples/reports/demo.rlnb`
3. Open generated `report/index.html` in browser
4. Verify: markdown rendered, code shown, plots interactive, variables persist across blocks
5. Test edge cases: empty code blocks, code blocks with no plots, multiple plots per block
6. `cargo test --features viewer` passes
