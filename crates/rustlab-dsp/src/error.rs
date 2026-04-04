use thiserror::Error;

#[derive(Debug, Error)]
pub enum DspError {
    #[error(transparent)]
    Core(#[from] rustlab_core::CoreError),
    #[error("invalid filter order: {0}")]
    InvalidOrder(usize),
    #[error("cutoff frequency {cutoff} must be in (0, sample_rate/2) = (0, {nyquist})")]
    InvalidCutoff { cutoff: f64, nyquist: f64 },
    #[error("unknown window: {0}")]
    UnknownWindow(String),
    #[error("invalid Kaiser spec: {0}")]
    InvalidKaiserSpec(String),
    #[error("invalid Parks-McClellan spec: {0}")]
    InvalidPmSpec(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("invalid Q-format spec: {0}")]
    InvalidQFmt(String),
}
