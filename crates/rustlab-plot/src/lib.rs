pub mod ascii;
pub mod error;
pub mod figure;
pub mod file;
pub mod live;
#[cfg(feature = "viewer")]
pub mod viewer_client;
#[cfg(feature = "viewer")]
pub mod viewer_live;
pub mod html;
pub mod report;

pub use ascii::{
    imagesc_terminal,
    plot_complex, plot_db, plot_histogram, plot_real, stem_real,
    push_xy_line, push_xy_stem, push_xy_bar, push_xy_scatter,
    render_figure_terminal,
};
pub use error::PlotError;
pub use live::LiveFigure;
#[cfg(feature = "viewer")]
pub use viewer_live::ViewerFigure;
#[cfg(feature = "viewer")]
pub use viewer_live::{connect_viewer, disconnect_viewer, viewer_active, viewer_new_figure, sync_viewer};
pub use file::{
    render_figure_file, render_figure_state_to_file,
    save_db, save_histogram, save_imagesc_cmap, save_plot, save_stem,
    save_bar, save_scatter,
};
pub use figure::{
    colormap_rgb, FigureState, FigureOutput, LineStyle, PlotContext, PlotKind, Series, SeriesColor, SubplotState,
    FIGURE,
    figure_new, figure_new_html, figure_switch,
    current_figure_id, current_figure_output, set_current_figure_output,
    plot_context, set_plot_context,
};
pub use html::{render_figure_html, render_figure_plotly_div, set_html_figure_path, clear_html_figure_path, sync_html_file};
pub use report::{report_start, report_active, report_add, report_auto_capture, report_save, report_end, report_len};

use rustlab_core::RVector;

/// Sync the current figure to its non-terminal output (HTML file or viewer).
/// Called after FIGURE state mutations that don't go through render_figure_terminal().
pub fn sync_figure_outputs() {
    match current_figure_output() {
        FigureOutput::Html(_) => sync_html_file(),
        #[cfg(feature = "viewer")]
        FigureOutput::Viewer(_) => sync_viewer(),
        FigureOutput::Terminal => {}
    }
}

/// Backend-agnostic interface for live-updating plots.
///
/// Implemented by `LiveFigure` (ratatui terminal) and, when the `viewer`
/// feature is enabled, by `ViewerFigure` (egui via IPC).
pub trait LivePlot: Send + std::fmt::Debug {
    fn update_panel(&mut self, idx: usize, x: Vec<f64>, y: Vec<f64>);
    fn set_panel_labels(&mut self, idx: usize, title: &str, xlabel: &str, ylabel: &str);
    fn set_panel_limits(&mut self, idx: usize, xlim: (Option<f64>, Option<f64>), ylim: (Option<f64>, Option<f64>));
    fn redraw(&mut self) -> Result<(), PlotError>;
}

/// Compute histogram bin centers and counts.
/// Returns `(centers, counts, bin_width)`.
/// The last bin is closed on the right so the maximum value falls in it.
pub fn compute_histogram(data: &RVector, n_bins: usize) -> (Vec<f64>, Vec<f64>, f64) {
    if data.is_empty() || n_bins == 0 {
        return (vec![], vec![], 0.0);
    }
    let min = data.iter().copied().fold(f64::INFINITY,     f64::min);
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    let bin_width = if range < 1e-300 { 1.0 } else { range / n_bins as f64 };
    let mut counts = vec![0.0f64; n_bins];
    for &x in data.iter() {
        let idx = ((x - min) / bin_width) as usize;
        counts[idx.min(n_bins - 1)] += 1.0;
    }
    let centers: Vec<f64> = (0..n_bins)
        .map(|i| min + (i as f64 + 0.5) * bin_width)
        .collect();
    (centers, counts, bin_width)
}

/// Build a 2-row ndarray matrix from histogram output: row 0 = centers, row 1 = counts.
pub fn histogram_matrix(centers: &[f64], counts: &[f64]) -> rustlab_core::CMatrix {
    use ndarray::Array2;
    use num_complex::Complex;
    let n = centers.len();
    let mut m = Array2::zeros((2, n));
    for i in 0..n {
        m[(0, i)] = Complex::new(centers[i], 0.0);
        m[(1, i)] = Complex::new(counts[i],  0.0);
    }
    m
}
