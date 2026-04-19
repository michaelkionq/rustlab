use crate::error::PlotError;
use crate::figure::{
    colormap_rgb, plot_context, push_notebook_figure_snapshot, FigureState, LineStyle, PlotContext,
    PlotKind, SubplotState, SurfaceData, FIGURE,
};
use plotters::prelude::*;

const MARGIN: u32 = 20;
const X_LABEL_AREA: u32 = 50;
const Y_LABEL_AREA: u32 = 70;

/// Format a float compactly (no %g in Rust).
fn fmt_g(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
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
    if plot_context() == PlotContext::Notebook {
        push_notebook_figure_snapshot();
    }
    if path.ends_with(".html") || path.ends_with(".htm") {
        return crate::html::render_figure_html(path);
    }
    FIGURE.with(|fig| {
        let fig = fig.borrow();
        render_figure_state_to_file(&fig, path)
    })
}

/// Render a given FigureState to a file (PNG or SVG by extension).
pub fn render_figure_state_to_file(fig: &FigureState, path: &str) -> Result<(), PlotError> {
    let rows = fig.subplot_rows;
    let cols = fig.subplot_cols;
    let w = (cols as u32 * 900).min(3600);
    let h = (rows as u32 * 500).min(3000);

    if path.ends_with(".svg") {
        let root = SVGBackend::new(path, (w, h)).into_drawing_area();
        render_to_backend(root, fig, rows, cols)
    } else {
        let root = BitMapBackend::new(path, (w, h)).into_drawing_area();
        render_to_backend(root, fig, rows, cols)
    }
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
        if idx >= fig.subplots.len() {
            break;
        }
        let sp = &fig.subplots[idx];
        // Surface rendering takes precedence over heatmap/series
        if let Some(sf) = &sp.surface {
            let caption = if sp.title.is_empty() {
                format!("surf — {}", sf.colorscale)
            } else {
                format!("{} — surf {}", sp.title, sf.colorscale)
            };
            render_surface_to_backend(panel.clone(), sf, &caption)?;
            continue;
        }
        // Heatmap rendering takes precedence
        if let Some(hm) = &sp.heatmap {
            let nrows = hm.z.len();
            let ncols = if nrows > 0 { hm.z[0].len() } else { 0 };
            if nrows > 0 && ncols > 0 {
                let vals: Vec<f64> = hm.z.iter().flat_map(|row| row.iter().copied()).collect();
                let min_v = vals.iter().copied().fold(f64::INFINITY, f64::min);
                let max_v = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let range = (max_v - min_v).max(1e-12);
                let caption = if sp.title.is_empty() {
                    format!("{} [{}, {}]", hm.colorscale, fmt_g(min_v), fmt_g(max_v))
                } else {
                    format!(
                        "{} — {} [{}, {}]",
                        sp.title,
                        hm.colorscale,
                        fmt_g(min_v),
                        fmt_g(max_v)
                    )
                };
                render_imagesc_to_backend(
                    panel.clone(),
                    nrows,
                    ncols,
                    &vals,
                    min_v,
                    range,
                    &hm.colorscale,
                    &caption,
                )?;
            }
            continue;
        }
        if sp.series.is_empty() {
            continue;
        }
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
    let all_x: Vec<f64> = sp
        .series
        .iter()
        .flat_map(|s| s.x_data.iter().copied())
        .collect();
    let all_y: Vec<f64> = sp
        .series
        .iter()
        .flat_map(|s| s.y_data.iter().copied())
        .collect();
    if all_x.is_empty() || all_y.is_empty() {
        return Ok(());
    }

    let x_min = sp
        .xlim
        .0
        .unwrap_or_else(|| all_x.iter().copied().fold(f64::INFINITY, f64::min));
    let x_max = sp
        .xlim
        .1
        .unwrap_or_else(|| all_x.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    let y_min_raw = all_y.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max_raw = all_y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_margin = ((y_max_raw - y_min_raw).abs() * 0.1).max(1e-6);
    let y_min = sp.ylim.0.unwrap_or(y_min_raw - y_margin);
    let y_max = sp.ylim.1.unwrap_or(y_max_raw + y_margin);

    // Ensure non-degenerate range
    let x_lo = if (x_max - x_min).abs() < 1e-12 {
        x_min - 1.0
    } else {
        x_min
    };
    let x_hi = if (x_max - x_min).abs() < 1e-12 {
        x_max + 1.0
    } else {
        x_max
    };
    let y_lo = if (y_max - y_min).abs() < 1e-12 {
        y_min - 1.0
    } else {
        y_min
    };
    let y_hi = if (y_max - y_min).abs() < 1e-12 {
        y_max + 1.0
    } else {
        y_max
    };

    let title_str = sp.title.as_str();
    let xlabel = if !sp.xlabel.is_empty() {
        sp.xlabel.as_str()
    } else {
        "x"
    };
    let ylabel = if !sp.ylabel.is_empty() {
        sp.ylabel.as_str()
    } else {
        "y"
    };

    let mut chart = ChartBuilder::on(panel)
        .caption(title_str, ("sans-serif", 22u32).into_font())
        .margin(MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d(x_lo..x_hi, y_lo..y_hi)
        .map_err(err)?;

    if let Some(labels) = &sp.x_labels {
        let labels_c = labels.clone();
        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .x_labels(labels_c.len())
            .x_label_formatter(&|v| {
                let rounded = v.round();
                if (*v - rounded).abs() > 1e-6 {
                    return String::new();
                }
                let idx = (rounded as isize) - 1;
                if idx >= 0 && (idx as usize) < labels_c.len() {
                    labels_c[idx as usize].clone()
                } else {
                    String::new()
                }
            })
            .draw()
            .map_err(err)?;
    } else {
        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc(xlabel)
            .y_desc(ylabel)
            .draw()
            .map_err(err)?;
    }

    if sp.grid {
        const N: usize = 5;
        let grid_color = plotters::style::RGBAColor(100, 100, 100, 0.35);
        for i in 0..=N {
            let yv = y_lo + (y_hi - y_lo) * i as f64 / N as f64;
            chart
                .draw_series(LineSeries::new(
                    vec![(x_lo, yv), (x_hi, yv)],
                    grid_color.stroke_width(1),
                ))
                .map_err(err)?;
        }
        for i in 1..N {
            let xv = x_lo + (x_hi - x_lo) * i as f64 / N as f64;
            chart
                .draw_series(LineSeries::new(
                    vec![(xv, y_lo), (xv, y_hi)],
                    grid_color.stroke_width(1),
                ))
                .map_err(err)?;
        }
    }

    // Pre-compute grouped bar offsets
    let bar_series_count = sp.series.iter().filter(|s| s.kind == PlotKind::Bar).count();
    let mut bar_series_idx = 0usize;

    // Draw each series
    for s in &sp.series {
        let rgb = s.color.to_plotters();
        let stroke_width: u32 = if s.style == LineStyle::Dashed { 1 } else { 2 };
        let color = rgb.stroke_width(stroke_width);

        match s.kind {
            PlotKind::Line => {
                let pts: Vec<(f64, f64)> = s
                    .x_data
                    .iter()
                    .copied()
                    .zip(s.y_data.iter().copied())
                    .collect();

                if s.style == LineStyle::Dashed {
                    // Simulate dashed by drawing every other segment
                    let mut draw_seg = true;
                    for pair in pts.windows(2) {
                        if draw_seg {
                            chart
                                .draw_series(LineSeries::new(vec![pair[0], pair[1]], color))
                                .map_err(err)?;
                        }
                        draw_seg = !draw_seg;
                    }
                } else {
                    chart
                        .draw_series(LineSeries::new(pts, color))
                        .map_err(err)?;
                }
            }
            PlotKind::Stem => {
                // Baseline
                let x_lo_s = s.x_data.iter().copied().fold(f64::INFINITY, f64::min);
                let x_hi_s = s.x_data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                chart
                    .draw_series(LineSeries::new(
                        vec![(x_lo_s, 0.0), (x_hi_s, 0.0)],
                        BLACK.stroke_width(1),
                    ))
                    .map_err(err)?;

                // Stems
                chart
                    .draw_series(
                        s.x_data
                            .iter()
                            .copied()
                            .zip(s.y_data.iter().copied())
                            .map(|(x, y)| PathElement::new(vec![(x, 0.0), (x, y)], color)),
                    )
                    .map_err(err)?;

                // Tips
                chart
                    .draw_series(
                        s.x_data
                            .iter()
                            .copied()
                            .zip(s.y_data.iter().copied())
                            .map(|(x, y)| Circle::new((x, y), 3, rgb.filled())),
                    )
                    .map_err(err)?;
            }
            PlotKind::Bar => {
                let n = s.x_data.len();
                let group_w = if n > 1 {
                    let span = s.x_data[n - 1] - s.x_data[0];
                    (span / (n - 1) as f64) * 0.8
                } else {
                    0.8
                };
                let (bar_w, offset) = if bar_series_count > 1 {
                    let bw = group_w / bar_series_count as f64;
                    let off = -group_w / 2.0 + bw * bar_series_idx as f64 + bw / 2.0;
                    (bw * 0.9, off)
                } else {
                    (group_w, 0.0)
                };
                bar_series_idx += 1;
                let half = bar_w / 2.0;

                // Baseline
                chart
                    .draw_series(LineSeries::new(
                        vec![(x_lo, 0.0), (x_hi, 0.0)],
                        BLACK.stroke_width(1),
                    ))
                    .map_err(err)?;

                // Filled bars
                chart
                    .draw_series(s.x_data.iter().copied().zip(s.y_data.iter().copied()).map(
                        |(x, y)| {
                            let cx = x + offset;
                            let (y0, y1) = if y >= 0.0 { (0.0, y) } else { (y, 0.0) };
                            Rectangle::new([(cx - half, y0), (cx + half, y1)], rgb.filled())
                        },
                    ))
                    .map_err(err)?;

                // Bar outlines
                chart
                    .draw_series(s.x_data.iter().copied().zip(s.y_data.iter().copied()).map(
                        |(x, y)| {
                            let cx = x + offset;
                            let (y0, y1) = if y >= 0.0 { (0.0, y) } else { (y, 0.0) };
                            Rectangle::new(
                                [(cx - half, y0), (cx + half, y1)],
                                BLACK.stroke_width(1),
                            )
                        },
                    ))
                    .map_err(err)?;
            }
            PlotKind::Scatter => {
                chart
                    .draw_series(
                        s.x_data
                            .iter()
                            .copied()
                            .zip(s.y_data.iter().copied())
                            .map(|(x, y)| Circle::new((x, y), 4, rgb.filled())),
                    )
                    .map_err(err)?;
            }
        }
    }
    Ok(())
}

/// Render a 3D surface to a plotters backend using a fixed isometric camera.
/// Draws colored quads over the grid (painter's algorithm depth sort) plus a
/// simple wireframe + axis box. Matches the HTML/viewer surface output at a
/// reasonable static angle so SVG/PNG exports look like 3D surfaces, not 2D heatmaps.
fn render_surface_to_backend<DB>(
    root: DrawingArea<DB, plotters::coord::Shift>,
    sf: &SurfaceData,
    caption: &str,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync + 'static,
{
    let err = |e: DrawingAreaErrorKind<DB::ErrorType>| PlotError::FileOutput(e.to_string());
    root.fill(&WHITE).map_err(err)?;

    let nrows = sf.z.len();
    let ncols = if nrows > 0 { sf.z[0].len() } else { 0 };
    if nrows < 2 || ncols < 2 {
        return Ok(());
    }

    // Caption
    let (w_pixels, h_pixels) = root.dim_in_pixel();
    let caption_style: TextStyle = ("sans-serif", 18u32).into_font().into();
    root.draw_text(caption, &caption_style, (MARGIN as i32, 4))
        .map_err(err)?;

    // Plot area inside root (leave margins for caption + axes).
    let pad_l = 50i32;
    let pad_r = 20i32;
    let pad_t = 34i32;
    let pad_b = 40i32;
    let plot_w = (w_pixels as i32 - pad_l - pad_r).max(50);
    let plot_h = (h_pixels as i32 - pad_t - pad_b).max(50);

    // Z min/max for normalization.
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;
    for row in &sf.z {
        for &v in row {
            if v < min_z {
                min_z = v;
            }
            if v > max_z {
                max_z = v;
            }
        }
    }
    let z_range = (max_z - min_z).max(1e-12);

    // Data bounds
    let x_min = sf.x.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = sf.x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_min = sf.y.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = sf.y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let x_span = (x_max - x_min).max(1e-12);
    let y_span = (y_max - y_min).max(1e-12);

    // Camera: yaw about z (azimuth) + pitch about x (elevation).
    // yaw = -45°, pitch = 30° gives a standard isometric look.
    let yaw = -45f64.to_radians();
    let pitch = 30f64.to_radians();
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();

    // World-to-camera projection (orthographic) of (x, y, z) in normalized units.
    // Each axis is mapped to [-1, 1] before projection.
    let project = |xi: f64, yi: f64, zi: f64| -> (f64, f64, f64) {
        let nx = 2.0 * (xi - x_min) / x_span - 1.0;
        let ny = 2.0 * (yi - y_min) / y_span - 1.0;
        let nz = 2.0 * (zi - min_z) / z_range - 1.0;
        // Rotate about z (yaw)
        let xr = nx * cy - ny * sy;
        let yr = nx * sy + ny * cy;
        // Rotate about x (pitch)
        let zr = nz * cp - yr * sp;
        let yr2 = nz * sp + yr * cp;
        (xr, yr2, zr) // (screen-x, depth, screen-y)
    };

    // Compute projected-extent to rescale to pixel coords.
    let mut sxmin = f64::INFINITY;
    let mut sxmax = f64::NEG_INFINITY;
    let mut symin = f64::INFINITY;
    let mut symax = f64::NEG_INFINITY;
    for r in 0..nrows {
        for c in 0..ncols {
            let (sx, _d, sz) = project(sf.x[c], sf.y[r], sf.z[r][c]);
            if sx < sxmin {
                sxmin = sx;
            }
            if sx > sxmax {
                sxmax = sx;
            }
            if sz < symin {
                symin = sz;
            }
            if sz > symax {
                symax = sz;
            }
        }
    }
    let sxr = (sxmax - sxmin).max(1e-12);
    let syr = (symax - symin).max(1e-12);
    let scale = (plot_w as f64 / sxr).min(plot_h as f64 / syr) * 0.92;
    let cx = pad_l as f64 + plot_w as f64 * 0.5;
    let cy_px = pad_t as f64 + plot_h as f64 * 0.5;
    let to_px = |sx: f64, sz: f64| -> (i32, i32) {
        let x = cx + (sx - (sxmin + sxmax) * 0.5) * scale;
        // Screen y grows downward; invert sz.
        let y = cy_px - (sz - (symin + symax) * 0.5) * scale;
        (x.round() as i32, y.round() as i32)
    };

    // Draw axes box as a faint wireframe cube: project the 8 corners of the
    // unit bounding box in world space, then connect edges.
    let corners = [
        (x_min, y_min, min_z),
        (x_max, y_min, min_z),
        (x_max, y_max, min_z),
        (x_min, y_max, min_z),
        (x_min, y_min, max_z),
        (x_max, y_min, max_z),
        (x_max, y_max, max_z),
        (x_min, y_max, max_z),
    ];
    let pc: Vec<(i32, i32)> = corners
        .iter()
        .map(|&(x, y, z)| {
            let (sx, _d, sz) = project(x, y, z);
            to_px(sx, sz)
        })
        .collect();
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let axis_color = RGBColor(160, 160, 160);
    for (a, b) in edges {
        root.draw(&PathElement::new(vec![pc[a], pc[b]], axis_color.stroke_width(1)))
            .map_err(err)?;
    }

    // Build quads with their centroid depth for sorting.
    struct Quad {
        depth: f64,
        pts: [(i32, i32); 4],
        color: RGBColor,
    }
    let mut quads: Vec<Quad> = Vec::with_capacity((nrows - 1) * (ncols - 1));
    for r in 0..(nrows - 1) {
        for c in 0..(ncols - 1) {
            let v00 = (sf.x[c], sf.y[r], sf.z[r][c]);
            let v10 = (sf.x[c + 1], sf.y[r], sf.z[r][c + 1]);
            let v11 = (sf.x[c + 1], sf.y[r + 1], sf.z[r + 1][c + 1]);
            let v01 = (sf.x[c], sf.y[r + 1], sf.z[r + 1][c]);
            let p00 = project(v00.0, v00.1, v00.2);
            let p10 = project(v10.0, v10.1, v10.2);
            let p11 = project(v11.0, v11.1, v11.2);
            let p01 = project(v01.0, v01.1, v01.2);
            let depth = (p00.1 + p10.1 + p11.1 + p01.1) * 0.25;
            let zc = (sf.z[r][c] + sf.z[r][c + 1] + sf.z[r + 1][c + 1] + sf.z[r + 1][c]) * 0.25;
            let t = (zc - min_z) / z_range;
            let (rr, gg, bb) = colormap_rgb(t, &sf.colorscale);
            quads.push(Quad {
                depth,
                pts: [
                    to_px(p00.0, p00.2),
                    to_px(p10.0, p10.2),
                    to_px(p11.0, p11.2),
                    to_px(p01.0, p01.2),
                ],
                color: RGBColor(rr, gg, bb),
            });
        }
    }
    // Painter's algorithm: draw far faces first (smallest depth first).
    quads.sort_by(|a, b| {
        a.depth
            .partial_cmp(&b.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let edge_style = RGBColor(80, 80, 80).stroke_width(1);
    for q in &quads {
        root.draw(&Polygon::new(q.pts.to_vec(), q.color.filled()))
            .map_err(err)?;
        let mut ring = q.pts.to_vec();
        ring.push(q.pts[0]);
        root.draw(&PathElement::new(ring, edge_style)).map_err(err)?;
    }

    // Axis tick labels (min/max on X and Y, min/max on Z).
    let tick_font = ("sans-serif", 11u32).into_font();
    let label = |corner: &(i32, i32), s: String| -> Result<(), PlotError> {
        root.draw(&Text::new(s, (corner.0 + 4, corner.1 + 2), tick_font.clone()))
            .map_err(err)?;
        Ok(())
    };
    label(&pc[0], format!("x={:.3}", x_min))?;
    label(&pc[1], format!("x={:.3}", x_max))?;
    label(&pc[3], format!("y={:.3}", y_max))?;
    label(&pc[4], format!("z={:.3}", max_z))?;

    root.present().map_err(err)?;
    Ok(())
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
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x0, y0), (x0 + 1.0, y0 + 1.0)],
                    color.filled(),
                )))
                .map_err(err)?;
        }
    }

    root.present().map_err(err)?;
    Ok(())
}

// NOTE: Legacy save_* wrapper functions were removed — save builtins now use
// the same push helpers as interactive builtins (push_xy_line, push_xy_stem, etc.)
// followed by render_figure_file(). See builtins.rs for the consolidated logic.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{push_xy_bar, push_xy_line, push_xy_scatter, push_xy_stem};

    fn tmp_path(suffix: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "rustlab_plot_test_{}{}",
            std::process::id(),
            suffix
        ));
        p.to_str().unwrap().to_string()
    }

    // Tests use the push helpers + render_figure_file pattern (same as builtins)

    #[test]
    fn push_line_and_render_produces_svg() {
        let path = tmp_path("_line.svg");
        let x: Vec<f64> = (0..64).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi.sin()).collect();
        push_xy_line(x, y, "value", "Test Line", None, LineStyle::Solid);
        render_figure_file(&path).expect("render should succeed");
        let meta = std::fs::metadata(&path).expect("SVG file should exist");
        assert!(
            meta.len() > 500,
            "SVG should be non-trivial (>500 bytes), got {}",
            meta.len()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn push_stem_and_render_produces_svg() {
        let path = tmp_path("_stem.svg");
        let x: Vec<f64> = (0..32).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi.sin()).collect();
        push_xy_stem(x, y, "stem", "Test Stem", None);
        render_figure_file(&path).expect("render should succeed");
        let meta = std::fs::metadata(&path).expect("stem SVG should exist");
        assert!(
            meta.len() > 500,
            "stem SVG should be non-trivial (>500 bytes), got {}",
            meta.len()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn push_bar_and_render_produces_svg() {
        let path = tmp_path("_bar.svg");
        push_xy_bar(
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
            vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0],
            "bar",
            "Test Bar",
            None,
        );
        render_figure_file(&path).expect("render should succeed");
        let meta = std::fs::metadata(&path).expect("bar SVG should exist");
        assert!(
            meta.len() > 500,
            "bar SVG should be non-trivial (>500 bytes), got {}",
            meta.len()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn push_bar_negative_values() {
        let path = tmp_path("_bar_neg.svg");
        push_xy_bar(
            vec![0.0, 1.0, 2.0, 3.0],
            vec![-3.0, 2.0, -1.0, 5.0],
            "bar",
            "Negative Bars",
            None,
        );
        render_figure_file(&path).expect("render should succeed");
        let meta = std::fs::metadata(&path).expect("file should exist");
        assert!(meta.len() > 500);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn push_scatter_and_render_produces_svg() {
        let path = tmp_path("_scatter.svg");
        let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.5).collect();
        let y: Vec<f64> = x.iter().map(|&xi| xi * xi * 0.1).collect();
        push_xy_scatter(x, y, "scatter", "Test Scatter", None);
        render_figure_file(&path).expect("render should succeed");
        let meta = std::fs::metadata(&path).expect("scatter SVG should exist");
        assert!(
            meta.len() > 500,
            "scatter SVG should be non-trivial, got {}",
            meta.len()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn push_scatter_contains_svg_tag() {
        let path = tmp_path("_scatter_tag.svg");
        push_xy_scatter(
            vec![1.0, 2.0, 3.0],
            vec![4.0, 2.0, 5.0],
            "pts",
            "Scatter Tag",
            None,
        );
        render_figure_file(&path).expect("render should succeed");
        let content = std::fs::read_to_string(&path).expect("should read SVG");
        assert!(
            content.contains("<svg"),
            "scatter SVG should contain '<svg' tag"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn categorical_bar_svg_renders_each_label_once() {
        // Regression: bar(labels, y) used to emit each tick label twice because
        // plotters generates ticks at half-integer positions and the formatter
        // mapped (v as usize)-1 to labels[0] for both v=1.0 and v=1.5. The
        // formatter now returns "" for non-integer ticks.
        let path = tmp_path("_cat_bar.svg");
        let labels = vec![
            "|00>".to_string(),
            "|01>".to_string(),
            "|10>".to_string(),
            "|11>".to_string(),
        ];
        FIGURE.with(|fig| {
            let mut fig = fig.borrow_mut();
            let sp = fig.current_mut();
            sp.series.clear();
            sp.title.clear();
            sp.x_labels = Some(labels.clone());
        });
        push_xy_bar(
            vec![1.0, 2.0, 3.0, 4.0],
            vec![0.25, 0.12, 0.48, 0.15],
            "bar",
            "Categorical Bar",
            None,
        );
        render_figure_file(&path).expect("render should succeed");
        let content = std::fs::read_to_string(&path).expect("should read SVG");
        // SVG escapes '>' as '&gt;'; check the escaped form.
        for lbl in ["|00&gt;", "|01&gt;", "|10&gt;", "|11&gt;"] {
            let count = content.matches(lbl).count();
            assert_eq!(
                count, 1,
                "expected {} to appear once in SVG, found {}",
                lbl, count
            );
        }
        // Reset state so sibling tests aren't affected.
        FIGURE.with(|fig| fig.borrow_mut().current_mut().x_labels = None);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn heatmap_in_figure_renders_to_svg() {
        let path = tmp_path("_heatmap.svg");
        FIGURE.with(|fig| {
            let mut fig = fig.borrow_mut();
            let sp = fig.current_mut();
            sp.series.clear();
            sp.title = "Heatmap Test".to_string();
            sp.heatmap = Some(crate::figure::HeatmapData {
                z: vec![
                    vec![0.0, 0.5, 1.0],
                    vec![0.3, 0.7, 0.2],
                    vec![1.0, 0.1, 0.5],
                ],
                colorscale: "viridis".to_string(),
            });
        });
        render_figure_file(&path).expect("heatmap render should succeed");
        let content = std::fs::read_to_string(&path).expect("should read SVG");
        assert!(
            content.contains("<svg"),
            "heatmap SVG should contain '<svg' tag"
        );
        let _ = std::fs::remove_file(&path);
    }
}
