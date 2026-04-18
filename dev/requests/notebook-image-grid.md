# Feature Request: Image grid layout (`<!-- grid: N -->`)

> **Status: Completed — 2026-04-18.** The `<!-- grid: N -->` directive is implemented in `crates/rustlab-notebook/src/parse.rs` and rendered by both `render.rs` (HTML) and `render_latex.rs` (LaTeX/PDF). Example code below uses the now-removed `saveimagesc` function; substitute `imagesc(M, cmap); savefig("file.html")` in new notebooks.

## Problem

When a code block produces multiple static images via `saveimagesc` or `savefig`, each image renders at full width in its own row. For small images (2x2 gate matrix heatmaps, side-by-side orbital comparisons), this wastes space and prevents visual comparison. Lesson 01 has a block that produces 3 orbital clouds meant to be compared at the same scale — stacking them vertically forces the reader to scroll back and forth.

## Proposed directive

Place `<!-- grid: N -->` on the line immediately before a code block. The renderer tiles that block's image outputs N-across in a responsive layout instead of stacking them vertically.

### Usage

````markdown
Plotting all three s-orbitals on the same 35 $a_0$ grid makes the $n^2$ size scaling dramatic:

<!-- grid: 3 -->
```rustlab
saveimagesc(P_1s_cmp, "outputs/compare_1s_35a0.svg", "H 1s on 35a0 grid (tiny)", "viridis")
saveimagesc(P_2s_cmp, "outputs/compare_2s_35a0.svg", "H 2s on 35a0 grid",        "viridis")
saveimagesc(P_3s_cmp, "outputs/compare_3s_35a0.svg", "H 3s on 35a0 grid",        "viridis")
```
````

### Rendered output

The three heatmaps appear side-by-side in a single row, each taking ~1/3 of the content width. Titles appear as captions below each image.

## Output format mapping

| Format | Rendering |
|---|---|
| HTML | CSS grid: `display: grid; grid-template-columns: repeat(N, 1fr); gap: 1rem;`. Responsive: collapse to fewer columns on narrow viewports. |
| LaTeX | `\begin{figure}` with N `\begin{minipage}{width}` blocks, or `\begin{subfigure}` with `\subcaption`. |
| PDF | Same as LaTeX |

## Behavior

- `N` is the maximum number of columns. If the block produces fewer than `N` images, use as many columns as there are images.
- If the block produces more than `N` images, wrap to additional rows.
- Text output (from `print`, `fprintf`) appears above the grid, full-width.
- Interactive Plotly charts (from `savefig("*.html")`) are excluded from the grid and render full-width as usual (they need room for interactivity).
- Source code (if not hidden) appears full-width above everything.

## Interaction with other directives

- `<!-- grid: 3 -->` + `<!-- details: Title -->`: collapsed by default, expands to reveal a 3-column grid. Useful for "supplementary gallery" blocks.
- `<!-- grid: 2 -->` + `<!-- hide -->`: hidden source, images tiled 2-across.

## Motivation from quantum_lab

| Lesson | Block | Images | Ideal N |
|---|---|---|---|
| 01 | Orbital size comparison | 3 heatmaps (1s/2s/3s on 35a0 grid) | 3 |
| 01 | Density matrices | 6 heatmaps (2x2 matrices) | 3 or 2 |
| 01 | Individual cloud visualizations | 3 heatmaps (1s/2s/3s) | 3 |
| 02 | Gate matrix heatmaps | 6 heatmaps (2x2 matrices) | 3 |
| 04 | Fock state wavefunctions | 5 line plots | 3 |
| 06 | Bell state probabilities | 2 bar charts | 2 |
