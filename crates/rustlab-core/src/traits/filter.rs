use crate::{error::CoreError, types::CVector};

/// A digital filter that can be applied to a complex signal.
///
/// Implementors include `FirFilter` and `IirFilter` from `rustlab-dsp`. Both FIR and IIR
/// variants operate on [`CVector`] (complex-valued ndarray arrays) so that the
/// same interface serves real signals (zero imaginary part) and analytic/complex
/// signals alike.
pub trait Filter {
    /// Apply the filter to an input signal, returning the filtered output.
    ///
    /// The output length depends on the implementation: FIR filters produce a
    /// full linear convolution (`input.len() + taps - 1` samples), while IIR
    /// filters return the same number of samples as `input`.
    fn apply(&self, input: &CVector) -> Result<CVector, CoreError>;

    /// Compute the complex frequency response H(e^{jω}) at `n_points`
    /// evenly-spaced normalized frequencies in [0, 0.5).
    ///
    /// Normalized frequency 0.5 corresponds to the Nyquist frequency (half the
    /// sample rate). The returned vector has length `n_points`; each element is
    /// the complex gain of the filter at that frequency.
    fn frequency_response(&self, n_points: usize) -> Result<CVector, CoreError>;
}
