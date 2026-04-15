//! Export the current `FIGURE` state to a self-contained HTML file using
//! Plotly.js (loaded from CDN).

use std::cell::RefCell;

use crate::error::PlotError;
use crate::figure::{FigureState, LineStyle, PlotKind, SeriesColor, FIGURE};
use crate::theme::{Theme, ThemeColors};

thread_local! {
    /// When set, every FIGURE mutation re-writes the HTML file at this path.
    static HTML_FIGURE_PATH: RefCell<Option<String>> = RefCell::new(None);
}

/// Set the active HTML figure output path. Subsequent FIGURE mutations
/// will auto-render to this file.
pub fn set_html_figure_path(path: &str) {
    HTML_FIGURE_PATH.with(|p| *p.borrow_mut() = Some(path.to_string()));
}

/// Clear the active HTML figure path (stop auto-rendering).
pub fn clear_html_figure_path() {
    HTML_FIGURE_PATH.with(|p| *p.borrow_mut() = None);
}

/// Returns true when an HTML figure path is active.
pub fn html_figure_active() -> bool {
    HTML_FIGURE_PATH.with(|p| p.borrow().is_some())
}

/// Get the current HTML figure path, if any.
pub fn get_html_figure_path() -> Option<String> {
    HTML_FIGURE_PATH.with(|p| p.borrow().clone())
}

/// If an HTML figure path is active, re-render the current FIGURE to it.
/// No-op when no path is set — safe to call unconditionally.
pub fn sync_html_file() {
    HTML_FIGURE_PATH.with(|p| {
        let p = p.borrow();
        if let Some(path) = p.as_ref() {
            let _ = render_figure_html(path);
        }
    });
}

/// Render the current thread-local FIGURE to an HTML file with Plotly.
pub fn render_figure_html(path: &str) -> Result<(), PlotError> {
    FIGURE.with(|fig| {
        let fig = fig.borrow();
        render_figure_state_html(&fig, path)
    })
}

/// Render a `FigureState` to an HTML file with Plotly (default dark theme).
pub fn render_figure_state_html(fig: &FigureState, path: &str) -> Result<(), PlotError> {
    render_figure_state_html_themed(fig, path, Theme::default().colors())
}

/// Render a `FigureState` to an HTML file with Plotly using the given theme.
pub fn render_figure_state_html_themed(fig: &FigureState, path: &str, theme: &ThemeColors) -> Result<(), PlotError> {
    let div_content = render_figure_plotly_div(fig, "plot", theme);

    let mut html = String::with_capacity(4096 + div_content.len());
    html.push_str(&format!(r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>RustLab Plot</title>
<script src="https://cdn.plot.ly/plotly-2.35.0.min.js"></script>
<style>
  body {{ margin: 0; background: {bg}; }}
  #plot {{ width: 100vw; height: 100vh; }}
</style>
</head>
<body>
"##, bg = theme.bg));
    html.push_str(&div_content);
    html.push_str(r##"</body>
</html>
"##);

    std::fs::write(path, html).map_err(|e| PlotError::FileOutput(e.to_string()))
}

/// Render a `FigureState` as a Plotly `<div>` + `<script>` fragment.
/// The `div_id` is used as the element ID for `Plotly.newPlot()`.
/// This is the shared building block for both single-file HTML export
/// and multi-figure report generation.
pub fn render_figure_plotly_div(fig: &FigureState, div_id: &str, theme: &ThemeColors) -> String {
    let rows = fig.subplot_rows;
    let cols = fig.subplot_cols;
    let n_panels = rows * cols;

    let mut traces = String::new();
    let mut layout_axes = String::new();
    let mut annotations = String::new();

    for (idx, panel) in fig.subplots.iter().enumerate().take(n_panels) {
        let row = idx / cols;
        let col = idx % cols;

        // Plotly subplot axis naming: xaxis, xaxis2, xaxis3, ...
        let axis_suffix = if idx == 0 { String::new() } else { format!("{}", idx + 1) };
        let xaxis_ref = format!("x{}", axis_suffix);
        let yaxis_ref = format!("y{}", axis_suffix);

        // Domain computation for subplot positioning
        let col_width = 1.0 / cols as f64;
        let row_height = 1.0 / rows as f64;
        let gap = 0.08;
        let x_start = col as f64 * col_width + gap / 2.0;
        let x_end = (col + 1) as f64 * col_width - gap / 2.0;
        // Plotly y-axis goes bottom-to-top, but we want row 0 at top
        let y_start = 1.0 - (row + 1) as f64 * row_height + gap / 2.0;
        let y_end = 1.0 - row as f64 * row_height - gap / 2.0;

        // Axis layout
        let show_grid = if panel.grid { "true" } else { "false" };
        // Categorical tick labels for x-axis
        let xtick_extra = if let Some(labels) = &panel.x_labels {
            let tickvals: Vec<String> = (1..=labels.len()).map(|i| i.to_string()).collect();
            let ticktext: Vec<String> = labels.iter().map(|l| format!("\"{}\"", escape_js(l))).collect();
            format!(", tickvals: [{}], ticktext: [{}]", tickvals.join(","), ticktext.join(","))
        } else {
            String::new()
        };
        layout_axes.push_str(&format!(
            r#"xaxis{ax}: {{ domain: [{x0:.4}, {x1:.4}], title: {{ text: "{xlabel}" }}{xrange}, showgrid: {grid}, gridcolor: "{plot_grid}"{xtick} }},
yaxis{ax}: {{ domain: [{y0:.4}, {y1:.4}], title: {{ text: "{ylabel}" }}{yrange}, showgrid: {grid}, gridcolor: "{plot_grid}" }},
"#,
            ax = axis_suffix,
            x0 = x_start, x1 = x_end,
            y0 = y_start, y1 = y_end,
            grid = show_grid,
            plot_grid = theme.plot_grid,
            xlabel = escape_js(&panel.xlabel),
            ylabel = escape_js(&panel.ylabel),
            xrange = format_range(panel.xlim),
            yrange = format_range(panel.ylim),
            xtick = xtick_extra,
        ));

        // Title as annotation
        if !panel.title.is_empty() {
            annotations.push_str(&format!(
                r#"{{ text: "{title}", xref: "paper", yref: "paper", x: {cx:.4}, y: {ty:.4}, showarrow: false, font: {{ size: 14 }} }},
"#,
                title = escape_js(&panel.title),
                cx = (x_start + x_end) / 2.0,
                ty = y_end + 0.03,
            ));
        }

        // Traces for each series
        for series in &panel.series {
            let color_str = color_to_css(&series.color);
            // Use WebGL backend for large traces (>10k points)
            let scatter_type = if series.x_data.len() > 10_000 { "scattergl" } else { "scatter" };
            match series.kind {
                PlotKind::Line => {
                    let dash = match series.style {
                        LineStyle::Solid => "solid",
                        LineStyle::Dashed => "dash",
                    };
                    traces.push_str(&format!(
                        r#"{{ x: {x}, y: {y}, type: "{stype}", mode: "lines", name: "{name}", line: {{ color: "{color}", dash: "{dash}" }}, xaxis: "{xa}", yaxis: "{ya}" }},
"#,
                        stype = scatter_type,
                        x = json_f64_array(&series.x_data),
                        y = json_f64_array(&series.y_data),
                        name = escape_js(&series.label),
                        color = color_str,
                        dash = dash,
                        xa = xaxis_ref,
                        ya = yaxis_ref,
                    ));
                }
                PlotKind::Scatter => {
                    traces.push_str(&format!(
                        r#"{{ x: {x}, y: {y}, type: "{stype}", mode: "markers", name: "{name}", marker: {{ color: "{color}", size: 6 }}, xaxis: "{xa}", yaxis: "{ya}" }},
"#,
                        stype = scatter_type,
                        x = json_f64_array(&series.x_data),
                        y = json_f64_array(&series.y_data),
                        name = escape_js(&series.label),
                        color = color_str,
                        xa = xaxis_ref,
                        ya = yaxis_ref,
                    ));
                }
                PlotKind::Bar => {
                    traces.push_str(&format!(
                        r#"{{ x: {x}, y: {y}, type: "bar", name: "{name}", marker: {{ color: "{color}" }}, xaxis: "{xa}", yaxis: "{ya}" }},
"#,
                        x = json_f64_array(&series.x_data),
                        y = json_f64_array(&series.y_data),
                        name = escape_js(&series.label),
                        color = color_str,
                        xa = xaxis_ref,
                        ya = yaxis_ref,
                    ));
                }
                PlotKind::Stem => {
                    // Stems: vertical lines from y=0 to each point
                    let mut sx = Vec::new();
                    let mut sy = Vec::new();
                    for (&xi, &yi) in series.x_data.iter().zip(series.y_data.iter()) {
                        sx.push(format!("{}", xi));
                        sx.push(format!("{}", xi));
                        sx.push("null".to_string());
                        sy.push("0".to_string());
                        sy.push(format!("{}", yi));
                        sy.push("null".to_string());
                    }
                    // Stem lines
                    traces.push_str(&format!(
                        r#"{{ x: [{sx}], y: [{sy}], type: "{stype}", mode: "lines", name: "{name}", line: {{ color: "{color}" }}, xaxis: "{xa}", yaxis: "{ya}", showlegend: false }},
"#,
                        stype = scatter_type,
                        sx = sx.join(","),
                        sy = sy.join(","),
                        name = escape_js(&series.label),
                        color = color_str,
                        xa = xaxis_ref,
                        ya = yaxis_ref,
                    ));
                    // Marker tips
                    traces.push_str(&format!(
                        r#"{{ x: {x}, y: {y}, type: "{stype}", mode: "markers", name: "{name}", marker: {{ color: "{color}", size: 6 }}, xaxis: "{xa}", yaxis: "{ya}" }},
"#,
                        stype = scatter_type,
                        x = json_f64_array(&series.x_data),
                        y = json_f64_array(&series.y_data),
                        name = escape_js(&series.label),
                        color = color_str,
                        xa = xaxis_ref,
                        ya = yaxis_ref,
                    ));
                }
            }
        }
    }

    // JS variable names can't contain hyphens, so replace with underscores
    let js_var = div_id.replace('-', "_");

    let mut out = String::with_capacity(4096 + traces.len());
    out.push_str(&format!(r#"<div id="{div_id}"></div>
<script>
var data_{js_var} = ["#, div_id = div_id, js_var = js_var));
    out.push_str(&traces);
    out.push_str(&format!(r##"];
var layout_{js_var} = {{
  paper_bgcolor: "{plot_bg}",
  plot_bgcolor: "{plot_bg}",
  font: {{ color: "{text}" }},
  "##, js_var = js_var, plot_bg = theme.plot_bg, text = theme.text));
    out.push_str(&layout_axes);
    out.push_str("  annotations: [");
    out.push_str(&annotations);
    out.push_str(&format!(r##"],
  margin: {{ t: 60, b: 60, l: 70, r: 30 }},
  barmode: "group",
}};
Plotly.newPlot("{div_id}", data_{js_var}, layout_{js_var}, {{ responsive: true }});
</script>
"##, div_id = div_id, js_var = js_var));

    out
}

fn color_to_css(c: &SeriesColor) -> String {
    match c {
        SeriesColor::Blue    => "rgb(31,119,180)".into(),
        SeriesColor::Red     => "rgb(214,39,40)".into(),
        SeriesColor::Green   => "rgb(44,160,44)".into(),
        SeriesColor::Cyan    => "rgb(23,190,207)".into(),
        SeriesColor::Magenta => "rgb(148,103,189)".into(),
        SeriesColor::Yellow  => "rgb(188,189,34)".into(),
        SeriesColor::Black   => "rgb(0,0,0)".into(),
        SeriesColor::White   => "rgb(255,255,255)".into(),
        SeriesColor::Rgb(r,g,b) => format!("rgb({},{},{})", r, g, b),
    }
}

fn json_f64_array(data: &[f64]) -> String {
    let mut s = String::with_capacity(data.len() * 10);
    s.push('[');
    for (i, v) in data.iter().enumerate() {
        if i > 0 { s.push(','); }
        if v.is_finite() {
            s.push_str(&format!("{}", v));
        } else {
            s.push_str("null");
        }
    }
    s.push(']');
    s
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
}

fn format_range(lim: (Option<f64>, Option<f64>)) -> String {
    match lim {
        (Some(lo), Some(hi)) => format!(", range: [{}, {}]", lo, hi),
        _ => String::new(),
    }
}
