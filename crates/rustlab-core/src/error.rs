use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("singular matrix")]
    SingularMatrix,
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}
