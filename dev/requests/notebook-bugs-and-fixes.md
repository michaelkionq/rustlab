# Notebook Rendering Bugs & Feature Requests

Found while debugging quantum_lab notebook rendering (2026-04-18).

**Status: All 5 items fixed on 2026-04-18.** See individual entries below.

---

## Bug: `stem()`, `plot()`, `bar()` reject column vectors
**Status:** Fixed 2026-04-18. `flatten_column_matrix_args` now converts Nx1 matrices to vectors at function entry for `plot/stem/bar/scatter`.

**Observed:** Calling `stem(v, "title")` where `v` is a column vector (`Matrix(Nx1)`) produces:

```
type error: stem: cannot plot Matrix(8x1)
```

**Expected:** These functions should accept both row vectors (`[1×N]`) and column vectors (`Matrix(Nx1)`). The distinction is an implementation detail that shouldn't leak to the user — a 1D dataset is a 1D dataset regardless of orientation.

**Workaround:** Use comma syntax `[a, b, c]` instead of semicolons `[a; b; c]`, or transpose with `v'` before plotting.

**Affected lessons:**
- Lesson 01: `E_ba = [0.000; 1.120; ...]` passed to `stem()`
- Lesson 02: `bz_key = [1.0; -1.0; 0.0; 0.0]` passed to `stem()`
- Lesson 02: `bz_traj` (5-element column) passed to `plot()`

---

## Bug: Only one plot captured per notebook code block
**Status:** Fixed 2026-04-18. `Rendered::Code` now carries `figures: Vec<FigureState>`; each `savefig()` under `PlotContext::Notebook` pushes a snapshot.

**Observed:** When a single `` ```rustlab `` code block contains multiple `plot()` or `imagesc()` calls (each followed by `savefig()`), only the **last** figure is captured as an inline Plotly chart. Earlier figures write their SVG to disk but produce no inline output.

**Example (lesson 01):** A block with six `imagesc()` calls for density matrices only renders the sixth heatmap inline.

**Expected:** Each `savefig()` call (or each new `figure()`) should flush the current figure to the inline output and start a new one.

**Workaround:** Split multi-plot code into separate `` ```rustlab `` blocks, one plot per block. This is verbose but works.

**Impact:** Lesson 01 rendered 11 of 22 plots, lesson 04 rendered 4 of 10, lesson 06 rendered 4 of 7 due to this issue.

---

## Feature Request: Suppress assignment echo in notebook mode
**Status:** Implemented 2026-04-18 (Option 1). Under `PlotContext::Notebook`, assignments are silent; only bare expressions, `print()`, and `disp()` produce visible text output.

**Observed:** Every assignment in a notebook code block echoes its value to the output pane. For large intermediate results (e.g., `meshgrid` returning 100×100 matrices), this produces hundreds of lines of matrix text in the rendered HTML — drowning out the meaningful `print()` output.

**Current workaround:** Append `;` to every intermediate assignment. This works but is tedious and clutters the source.

**Requested behavior (pick one or both):**
1. **Default suppression:** In notebook mode, suppress assignment echo unless the statement is a bare expression (no `=`). Only `print()` and bare expressions produce visible text output.
2. **Truncation:** If suppression isn't feasible, truncate large matrix output to e.g. 3 lines with a `... (100x100 matrix)` summary, matching REPL behavior.

Option 1 is strongly preferred — it matches Jupyter notebook behavior where assignments are silent by default.

---

## Bug: Unary minus binds tighter than `.^` (operator precedence)
**Status:** Fixed 2026-04-18. Unary minus/not moved to a `parse_unary` level that sits between `parse_term` and `parse_factor`, so `.^` and `^` now bind tighter than the prefix `-`.

**Observed:** `-x .^ 2` is parsed as `(-x) .^ 2 = x .^ 2`, losing the negation:

```
>> x = [-2.0, -1.0, 0.0, 1.0, 2.0]
>> exp(-x .^ 2)
[1×5]  54.598150  2.718282  1.000000  2.718282  54.598150
```

**Expected (Octave behavior):** `.^` should bind tighter than unary minus:

```
exp(-x .^ 2)   →  exp(-(x .^ 2))  →  [0.0183, 0.3679, 1.0, 0.3679, 0.0183]
```

**Impact:** Silent wrong results — Gaussian envelopes blow up instead of decaying. `trapz` on the corrupted data returns ~10^10 instead of ~1.0. Caught because `norm_check` in lesson 04 was wildly wrong.

**Workaround:** Use explicit parentheses: `exp(-(x .^ 2) / 2.0)`.

---

## Bug: Column vector indexing returns non-scalar
**Status:** Fixed 2026-04-18. `Value::index_1d` on a Matrix with `ncols() == 1` now returns the single-element value as a scalar (or complex) rather than a 1-element Vector.

**Observed:** Indexing a column vector `v = [1.0; 0.0]` with `v(1)` returns a 1-element vector rather than a true scalar. This causes `${var:%.3f}` template interpolation to fail:

```
ERROR: format: line 1: runtime error: fprintf %f: expected scalar, got vector
```

**Expected:** `v(1)` on any 1D vector (row or column) should return a scalar.

**Workaround:** Use `dot()` for scalar-valued expressions: `bz = real(dot(psi, Z * psi))` instead of `abs(psi(1))^2 - abs(psi(2))^2`.
