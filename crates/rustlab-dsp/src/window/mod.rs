//! Window functions for DSP applications.
//!
//! In digital signal processing a *window function* is multiplied element-wise
//! with a finite-length sequence before computing its Discrete Fourier Transform.
//! Without windowing, the abrupt truncation of an infinite signal at the block
//! boundaries introduces *spectral leakage* — energy from one frequency bin
//! spills into neighbouring bins through high *sidelobes*.
//!
//! Different windows trade off:
//! - **Frequency resolution** (width of the main lobe): a wider main lobe means
//!   two nearby tones are harder to resolve.
//! - **Sidelobe rejection** (how much leakage is suppressed): higher rejection
//!   reduces the floor at which a small signal can be detected next to a large one.
//!
//! # Choosing a window
//!
//! | Window      | Main-lobe width | Peak sidelobe |
//! |-------------|----------------|--------------|
//! | Rectangular | Narrowest      | ~−13 dB      |
//! | Hann        | 2× rect        | ~−31 dB      |
//! | Hamming     | 2× rect        | ~−41 dB      |
//! | Blackman    | 3× rect        | ~−57 dB      |
//! | Kaiser(β)   | Parametric     | Parametric   |

use std::f64::consts::PI;
use rustlab_core::RVector;
use ndarray::Array1;
use crate::error::DspError;

/// Available window functions for FIR filter design and spectral analysis.
///
/// Use [`WindowFunction::generate`] to obtain the coefficient array, or
/// [`WindowFunction::from_str`] to construct a variant by name.
#[derive(Debug, Clone)]
pub enum WindowFunction {
    /// Rectangular (boxcar) window — equivalent to no windowing at all.
    ///
    /// Provides the **narrowest main lobe** (best frequency resolution) but
    /// has the **highest sidelobe leakage** (~−13 dB peak sidelobe level).
    /// Suitable only when all spectral components have similar amplitudes or
    /// when the signal exactly fits in the analysis block.
    Rectangular,

    /// Hann (raised-cosine) window — a good general-purpose choice.
    ///
    /// Achieves ~−31 dB peak sidelobe rejection with an 18 dB/octave roll-off
    /// rate. The main lobe is roughly twice as wide as the rectangular window.
    /// Often the default choice when sidelobe leakage matters but high
    /// resolution is still needed.
    Hann,

    /// Hamming window — optimised to minimise the nearest sidelobe.
    ///
    /// The non-zero endpoints push the first sidelobe down to ~−41 dB, slightly
    /// better than Hann near the main lobe. However, the far sidelobes decay
    /// more slowly (~6 dB/octave) compared to Hann's 18 dB/octave roll-off.
    /// Preferred when the first sidelobe level is the primary concern.
    Hamming,

    /// Blackman window — very low sidelobes at the cost of a wider main lobe.
    ///
    /// Uses a three-term cosine series to achieve ~−57 dB peak sidelobe level
    /// and ~18 dB/octave roll-off. The main lobe is approximately three times
    /// the width of a rectangular window. Use when high dynamic range is more
    /// important than frequency resolution.
    Blackman,

    /// Kaiser (parametric) window — controlled sidelobe/main-lobe trade-off.
    ///
    /// The shape parameter `beta` (β) lets you dial in any point on the
    /// sidelobe-vs-resolution curve:
    /// - β = 0: equivalent to a rectangular window
    /// - β = 5: approximately Hamming-level sidelobes
    /// - β = 8.6: ~−60 dB peak sidelobe level (common default)
    /// - Higher β: lower sidelobes, wider main lobe
    ///
    /// Kaiser windows are widely used in optimal FIR filter design because the
    /// required β and tap count can be estimated directly from the desired
    /// stopband attenuation and transition bandwidth.
    Kaiser { beta: f64 },
}

impl WindowFunction {
    /// Generate an array of `length` window coefficients.
    ///
    /// Returns a real-valued [`RVector`] (1-D ndarray) where each element is the
    /// window weight for the corresponding sample index.
    ///
    /// Special cases:
    /// - `length == 0` returns an empty array.
    /// - `length == 1` returns `[1.0]` regardless of window type.
    pub fn generate(&self, length: usize) -> RVector {
        if length == 0 {
            return Array1::zeros(0);
        }
        if length == 1 {
            return Array1::ones(1);
        }

        let n = length;
        let m = (n - 1) as f64; // N-1

        match self {
            WindowFunction::Rectangular => Array1::ones(n),

            WindowFunction::Hann => Array1::from_iter((0..n).map(|i| {
                0.5 * (1.0 - (2.0 * PI * i as f64 / m).cos())
            })),

            WindowFunction::Hamming => Array1::from_iter((0..n).map(|i| {
                0.54 - 0.46 * (2.0 * PI * i as f64 / m).cos()
            })),

            WindowFunction::Blackman => Array1::from_iter((0..n).map(|i| {
                let x = 2.0 * PI * i as f64 / m;
                0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos()
            })),

            WindowFunction::Kaiser { beta } => {
                let m_f = m; // M = length - 1
                let half_m = m_f / 2.0;
                let i0_beta = bessel_i0(*beta);
                Array1::from_iter((0..n).map(|i| {
                    let ratio = (i as f64 - half_m) / half_m;
                    let inner = beta * (1.0 - ratio * ratio).max(0.0).sqrt();
                    bessel_i0(inner) / i0_beta
                }))
            }
        }
    }

    /// Construct a [`WindowFunction`] from a human-readable name string.
    ///
    /// Matching is case-insensitive. Accepted aliases:
    ///
    /// | Variant       | Accepted names                        |
    /// |---------------|---------------------------------------|
    /// | `Rectangular` | `"rectangular"`, `"rect"`, `"boxcar"` |
    /// | `Hann`        | `"hann"`, `"hanning"`                 |
    /// | `Hamming`     | `"hamming"`                           |
    /// | `Blackman`    | `"blackman"`                          |
    /// | `Kaiser`      | `"kaiser"`                            |
    ///
    /// For the `Kaiser` variant, `beta` sets the shape parameter. If `beta` is
    /// `None`, the default value of 8.6 is used (~−60 dB sidelobes).
    ///
    /// Returns [`DspError::UnknownWindow`] if the name is not recognised.
    pub fn from_str(s: &str, beta: Option<f64>) -> Result<WindowFunction, DspError> {
        match s.to_ascii_lowercase().as_str() {
            "rectangular" | "rect" | "boxcar" => Ok(WindowFunction::Rectangular),
            "hann" | "hanning" => Ok(WindowFunction::Hann),
            "hamming" => Ok(WindowFunction::Hamming),
            "blackman" => Ok(WindowFunction::Blackman),
            "kaiser" => {
                let b = beta.unwrap_or(8.6);
                Ok(WindowFunction::Kaiser { beta: b })
            }
            other => Err(DspError::UnknownWindow(other.to_string())),
        }
    }
}

/// Modified Bessel function of the first kind, order 0.
/// Computed via series expansion: I0(x) = sum_{k=0}^{inf} ((x/2)^k / k!)^2
pub(crate) fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0_f64;
    let mut term = 1.0_f64;
    let half_x = x / 2.0;
    for k in 1..=100 {
        term *= half_x / k as f64;
        let contribution = term * term;
        sum += contribution;
        if contribution < 1e-10 {
            break;
        }
    }
    sum
}
