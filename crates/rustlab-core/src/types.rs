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

#[cfg(test)]
mod tests {
    use super::*;

    // ── RoundMode ───────────────────────────────────────────────────────────

    #[test]
    fn round_mode_from_str_all_variants() {
        assert_eq!(RoundMode::from_str("floor"),     Some(RoundMode::Floor));
        assert_eq!(RoundMode::from_str("ceil"),      Some(RoundMode::Ceil));
        assert_eq!(RoundMode::from_str("zero"),      Some(RoundMode::Zero));
        assert_eq!(RoundMode::from_str("round"),     Some(RoundMode::Round));
        assert_eq!(RoundMode::from_str("round_even"), Some(RoundMode::RoundEven));
    }

    #[test]
    fn round_mode_aliases() {
        assert_eq!(RoundMode::from_str("truncate"),   Some(RoundMode::Floor));
        assert_eq!(RoundMode::from_str("trunc"),      Some(RoundMode::Floor));
        assert_eq!(RoundMode::from_str("even"),       Some(RoundMode::RoundEven));
        assert_eq!(RoundMode::from_str("convergent"), Some(RoundMode::RoundEven));
    }

    #[test]
    fn round_mode_case_insensitive() {
        assert_eq!(RoundMode::from_str("FLOOR"), Some(RoundMode::Floor));
        assert_eq!(RoundMode::from_str("Round_Even"), Some(RoundMode::RoundEven));
    }

    #[test]
    fn round_mode_hyphen_alias() {
        assert_eq!(RoundMode::from_str("round-even"), Some(RoundMode::RoundEven));
    }

    #[test]
    fn round_mode_unknown_returns_none() {
        assert_eq!(RoundMode::from_str("banana"), None);
        assert_eq!(RoundMode::from_str(""), None);
    }

    #[test]
    fn round_mode_round_trip() {
        for mode in [RoundMode::Floor, RoundMode::Ceil, RoundMode::Zero,
                     RoundMode::Round, RoundMode::RoundEven] {
            assert_eq!(RoundMode::from_str(mode.as_str()), Some(mode));
        }
    }

    // ── OverflowMode ────────────────────────────────────────────────────────

    #[test]
    fn overflow_mode_from_str_all_variants() {
        assert_eq!(OverflowMode::from_str("saturate"), Some(OverflowMode::Saturate));
        assert_eq!(OverflowMode::from_str("wrap"),     Some(OverflowMode::Wrap));
    }

    #[test]
    fn overflow_mode_aliases() {
        assert_eq!(OverflowMode::from_str("sat"), Some(OverflowMode::Saturate));
    }

    #[test]
    fn overflow_mode_case_insensitive() {
        assert_eq!(OverflowMode::from_str("SATURATE"), Some(OverflowMode::Saturate));
        assert_eq!(OverflowMode::from_str("Wrap"), Some(OverflowMode::Wrap));
    }

    #[test]
    fn overflow_mode_unknown_returns_none() {
        assert_eq!(OverflowMode::from_str("clamp"), None);
        assert_eq!(OverflowMode::from_str(""), None);
    }

    #[test]
    fn overflow_mode_round_trip() {
        for mode in [OverflowMode::Saturate, OverflowMode::Wrap] {
            assert_eq!(OverflowMode::from_str(mode.as_str()), Some(mode));
        }
    }
}
