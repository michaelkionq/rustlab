use plotters::prelude::*;
use rustlab_core::{CMatrix, CVector, RVector};
use crate::error::PlotError;
use crate::compute_histogram;
use crate::figure::{colormap_rgb, FigureState, LineStyle, PlotKind, SubplotState, FIGURE};

const MARGIN: u32 = 20;
const X_LABEL_AREA: u32 = 50;
const Y_LABEL_AREA: u32 = 70;

/// Format a float compactly (no %g in Rust).
fn fmt_g(v: f64) -> String {
    if v == 0.0 { return "0".to_string(); }
    let abs = v.abs();
    if abs < 0.001 || abs >= 10000.0 {
        format!("{:.2e}", v)
    } else if abs >= 100.0 {
        format!("{:.0}", v)
    } else if abs >= 10.0 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}

// ─── Main render entry points ───────────────────────────────────────────────

/// Render the current FIGURE state to a file (PNG or SVG by extension).
pub fn render_figure_file(path: &str) -> Result<(), PlotError> {
    FIGURE.with(|fig| {
        let fig = fig.borrow();
        let rows = fig.subplot_rows;
        let cols = fig.subplot_cols;
        let w = (cols as u32 * 900).min(3600);
        let h = (rows as u32 * 500).min(3000);

        if path.ends_with(".svg") {
            let root = SVGBackend::new(path, (w, h)).into_drawing_area();
            render_to_backend(root, &fig, rows, cols)
        } else {
            let root = BitMapBackend::new(path, (w, h)).into_drawing_area();
            render_to_backend(root, &fig, rows, cols)
        }
    })
}

fn render_to_backend<DB>(
    root: DrawingArea<DB, plotters::coord::Shift>,
    fig: &FigureState,
    rows: usize,
    cols: usize,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync + 'static,
{
    let err = |e: DrawingAreaErrorKind<DB::ErrorType>| PlotError::FileOutput(e.to_string());
    root.fill(&WHITE).map_err(err)?;

    let panels: Vec<_> = root.split_evenly((rows, cols));

    for (idx, panel) in panels.iter().enumerate() {
        if idx >= fig.subplots.len() { break; }
        let sp = &fig.subplots[idx];
        if sp.series.is_empty() { continue; }
        render_subplot_to_panel(panel, sp)?;
    }
    root.present().map_err(err)?;
    Ok(())
}

fn render_subplot_to_panel<DB>(
    panel: &DrawingArea<DB, plotters::coord::Shift>,
    sp: &SubplotState,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync + 'static,
{
    let err = |e: DrawingAreaErrorKind<DB::ErrorType>| PlotError::FileOutput(e.to_string());

    // Compute axis bounds
    let all_x: Vec<f64> = sp.series.iter().flat_map(|s| s.x_data.iter().copied()).collect();
    let all_y: Vec<f64> = sp.series.iter().flat_map(|s| s.y_data.iter().copied()).collect();
    if all_x.is_empty() || all_y.is_empty() { return Ok(()); }

    let x_min = sp.xlim.0.unwrap_or_else(|| all_x.iter().copied().fold(f64::INFINITY, f64::min));
    let x_max = sp.xlim.1.unwrap_or_else(|| all_x.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    let y_min_raw = all_y.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max_raw = all_y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_margin = ((y_max_raw - y_min_raw).abs() * 0.1).max(1e-6);
    let y_min = sp.ylim.0.unwrap_or(y_min_raw - y_margin);
    let y_max = sp.ylim.1.unwrap_or(y_max_raw + y_margin);

    // Ensure non-degenerate range
    let x_lo = if (x_max - x_min).abs() < 1e-12 { x_min - 1.0 } else { x_min };
    let x_hi = if (x_max - x_min).abs() < 1e-12 { x_max + 1.0 } else { x_max };
    let y_lo = if (y_max - y_min).abs() < 1e-12 { y_min - 1.0 } else { y_min };
    let y_hi = if (y_max - y_min).abs() < 1e-12 { y_max + 1.0 } else { y_max };

    let title_str = sp.title.as_str();
    let xlabel = if !sp.xlabel.is_empty() { sp.xlabel.as_str() } else { "x" };
    let ylabel = if !sp.ylabel.is_empty() { sp.ylabel.as_str() } else { "y" };

    let mut chart = ChartBuilder::on(panel)
        .caption(title_str, ("sans-serif", 22u32).into_font())
        .margin(MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d(x_lo..x_hi, y_lo..y_hi)
        .map_err(err)?;

    chart.configure_mesh()
        .disable_mesh()
        .x_desc(xlabel)
        .y_desc(ylabel)
        .draw()
        .map_err(err)?;

    if sp.grid {
        const N: usize = 5;
        let grid_color = plotters::style::RGBAColor(100, 100, 100, 0.35);
        for i in 0..=N {
            let yv = y_lo + (y_hi - y_lo) * i as f64 / N as f64;
            chart.draw_series(LineSeries::new(vec![(x_lo, yv), (x_hi, yv)], grid_color.stroke_width(1))).map_err(err)?;
        }
        for i in 1..N {
            let xv = x_lo + (x_hi - x_lo) * i as f64 / N as f64;
            chart.draw_series(LineSeries::new(vec![(xv, y_lo), (xv, y_hi)], grid_color.stroke_width(1))).map_err(err)?;
        }
    }

    // Draw each series
    for s in &sp.series {
        let rgb = s.color.to_plotters();
        let stroke_width: u32 = if s.style == LineStyle::Dashed { 1 } else { 2 };
        let color = rgb.stroke_width(stroke_width);

        match s.kind {
            PlotKind::Line => {
                let pts: Vec<(f64, f64)> = s.x_data.iter().copied()
                    .zip(s.y_data.iter().copied())
                    .collect();

                if s.style == LineStyle::Dashed {
                    // Simulate dashed by drawing every other segment
                    let mut draw_seg = true;
                    for pair in pts.windows(2) {
                        if draw_seg {
                            chart.draw_series(LineSeries::new(
                                vec![pair[0], pair[1]],
                                color,
                            )).map_err(err)?;
                        }
                        draw_seg = !draw_seg;
                    }
                } else {
                    chart.draw_series(LineSeries::new(pts, color)).map_err(err)?;
                }
            }
            PlotKind::Stem => {
                // Baseline
                let x_lo_s = s.x_data.iter().copied().fold(f64::INFINITY, f64::min);
                let x_hi_s = s.x_data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                chart.draw_series(LineSeries::new(
                    vec![(x_lo_s, 0.0), (x_hi_s, 0.0)],
                    BLACK.stroke_width(1),
                )).map_err(err)?;

                // Stems
                chart.draw_series(
                    s.x_data.iter().copied().zip(s.y_data.iter().copied()).map(|(x, y)| {
                        PathElement::new(vec![(x, 0.0), (x, y)], color)
                    })
                ).map_err(err)?;

                // Tips
                chart.draw_series(
                    s.x_data.iter().copied().zip(s.y_data.iter().copied()).map(|(x, y)| {
                        Circle::new((x, y), 3, rgb.filled())
                    })
                ).map_err(err)?;
            }
        }
    }
    Ok(())
}

// ─── imagesc file output ────────────────────────────────────────────────────

/// Save a matrix as an imagesc heatmap with named colormap.
pub fn save_imagesc_cmap(matrix: &CMatrix, title: &str, colormap: &str, path: &str) -> Result<(), PlotError> {
    let (nrows, ncols) = (matrix.nrows(), matrix.ncols());
    if nrows == 0 || ncols == 0 { return Err(PlotError::EmptyData); }

    let vals: Vec<f64> = matrix.iter().map(|c| c.norm()).collect();
    let min_v = vals.iter().copied().fold(f64::INFINITY, f64::min);
    let max_v = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max_v - min_v).max(1e-12);

    let out_w: u32 = 1024;
    let out_h: u32 = 768;
    let caption = if title.is_empty() {
        format!("{} [{}, {}]", colormap, fmt_g(min_v), fmt_g(max_v))
    } else {
        format!("{} — {} [{}, {}]", title, colormap, fmt_g(min_v), fmt_g(max_v))
    };
    let cmap = colormap.to_string();

    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (out_w, out_h)).into_drawing_area();
        render_imagesc_to_backend(root, nrows, ncols, &vals, min_v, range, &cmap, &caption)
    } else {
        let root = BitMapBackend::new(path, (out_w, out_h)).into_drawing_area();
        render_imagesc_to_backend(root, nrows, ncols, &vals, min_v, range, &cmap, &caption)
    }
}

fn render_imagesc_to_backend<DB>(
    root: DrawingArea<DB, plotters::coord::Shift>,
    nrows: usize,
    ncols: usize,
    vals: &[f64],
    min_v: f64,
    range: f64,
    colormap: &str,
    caption: &str,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync + 'static,
{
    let err = |e: DrawingAreaErrorKind<DB::ErrorType>| PlotError::FileOutput(e.to_string());
    root.fill(&WHITE).map_err(err)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 16u32).into_font())
        .margin(MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d(0.0..(ncols as f64), 0.0..(nrows as f64))
        .map_err(err)?;

    chart.configure_mesh().disable_mesh().draw().map_err(err)?;

    for r in 0..nrows {
        for c in 0..ncols {
            let v = vals[r * ncols + c];
            let t = (v - min_v) / range;
            let (rr, gg, bb) = colormap_rgb(t, colormap);
            let color = RGBColor(rr, gg, bb);
            let x0 = c as f64;
            let y0 = (nrows - 1 - r) as f64; // flip y so row 0 is at top
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x0, y0), (x0 + 1.0, y0 + 1.0)],
                color.filled(),
            ))).map_err(err)?;
        }
    }

    root.present().map_err(err)?;
    Ok(())
}

// ─── Backward-compat wrappers ───────────────────────────────────────────────

fn prepare_figure_for_save(title: &str, xlabel: &str, ylabel: &str) {
    FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        fig.current_mut().series.clear();
        let sp = fig.current_mut();
        if !title.is_empty() { sp.title = title.to_string(); }
        if sp.xlabel.is_empty() { sp.xlabel = xlabel.to_string(); }
        if sp.ylabel.is_empty() { sp.ylabel = ylabel.to_string(); }
    });
}

/// Push a line series and render to file.
pub fn save_plot(data: &RVector, title: &str, path: &str) -> Result<(), PlotError> {
    if data.is_empty() { return Err(PlotError::EmptyData); }
    prepare_figure_for_save(title, "Sample", "Value");
    FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        let color = fig.next_color();
        let sp = fig.current_mut();
        sp.series.push(crate::figure::Series {
            label: "value".to_string(),
            x_data: (0..data.len()).map(|i| i as f64).collect(),
            y_data: data.iter().copied().collect(),
            color,
            style: LineStyle::Solid,
            kind: PlotKind::Line,
        });
    });
    render_figure_file(path)
}

/// Push a stem series and render to file.
pub fn save_stem(data: &RVector, title: &str, path: &str) -> Result<(), PlotError> {
    if data.is_empty() { return Err(PlotError::EmptyData); }
    prepare_figure_for_save(title, "Sample", "Value");
    FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        let color = fig.next_color();
        let sp = fig.current_mut();
        sp.series.push(crate::figure::Series {
            label: "stem".to_string(),
            x_data: (0..data.len()).map(|i| i as f64).collect(),
            y_data: data.iter().copied().collect(),
            color,
            style: LineStyle::Solid,
            kind: PlotKind::Stem,
        });
    });
    render_figure_file(path)
}

/// Push dB series and render to file.
pub fn save_db(freqs: &RVector, h: &CVector, title: &str, path: &str) -> Result<(), PlotError> {
    let n = freqs.len().min(h.len());
    if n == 0 { return Err(PlotError::EmptyData); }
    const FLOOR_DB: f64 = -120.0;
    let x: Vec<f64> = freqs.iter().take(n).copied().collect();
    let y: Vec<f64> = h.iter().take(n).map(|c| {
        let m = c.norm();
        if m < 1e-12 { FLOOR_DB } else { 20.0 * m.log10() }
    }).collect();
    FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        fig.current_mut().series.clear();
        let color = fig.next_color();
        let sp = fig.current_mut();
        if !title.is_empty() { sp.title = title.to_string(); }
        sp.xlabel = "Frequency (Hz)".to_string();
        sp.ylabel = "Magnitude (dB)".to_string();
        sp.series.push(crate::figure::Series {
            label: "dB".to_string(), x_data: x, y_data: y,
            color, style: LineStyle::Solid, kind: PlotKind::Line,
        });
    });
    render_figure_file(path)
}

/// Push histogram series and render to file.
pub fn save_histogram(data: &RVector, n_bins: usize, title: &str, path: &str) -> Result<(), PlotError> {
    if data.is_empty() || n_bins == 0 { return Err(PlotError::EmptyData); }
    let (centers, counts, bin_width) = compute_histogram(data, n_bins);
    if centers.is_empty() { return Err(PlotError::EmptyData); }

    let x_min = centers[0]          - bin_width / 2.0;
    let x_max = centers[n_bins - 1] + bin_width / 2.0;
    let y_max = counts.iter().copied().fold(0.0f64, f64::max);
    let y_hi  = y_max * 1.1 + 1.0;

    FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        fig.current_mut().series.clear();
        let sp = fig.current_mut();
        if !title.is_empty() { sp.title = title.to_string(); }
        sp.xlabel = "Value".to_string();
        sp.ylabel = "Count".to_string();
        sp.xlim = (Some(x_min), Some(x_max));
        sp.ylim = (Some(0.0), Some(y_hi));
    });

    let mut x: Vec<f64> = Vec::with_capacity(n_bins * 4);
    let mut y: Vec<f64> = Vec::with_capacity(n_bins * 4);
    for i in 0..n_bins {
        let left  = centers[i] - bin_width / 2.0;
        let right = centers[i] + bin_width / 2.0;
        x.extend_from_slice(&[left, left, right, right]);
        y.extend_from_slice(&[0.0, counts[i], counts[i], 0.0]);
    }
    FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        let color = fig.next_color();
        let sp = fig.current_mut();
        sp.series.push(crate::figure::Series {
            label: "count".to_string(), x_data: x, y_data: y,
            color, style: LineStyle::Solid, kind: PlotKind::Line,
        });
    });
    render_figure_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array1;
    use num_complex::Complex;

    fn tmp_path(suffix: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("rustlab_plot_test_{}{}", std::process::id(), suffix));
        p.to_str().unwrap().to_string()
    }

    fn real_data(n: usize) -> RVector {
        Array1::from_iter((0..n).map(|i| (i as f64).sin()))
    }

    fn cvec_data(n: usize) -> CVector {
        Array1::from_iter((0..n).map(|i| Complex::new((i as f64).cos(), 0.0)))
    }

    #[test]
    fn savefig_svg_produces_nonempty_file() {
        let path = tmp_path("_line.svg");
        let data = real_data(64);
        save_plot(&data, "Test Line", &path).expect("save_plot should succeed");
        let meta = std::fs::metadata(&path).expect("SVG file should exist after save_plot");
        assert!(meta.len() > 500, "SVG file should be non-trivial (>500 bytes), got {}", meta.len());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_stem_svg_nonempty() {
        let path = tmp_path("_stem.svg");
        let data = real_data(32);
        save_stem(&data, "Test Stem", &path).expect("save_stem should succeed");
        let meta = std::fs::metadata(&path).expect("stem SVG should exist");
        assert!(meta.len() > 500, "stem SVG should be non-trivial (>500 bytes), got {}", meta.len());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_db_svg_contains_svg_tag() {
        let path = tmp_path("_db.svg");
        let n = 64usize;
        let freqs: RVector = Array1::from_iter((0..n).map(|i| i as f64 * 100.0));
        let h = cvec_data(n);
        save_db(&freqs, &h, "Test dB", &path).expect("save_db should succeed");
        let content = std::fs::read_to_string(&path).expect("should be able to read SVG");
        assert!(content.contains("<svg"), "SVG file should contain '<svg' tag");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_plot_empty_data_errors() {
        let path = tmp_path("_empty.svg");
        let empty: RVector = Array1::from_vec(vec![]);
        let result = save_plot(&empty, "Empty", &path);
        assert!(result.is_err(), "save_plot with empty data should return an error");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_histogram_svg_nonempty() {
        let path = tmp_path("_hist.svg");
        let data: RVector = Array1::from_iter((0..100).map(|i| (i % 10) as f64));
        save_histogram(&data, 10, "Test Histogram", &path).expect("save_histogram should succeed");
        let meta = std::fs::metadata(&path).expect("histogram SVG should exist");
        assert!(meta.len() > 500, "histogram SVG should be non-trivial, got {}", meta.len());
        let _ = std::fs::remove_file(&path);
    }
}
