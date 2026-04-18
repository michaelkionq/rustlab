# Plan: Add `heatmap()` and `image()` Builtins

**Status**: Approved, not started  
**Date**: 2026-04-18

## Context

rustlab has `imagesc(M)` which auto-scales matrix values to a colormap. Two related visualization functions are missing:

- **`heatmap()`** — like `imagesc` but with categorical axis labels (row/column names), more like a data table visualization
- **`image()`** — raw pixel display with no min/max normalization; values 0-255 map directly to colors; supports RGB via three matrices

Both must work across all output backends: **notebooks** (Plotly HTML), **savefig** (SVG/PNG/HTML), and **rustlab-viewer** (egui). Terminal rendering is **best-effort** (reuse existing `imagesc_terminal` approach where possible; labels in the TUI are not required).

## Design Decisions

1. **Extend `HeatmapData` rather than adding new structs.** All four backends already dispatch on `SubplotState.heatmap: Option<HeatmapData>`. A `HeatmapKind` enum discriminates behavior at render time.

2. **`image()` RGB: pre-merge to RGBA in the builtin.** The viewer already expects RGBA (`WireHeatmap`), Plotly `type: "image"` wants `[r,g,b,a]` per pixel, and plotters draws colored rectangles. Storing three separate z-matrices provides no benefit. An `rgba: Option<Vec<u8>>` field on `HeatmapData` carries the pre-rendered pixels.

3. **Plotly `type: "image"` for image().** Native Plotly trace type with `z = [[[r,g,b,a], ...], ...]`. Simpler than base64 PNG and gives zoom/pan/hover for free. Supported since Plotly.js 1.54 (rustlab uses 2.35.0).

4. **Refactor `imagesc_terminal` to split FIGURE push from TUI draw.** Both new builtins need to push richer `HeatmapData` than `imagesc_terminal` would, then separately trigger the TUI render.

## Existing Architecture (read this first)

The heatmap data flow is:

```
User Code (imagesc/heatmap/image)
    |
builtin function (builtins.rs)
    |
Pushes HeatmapData into FIGURE thread-local state (figure.rs)
    |
    +---> Terminal: colormap_rgb(t) -> colored blocks via ratatui (ascii.rs)
    +---> HTML/Plotly: z matrix -> JSON, colorscale mapping (html.rs)
    +---> SVG/PNG: render_imagesc_to_backend -> plotters rectangles (file.rs)
    +---> Report: auto-capture figure -> HTML export (report.rs)
    +---> Viewer: normalize z -> colormap_rgb -> RGBA pixels
    |       -> ViewerMsg::PanelHeatmap { WireHeatmap {width, height, rgba} }
    |       -> rustlab-viewer creates egui texture (viewer_live.rs -> app.rs)
    +---> Notebook: executor captures FIGURE state after code block,
            renders via render_figure_plotly_div (execute.rs -> html.rs)
```

### Key structs and locations

| What | File | Line |
|------|------|------|
| `HeatmapData { z, colorscale }` | `crates/rustlab-plot/src/figure.rs` | ~85 |
| `SubplotState.heatmap: Option<HeatmapData>` | `crates/rustlab-plot/src/figure.rs` | ~105 |
| `SubplotState.x_labels: Option<Vec<String>>` | `crates/rustlab-plot/src/figure.rs` | ~103 |
| `colormap_rgb(t, name) -> (u8,u8,u8)` | `crates/rustlab-plot/src/figure.rs` | ~389 |
| `imagesc_terminal()` | `crates/rustlab-plot/src/ascii.rs` | ~290 |
| `builtin_imagesc()` | `crates/rustlab-script/src/eval/builtins.rs` | ~2026 |
| `builtin_saveimagesc()` (deprecated) | `crates/rustlab-script/src/eval/builtins.rs` | ~2049 |
| Plotly heatmap trace generation | `crates/rustlab-plot/src/html.rs` | ~164 |
| SVG/PNG heatmap rendering | `crates/rustlab-plot/src/file.rs` | ~72, ~280 |
| Viewer RGBA pre-rendering | `crates/rustlab-plot/src/viewer_live.rs` | ~261 |
| `WireHeatmap { width, height, rgba }` | `crates/rustlab-proto/src/lib.rs` | ~103 |
| Viewer egui texture creation | `crates/rustlab-viewer/src/figure.rs` | ~133 |
| Notebook figure capture | `crates/rustlab-notebook/src/execute.rs` | ~59 |

### How `imagesc_terminal` works today

1. Extracts `.norm()` of each complex element -> `vals: Vec<f64>`
2. Computes min/max for normalization
3. Builds `z: Vec<Vec<f64>>` row-major
4. **Pushes** `HeatmapData { z, colorscale }` into `FIGURE` thread-local (lines ~300-317)
5. Early-returns if Notebook context or non-terminal stdout
6. Renders colored blocks via crossterm/ratatui using `colormap_rgb(t, colormap)`

The problem for new builtins: step 4 pushes a plain `HeatmapData`. The new builtins need to push enriched data (with labels or RGBA), so they can't delegate the FIGURE push to `imagesc_terminal`.

## Phase 1: Extend data model

**File: `crates/rustlab-plot/src/figure.rs`**

Add enum before `HeatmapData` (~line 83):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum HeatmapKind {
    /// imagesc: continuous values, min/max normalization, colormap applied
    Imagesc,
    /// heatmap: like Imagesc but with categorical axis labels
    Heatmap,
    /// image: raw RGBA pixel data, no normalization
    ImageRgba,
}
```

Extend `HeatmapData`:

```rust
pub struct HeatmapData {
    pub z: Vec<Vec<f64>>,
    pub colorscale: String,
    pub kind: HeatmapKind,                  // NEW
    pub x_labels: Option<Vec<String>>,      // NEW - column labels (Heatmap kind)
    pub y_labels: Option<Vec<String>>,      // NEW - row labels (Heatmap kind)
    pub rgba: Option<Vec<u8>>,              // NEW - pre-rendered pixels (ImageRgba kind)
    pub rgba_width: u32,                    // NEW
    pub rgba_height: u32,                   // NEW
}
```

**Update all existing `HeatmapData` construction sites** with default values for new fields. There are exactly 2 sites:

1. `crates/rustlab-plot/src/ascii.rs` line ~313 (inside `imagesc_terminal`)
2. `crates/rustlab-script/src/eval/builtins.rs` inside `builtin_saveimagesc` (~line 2068)

Add to each: `kind: HeatmapKind::Imagesc, x_labels: None, y_labels: None, rgba: None, rgba_width: 0, rgba_height: 0`

**Export `HeatmapKind`** from `crates/rustlab-plot/src/lib.rs` (line ~30, alongside existing `HeatmapData` export).

## Phase 2: Refactor terminal rendering

**File: `crates/rustlab-plot/src/ascii.rs`**

Split `imagesc_terminal` (~line 290) into two functions:

### `render_heatmap_tui()`
Extract the pure TUI rendering portion (lines ~324-380) into a standalone function:
```rust
pub fn render_heatmap_tui(
    vals: &[f64], nrows: usize, ncols: usize,
    min_v: f64, range: f64, title: &str, colormap: &str,
) -> Result<(), PlotError>
```
This does the crossterm EnterAlternateScreen, ratatui colored blocks, wait-for-key, restore. Does NOT touch FIGURE state. Includes the existing `IsTerminal` and `PlotContext::Notebook` early-return guards.

### Refactored `imagesc_terminal()`
Becomes:
1. Extract vals, compute min/max/range (existing code)
2. Build z-matrix, push `HeatmapData { kind: Imagesc, ... }` into FIGURE (existing code)
3. Call `render_heatmap_tui(vals, nrows, ncols, min_v, range, title, colormap)`

### `render_image_tui()`
New function for `image()` best-effort terminal rendering:
```rust
pub fn render_image_tui(
    rgba: &[u8], width: usize, height: usize, title: &str,
) -> Result<(), PlotError>
```
Same approach as `render_heatmap_tui` but reads `(r, g, b)` directly from the RGBA buffer instead of computing via colormap. Include `IsTerminal` + `Notebook` guards.

Export all three from `crates/rustlab-plot/src/lib.rs`.

## Phase 3: Add builtins

**File: `crates/rustlab-script/src/eval/builtins.rs`**

### `heatmap()`

Register (~line 152, after `imagesc`): `r.register("heatmap", builtin_heatmap);`

Signatures:
- `heatmap(M)` - numeric indices as labels
- `heatmap(M, "title")` - with title
- `heatmap(xlabels, ylabels, M)` - categorical string array labels
- `heatmap(xlabels, ylabels, M, "title")`
- `heatmap(xlabels, ylabels, M, "title", "colormap")`

Where `xlabels`/`ylabels` are string arrays like `["Mon", "Tue", "Wed"]`.

Implementation:
1. Parse args - detect leading `Value::StringArray` types vs matrix/scalar
2. Extract matrix, take `.norm()` for complex support (same as imagesc)
3. Validate label lengths: `xlabels.len() == ncols`, `ylabels.len() == nrows`
4. Build `z: Vec<Vec<f64>>` from matrix values
5. Push into FIGURE:
   ```rust
   sp.heatmap = Some(HeatmapData {
       z,
       colorscale: colormap.to_string(),
       kind: HeatmapKind::Heatmap,
       x_labels: Some(xlabels), // or None if no labels provided
       y_labels: Some(ylabels), // or None if no labels provided
       rgba: None,
       rgba_width: 0,
       rgba_height: 0,
   });
   ```
6. Call `render_heatmap_tui()` for terminal display (best-effort, no labels in TUI)
7. Call `sync_figure_outputs()`

**Note on string arrays**: Check how existing builtins parse `Value::StringArray`. The bar chart builtin (`builtin_bar`) already handles string array labels at ~line 4740. Follow that pattern.

### `image()`

Register: `r.register("image", builtin_image);`

Signatures:
- `image(M)` - grayscale, values clamped 0-255
- `image(M, "colormap")` - single channel mapped through colormap
- `image(R, G, B)` - three matrices for true-color RGB

Implementation:
1. Parse args:
   - 1 arg (matrix) -> grayscale
   - 2 args (matrix, string) -> matrix + colormap
   - 3 args (matrix, matrix, matrix) -> RGB
2. Build RGBA buffer:
   - **Grayscale**: `v = val.norm().clamp(0.0, 255.0) as u8; [v, v, v, 255]`
   - **Colormap**: `t = val.norm().clamp(0.0, 255.0) / 255.0; colormap_rgb(t, name) -> [r, g, b, 255]`
   - **RGB**: `r = R[i][j].re.clamp(0.0, 255.0) as u8` (use `.re` not `.norm()` for RGB channels), same for g, b; `[r, g, b, 255]`
3. Push into FIGURE:
   ```rust
   sp.heatmap = Some(HeatmapData {
       z: vec![],  // not used for ImageRgba
       colorscale: String::new(),
       kind: HeatmapKind::ImageRgba,
       x_labels: None,
       y_labels: None,
       rgba: Some(rgba_buf),
       rgba_width: ncols as u32,
       rgba_height: nrows as u32,
   });
   ```
4. Call `render_image_tui()` for terminal (best-effort)
5. Call `sync_figure_outputs()`

## Phase 4: Update rendering backends

### 4a. Plotly/HTML

**File: `crates/rustlab-plot/src/html.rs`**

In `render_figure_plotly_div`, expand the heatmap trace block (~line 164) to branch on `hm.kind`:

**`Imagesc`** (existing): No change. Emits `type: "heatmap"` with z-matrix JSON.

**`Heatmap`**: Emit `type: "heatmap"` with z-matrix PLUS `x` and `y` text arrays:
```javascript
{
  z: [[1,2,3],[4,5,6]],
  x: ["Mon","Tue","Wed"],
  y: ["Alice","Bob"],
  type: "heatmap",
  colorscale: "Viridis",
  showscale: true,
  xaxis: "x", yaxis: "y"
}
```
Plotly handles categorical axes natively from text x/y arrays.

**`ImageRgba`**: Emit `type: "image"` trace. Build z as 3D JSON array where each pixel is `[r,g,b,a]`:
```javascript
{
  z: [[[r,g,b,a],[r,g,b,a],...], ...],
  type: "image",
  xaxis: "x", yaxis: "y"
}
```
No colorscale needed. Skip the `scaleanchor` square aspect ratio logic for images.

### 4b. SVG/PNG

**File: `crates/rustlab-plot/src/file.rs`**

In `render_to_backend` heatmap branch (~line 72), branch on `hm.kind`:

**`Imagesc`**: No change (existing `render_imagesc_to_backend` call).

**`Heatmap`**: Call `render_imagesc_to_backend` (same colored rectangles). Optionally add axis label rendering via plotters tick formatters if feasible, otherwise just use the same rendering as Imagesc (labels will show in the Plotly/HTML output).

**`ImageRgba`**: New helper `render_image_rgba_to_backend`:
```rust
fn render_image_rgba_to_backend<DB>(
    root: DrawingArea<DB, ...>, width: usize, height: usize,
    rgba: &[u8], caption: &str,
) -> Result<(), PlotError>
```
Draw colored rectangles from RGBA directly. Each pixel at `(col, row)`:
```rust
let offset = (row * width + col) * 4;
let color = RGBColor(rgba[offset], rgba[offset+1], rgba[offset+2]);
```
Row 0 at top (image convention), so y-coordinate = `row` not `height - 1 - row`.

### 4c. Viewer

**File: `crates/rustlab-plot/src/viewer_live.rs`**

In `send_figure_state` heatmap branch (~line 261), branch on `hm.kind`:

**`Imagesc`**: No change (existing normalize + colormap_rgb -> RGBA path).

**`Heatmap`**: Same RGBA rendering as Imagesc. The viewer shows the image as a texture. Categorical labels are not rendered in the viewer egui texture in v1 (acceptable - the viewer is a live preview, detailed labels show in HTML/savefig output).

**`ImageRgba`**: Use `hm.rgba` directly, skip normalization:
```rust
if hm.kind == HeatmapKind::ImageRgba {
    if let Some(ref rgba) = hm.rgba {
        conn.client.send_nowait(&ViewerMsg::PanelHeatmap {
            fig_id,
            panel: idx as u16,
            heatmap: WireHeatmap {
                width: hm.rgba_width,
                height: hm.rgba_height,
                rgba: rgba.clone(),
            },
        })?;
    }
}
```

**No changes needed** to:
- `crates/rustlab-proto/src/lib.rs` - `WireHeatmap` already carries arbitrary RGBA
- `crates/rustlab-viewer/src/app.rs` - already renders any RGBA texture it receives
- `crates/rustlab-viewer/src/figure.rs` - same

## Phase 5: REPL help entries

**File: `crates/rustlab-cli/src/commands/repl.rs`**

Add `HelpEntry` for `heatmap` and `image` after the `imagesc` entry (~line 230):

```rust
HelpEntry { name: "heatmap", brief: "Labeled heatmap with categorical axis labels",
    detail: "heatmap(M)\nheatmap(M, \"title\")\nheatmap(xlabels, ylabels, M)\nheatmap(xlabels, ylabels, M, \"title\")\nheatmap(xlabels, ylabels, M, \"title\", \"colormap\")\n\n  xlabels, ylabels: string arrays [\"Mon\",\"Tue\",\"Wed\"]\n  colormap: \"viridis\" (default), \"jet\", \"hot\", \"gray\"" },
HelpEntry { name: "image", brief: "Raw pixel display (no normalization, values 0-255)",
    detail: "image(M)              -- grayscale (values 0-255)\nimage(M, \"colormap\")  -- single channel with colormap\nimage(R, G, B)        -- true-color RGB (values 0-255)\n\nValues clamped to [0, 255]. No min/max normalization (unlike imagesc)." },
```

Add `"heatmap"` and `"image"` to the "Plotting" category list (~line 863).

## Files to Modify (summary)

| File | Change |
|------|--------|
| `crates/rustlab-plot/src/figure.rs` | Add `HeatmapKind` enum, extend `HeatmapData` with 5 new fields |
| `crates/rustlab-plot/src/lib.rs` | Export `HeatmapKind`, new TUI functions |
| `crates/rustlab-plot/src/ascii.rs` | Refactor `imagesc_terminal` into push + render; add `render_heatmap_tui`, `render_image_tui` |
| `crates/rustlab-script/src/eval/builtins.rs` | Add `builtin_heatmap`, `builtin_image`; register both; update 1 existing `HeatmapData` construction site |
| `crates/rustlab-plot/src/html.rs` | Branch on `HeatmapKind` for Plotly trace generation |
| `crates/rustlab-plot/src/file.rs` | Branch on `HeatmapKind` for SVG/PNG rendering |
| `crates/rustlab-plot/src/viewer_live.rs` | Branch on `HeatmapKind` for viewer RGBA dispatch |
| `crates/rustlab-cli/src/commands/repl.rs` | Add help entries + category for `heatmap`, `image` |

## Build Order

Phases must be done in order because of compile dependencies:
1. **Phase 1** (data model) - everything else depends on the new fields
2. **Phase 2** (ascii.rs refactor) - builtins call the new TUI functions
3. **Phase 3** (builtins) - can partially overlap with Phase 4
4. **Phase 4** (rendering backends) - html.rs, file.rs, viewer_live.rs are independent of each other
5. **Phase 5** (help entries) - independent, do last

Build and test after each phase to catch errors early.

## Verification

1. `cargo test --workspace` - all tests pass, existing imagesc behavior unchanged
2. Run `examples/matrix_ops.r` - existing imagesc still works
3. Notebook test: create a notebook with `heatmap()` and `image()` calls, `cargo run -- notebook render`, verify Plotly charts appear correctly in the HTML
4. `savefig` test:
   - `heatmap(["A","B","C"], ["X","Y"], [1,2,3;4,5,6], "Test"); savefig("/tmp/hm.svg")` - produces SVG
   - `image(randn(8,8)*128+128); savefig("/tmp/img.png")` - produces PNG
   - `M = randn(8,8)*128+128; image(M, M, M); savefig("/tmp/rgb.html")` - produces HTML with image
5. Viewer test: `viewer on; heatmap(eye(4)); image(randn(8,8)*128+128)` - both render in viewer window
6. Terminal test: `heatmap(eye(4))` and `image(randn(8,8)*128+128)` render colored blocks in REPL (best-effort)
