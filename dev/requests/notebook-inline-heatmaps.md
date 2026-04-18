# Feature Request: Inline heatmap rendering for `saveimagesc` in notebooks

> **Status: Completed (superseded) — 2026-04-18.** The `saveimagesc` builtin has been removed; the supported pattern is `imagesc(M, cmap); savefig("file.html")`. Commit `a328c8a` fixed `report_auto_capture` to also capture heatmap-only figures, so this pattern now produces an inline Plotly heatmap in notebook output. Examples below reference the removed `saveimagesc` function and are retained only for historical context.

## Problem

When a notebook code block calls `saveimagesc(matrix, "file.svg", "title", "colormap")`, the SVG file is written to disk but **nothing is embedded in the rendered notebook**. The reader sees the source code and text output but no image. This is inconsistent with `savestem`, `savefig` (1D data), and `savebar`, which all produce inline Plotly charts in notebook output.

This affects 21 visualizations across the quantum_lab notebooks — all of the 2D heatmap content:

| Lesson | Missing outputs | Content |
|---|---|---|
| 01 — Atomic Physics | 14 | Hydrogen orbital clouds (1s/2s/3s), orbital size comparisons, barium 6s cloud, hydrogen-on-barium-scale comparison, 6 density matrix heatmaps |
| 02 — Quantum Gates | 6 | Gate matrix heatmaps (Pauli X, Y, Z, Hadamard H, T gate real/imag) |
| 06 — Entanglement | 1 | Bell state density matrix Re(rho) for \|Phi+\> |

The orbital clouds and density matrices are the most visually striking parts of the tutorial. Without inline rendering, the notebook narrative references images the reader cannot see.

## Current behavior

1D plot functions in notebooks:
- `savestem(vec, "file.svg", "title")` — saves SVG **and** produces inline Plotly scatter/stem chart
- `savefig(vec, "file.svg", "title")` — saves SVG **and** produces inline Plotly line chart
- `savebar(vec, "file.svg", "title")` — saves SVG **and** produces inline Plotly bar chart
- `figure(); plot(...); savefig("file.html")` — produces inline Plotly chart

2D plot function in notebooks:
- `saveimagesc(mat, "file.svg", "title", "cmap")` — saves SVG, **no inline output**

## Proposed fix

In notebook mode, `saveimagesc` should produce an inline Plotly heatmap chart, consistent with how the other save functions work. The SVG file should still be written to disk (for use by standalone scripts and LaTeX output).

### Expected Plotly output

```javascript
var data = [{
  z: [[row0], [row1], ...],   // the matrix data
  type: "heatmap",
  colorscale: "Viridis",       // mapped from the rustlab colormap name
  showscale: true
}];
var layout = {
  title: "H 3s: |psi|^2 — two node rings",
  yaxis: { scaleanchor: "x" }  // square aspect ratio for spatial data
};
Plotly.newPlot("plot-N", data, layout, { responsive: true });
```

This gives the reader:
- The heatmap displayed inline in the notebook
- Hover to read matrix values at any pixel
- Zoom/pan to inspect fine structure (radial nodes in orbital clouds, off-diagonal coherences in density matrices)
- Consistent visual style with the 1D Plotly charts already in the notebook

### Colormap mapping

The rustlab colormap name (currently always `"viridis"` in quantum_lab) maps to Plotly's built-in `"Viridis"` colorscale. Other rustlab colormaps should map to their Plotly equivalents or a closest match.

### Aspect ratio

For square matrix data (the common case in quantum_lab — 100x100 orbital clouds, 2x2 density matrices), the Plotly layout should enforce square pixels via `yaxis.scaleanchor: "x"`. This preserves the spatial meaning of the heatmap (equal scaling in x and z for orbital clouds).

## Output format mapping

| Format | Rendering |
|---|---|
| HTML | Inline Plotly heatmap chart (interactive: hover, zoom, pan) |
| LaTeX | `\includesvg` or `\includegraphics` referencing the saved SVG file in the `_plots/` directory |
| PDF | Same as LaTeX (static SVG embedded via the existing plot pipeline) |

## Scope

This request covers `saveimagesc` only. The `imagesc()` function (which renders to the TUI figure) should also produce inline output in notebook mode when followed by `savefig("file.html")`, but that is a separate code path and could be handled as a follow-up.
