use ndarray::{Array1, Array2};
use num_complex::Complex;

/// Complex 64-bit float (the native scalar type throughout rustlab)
pub type C64     = Complex<f64>;
/// Complex column vector
pub type CVector = Array1<C64>;
/// Complex matrix
pub type CMatrix = Array2<C64>;
/// Real vector
pub type RVector = Array1<f64>;
/// Real matrix
pub type RMatrix = Array2<f64>;

/// Fixed-point rounding mode.
#[derive(Debug, Clone, PartialEq)]
pub enum RoundMode {
    /// Truncate toward −∞ — free in hardware (default).
    Floor,
    /// Toward +∞.
    Ceil,
    /// Truncate toward zero (symmetric floor).
    Zero,
    /// Round half away from zero.
    Round,
    /// Round half to even (convergent / banker's rounding).
    RoundEven,
}

impl RoundMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "floor" | "truncate" | "trunc" => Some(Self::Floor),
            "ceil"                          => Some(Self::Ceil),
            "zero"                          => Some(Self::Zero),
            "round"                         => Some(Self::Round),
            "round_even" | "even" | "convergent" => Some(Self::RoundEven),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Floor     => "floor",
            Self::Ceil      => "ceil",
            Self::Zero      => "zero",
            Self::Round     => "round",
            Self::RoundEven => "round_even",
        }
    }
}

/// Fixed-point overflow mode.
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowMode {
    /// Clamp to [min, max] (default).
    Saturate,
    /// 2's complement wrap.
    Wrap,
}

impl OverflowMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "saturate" | "sat" => Some(Self::Saturate),
            "wrap"             => Some(Self::Wrap),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Saturate => "saturate",
            Self::Wrap     => "wrap",
        }
    }
}
