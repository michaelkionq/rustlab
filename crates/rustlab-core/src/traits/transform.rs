use crate::{error::CoreError, types::CVector};

/// A reversible signal transform (e.g. FFT, DCT).
pub trait Transform {
    fn forward(&self, input: &CVector) -> Result<CVector, CoreError>;
    fn inverse(&self, input: &CVector) -> Result<CVector, CoreError>;
}
