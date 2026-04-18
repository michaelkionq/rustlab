# Feature Request: Collapsible output sections (`<!-- details: Title -->`)

> **Status: Completed — 2026-04-18.** The `<!-- details: Title -->` directive is implemented in `crates/rustlab-notebook/src/parse.rs` and rendered in both HTML (`<details><summary>`) and LaTeX. Example code below uses the now-removed `saveimagesc` function; substitute `imagesc(M, cmap); savefig("file.html")` in new notebooks.

## Problem

Notebooks with many plot outputs create an overwhelming vertical scroll. Lesson 01 in quantum_lab produces 23 plots (6 density matrix heatmaps, 6 cloud visualizations, 6 radial probability curves, etc.) — the rendered HTML is a wall of images that buries the narrative. There is no way to hide secondary visualizations while keeping them accessible.

## Proposed directive

Place `<!-- details: Title -->` on the line immediately before a code block. The renderer wraps that block's output (plots, text, and optionally source) in a collapsed disclosure widget. The code block still executes and all variables persist — only the rendered output is hidden behind a click.

### Usage

````markdown
The key insight is that superposition and mixed states differ in their off-diagonal elements.

<!-- details: Density Matrix Visualizations -->
```rustlab
saveimagesc(real(rho_g),     "outputs/rho_ground.svg",        "Pure Ground |0>",             "viridis")
saveimagesc(real(rho_e),     "outputs/rho_excited.svg",       "Pure Excited |1>",            "viridis")
saveimagesc(real(rho_sup),   "outputs/rho_superposition.svg", "Superposition |+> (real)",    "viridis")
saveimagesc(real(rho_mixed), "outputs/rho_mixed.svg",         "Mixed State (classical 50%)", "viridis")
```

Purity confirms the difference: ...
````

### Rendered output

The prose flows normally. Between the two paragraphs, a single clickable line appears:

```
▶ Density Matrix Visualizations
```

Clicking it expands to reveal the 4 heatmaps. The reader can collapse it again after inspecting them.

## Output format mapping

| Format | Rendering |
|---|---|
| HTML | `<details><summary>Title</summary>` ... `</details>` |
| LaTeX | `\begin{tcolorbox}[breakable, collapseible, title=Title]` or a custom environment. Could also use `\paragraph{Title}` with a visual separator if collapsibility isn't feasible in print. |
| PDF | Same as LaTeX |

## Interaction with other directives

- `<!-- hide -->` + `<!-- details: Title -->`: source code is hidden, output is collapsed. Useful for "supplementary figure" blocks where the reader doesn't need to see either the code or the images by default.
- `<!-- details: Title -->` alone: source code is shown (expanded), output is collapsed.

## Motivation from quantum_lab

Specific blocks that would benefit:

| Lesson | Block | Plot count | Purpose |
|---|---|---|---|
| 01 | Density matrices | 6 heatmaps | Secondary visualization of rho states |
| 01 | 2D probability clouds | 3+3 heatmaps | Individual orbitals + comparison grid |
| 02 | Gate matrix heatmaps | 6 heatmaps | Visual reference for 2x2 matrices |
| 04 | Fock state wavefunctions | 5 line plots | Individual psi_n(xi) for n=0..4 |
| 04 | JC Rabi oscillations | 3 line plots | Individual P_e(t) for n=0,1,4 |
