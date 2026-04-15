# Feature Request: Suppress TUI/liveplot rendering during notebook render

## Problem

Running `rustlab-notebook render` on any notebook that calls `figure()` in a code block produces TUI plots in the terminal and blocks on a keypress for each one. This makes batch rendering notebooks impractical — you have to hit Return once per plot.

### Root cause

`execute_notebook()` (`rustlab-notebook/src/execute.rs:30`) sets the figure output mode to `FigureOutput::Html(String::new())` before executing code blocks. This correctly causes `render_figure_terminal()` to early-return (see `rustlab-plot/src/ascii.rs:237`).

However, when a code block calls `figure()` with no arguments, `builtin_figure` calls `figure_new()` (`rustlab-plot/src/figure.rs:251`), which calls `default_new_output()` (`figure.rs:241`). That function returns `FigureOutput::Terminal` (or `Viewer` if connected), **overwriting the Html suppression**. From that point on, every `plot()`, `stem()`, `bar()`, `scatter()`, `imagesc()`, etc. call invokes `render_figure_terminal()` which enters the ratatui alternate screen and calls `wait_for_key()`.

### Affected code paths

- `figure.rs:241-248` — `default_new_output()` has no awareness of notebook/batch context
- `ascii.rs:234-278` — `render_figure_terminal()` enters alternate screen + waits for keypress
- `ascii.rs:281+` — `imagesc_terminal()` same pattern
- `builtins.rs:1793` — `builtin_figure()` no-arg path always creates a Terminal-mode figure

## Proposed solution

### 1. Batch/headless mode flag (new feature)

Add a thread-local or global flag (e.g. `BATCH_MODE: Cell<bool>`) in `rustlab-plot` that, when set:

- Makes `default_new_output()` return `FigureOutput::Html(String::new())` instead of `Terminal`
- Makes `render_figure_terminal()` and `imagesc_terminal()` early-return without entering alternate screen
- Prevents any `wait_for_key()` calls

`rustlab-notebook` would set this flag before executing code blocks. This is more robust than the current approach of setting `FigureOutput::Html` on a single figure, because it survives `figure()` calls.

### 2. CLI flag on `rustlab run` (new feature)

Add `--batch` or `--no-display` to `rustlab run` so standalone `.r` scripts can also run without TUI interaction:

```
rustlab run --batch rabi_oscillations.r
```

This would set the same batch mode flag internally. Useful for CI, automated rendering, and scripts that `savefig()` to disk without needing interactive display.

### 3. Fix the notebook executor (bug fix, immediate)

As a minimal fix until batch mode lands, `execute_notebook()` could re-apply `set_current_figure_output(FigureOutput::Html(String::new()))` after each code block execution (or hook into `figure_new` to preserve the override). But this is a band-aid — the batch mode flag is the right long-term fix.

## Workaround

Piping stdin from `/dev/null` avoids the keypress wait but still renders each TUI frame to the alternate screen (flicker). Not a real solution for notebook rendering.

## Affected notebooks

Any notebook with `figure()` + `plot()` in code blocks, including all six quantum_lab lessons.
