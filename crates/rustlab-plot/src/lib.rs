pub mod ascii;
pub mod error;
pub mod figure;
pub mod file;

pub use ascii::{
    imagesc_terminal,
    plot_complex, plot_db, plot_histogram, plot_real, stem_real,
    push_xy_line, push_xy_stem, push_xy_bar, push_xy_scatter,
    render_figure_terminal,
};
pub use error::PlotError;
pub use file::{
    render_figure_file,
    save_db, save_histogram, save_imagesc_cmap, save_plot, save_stem,
    save_bar, save_scatter,
};
pub use figure::{
    colormap_rgb, FigureState, LineStyle, PlotKind, Series, SeriesColor, SubplotState,
    FIGURE,
};

use rustlab_core::RVector;

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
