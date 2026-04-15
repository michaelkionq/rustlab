# rustlab-notebook

Render Markdown notebooks with executable rustlab code blocks into HTML
reports, LaTeX documents, or PDF.

## Quick Start

```
rustlab-notebook render analysis.md              # → analysis.html (default)
rustlab-notebook render analysis.md -f latex     # → analysis.tex + SVG plots
rustlab-notebook render analysis.md -f pdf       # → analysis.pdf (requires pdflatex)
rustlab-notebook render analysis.md -o out.html  # explicit output path
```

## Notebook Format

Notebooks are standard `.md` files. Any fenced code block tagged
`` ```rustlab `` is executed; everything else is rendered as prose.

````markdown
# My Analysis

Design a 64-tap lowpass filter with cutoff at $f_c = 3\,\text{kHz}$:

```rustlab
h = fir_lowpass(64, 3000, 16000, "hamming");
Hw = freqz(h, 512, 16000);
plot(Hw(1,:), 20*log10(abs(Hw(2,:))))
title("Magnitude Response")
xlabel("Frequency (Hz)")
ylabel("dB")
grid on
```

The passband ripple is well within spec.
````

### What gets rendered

Each code block produces up to three zones in the output:

1. **Source** — the rustlab code (syntax-highlighted in HTML)
2. **Text output** — anything the code prints (`disp()`, `ans =`, etc.)
3. **Plot** — interactive Plotly chart (HTML) or static SVG (LaTeX/PDF)

Errors are shown inline in red. Execution continues with subsequent blocks.

### Variable persistence

Variables persist across code blocks within a notebook. Define a signal
in one block, analyze it in the next:

````markdown
```rustlab
x = sin(2*pi*0.15*(0:1023)) + 0.3*randn(1024);
```

```rustlab
X = fft(x);
plot(abs(X(1:512)))
title("Spectrum")
```
````

### Formulas

Standard LaTeX math syntax works in prose. Inline: `$f_c$` renders as
$f_c$. Display math uses `$$...$$`:

```markdown
$$H(z) = \sum_{k=0}^{N-1} h[k]\,z^{-k}$$
```

In HTML output, formulas are rendered client-side by KaTeX. In LaTeX/PDF
output, they pass through as native LaTeX.

### Tables

Markdown tables render as styled HTML tables or LaTeX `tabular` with
booktabs:

```markdown
| Window      | Main Lobe Width | First Sidelobe |
|-------------|-----------------|----------------|
| Rectangular | $2/N$           | $-13$ dB       |
| Hann        | $4/N$           | $-31$ dB       |
```

## Directives

### `<!-- hide -->`

Place `<!-- hide -->` on the line immediately before a code block to
hide the source code in the rendered output. The block is still
executed — variables, plots, and text output all appear — but the code
itself is suppressed. Useful for setup code that would distract from
the narrative.

````markdown
<!-- hide -->
```rustlab
% Load data and define constants — reader doesn't need to see this
fs = 16000;
N = 1024;
x = randn(N);
```

The signal has been sampled at 16 kHz. Now we compute the spectrum:

```rustlab
X = fft(x);
plot(abs(X(1:N/2)))
title("Spectrum")
```
````

In the output, only the second block's source code is shown. The plot
from the hidden block (if any) still appears.

## Output Formats

### HTML (default)

Self-contained HTML with:
- Catppuccin dark theme
- Interactive Plotly charts (zoom, pan, hover)
- KaTeX formula rendering
- Navigation sidebar from headings
- Syntax-highlighted code blocks
- Responsive layout (sidebar collapses on mobile)

### LaTeX (`--format latex`)

Produces a `.tex` file and a `<name>_plots/` directory of SVG images.

```
rustlab-notebook render analysis.md -f latex
# → analysis.tex
# → analysis_plots/plot-1.svg, plot-2.svg, ...
```

The `.tex` file uses `article` class with `amsmath`, `booktabs`,
`graphicx`, `svg`, `xcolor`, and `hyperref`. Formulas render natively.
Compile with any LaTeX engine that supports `\includesvg` (e.g.,
lualatex with inkscape, or pdflatex with the svg package).

### PDF (`--format pdf`)

Generates LaTeX then compiles to PDF. Requires `pdflatex` or `tectonic`
in PATH.

```
rustlab-notebook render analysis.md -f pdf
# → analysis.pdf  (also keeps analysis.tex)
```

## Frontmatter

Optional YAML frontmatter is stripped before rendering (reserved for
future use):

```markdown
---
title: Filter Analysis
author: Jane Doe
---

# Filter Analysis
...
```

## Project Layout

A typical analysis project:

```
my-project/
  config.toml              # parameters
  data/
    measurements.csv
  notebooks/
    overview.md            # narrative + code + plots
    filter_design.md
    validation.md
  scripts/
    preprocess.r           # standalone rustlab scripts
```

## Multi-Notebook Rendering

Render an entire directory of notebooks at once:

```
rustlab-notebook render notebooks/           # → *.html + index.html
rustlab-notebook render notebooks/ -f pdf    # → *.pdf
```

This produces one output file per `.md` file plus an `index.html` linking
to all notebooks (HTML format only). Each notebook gets its own independent
evaluator — variables do not leak between notebooks.

### Cross-notebook links

Links to other `.md` files are automatically rewritten to `.html` in
the rendered output:

```markdown
See [Filter Design](filter_design.md) for details.
```

becomes `<a href="filter_design.html">` in the HTML output.

## Template Interpolation

Embed computed values in markdown prose using `${expr}` syntax:

```markdown
```rustlab
n = 1024;
fs = 16000;
```

This analysis uses **${n}** samples at ${fs} Hz,
giving a duration of ${n / fs:%.3f} seconds.
```

- `${expr}` — evaluates expression and inserts its value
- `${expr:format}` — applies `sprintf`-style formatting (e.g. `%,.2f`)
- `\${...}` — escape for literal output

Expressions are evaluated against the shared notebook environment, so
any variable defined in a prior code block is available.

## String Arrays

String arrays use brace syntax and enable categorical bar chart labels:

````markdown
```rustlab
months = {"Jan", "Feb", "Mar"};
sales = [120, 95, 140];
bar(months, sales, "Monthly Sales")
```

Total sales: ${sum(sales):%,.0f} units.
````

See `docs/functions.md` → Cell Arrays for full reference.

## Examples

See `examples/notebooks/` for working examples:

- **quick_look.md** — minimal one-block notebook (random signal + plot)
- **filter_analysis.md** — FIR filter design with frequency response plots
- **spectral_estimation.md** — periodogram vs. windowed PSD, tables, display math
- **template_interpolation.md** — embedding computed values with `${expr}` and format specs
- **string_arrays.md** — string arrays, categorical bar charts, `iscell()`
- **multi_notebook.md** — directory rendering and cross-notebook links
