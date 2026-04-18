//! Stub traits for matrix decompositions.
//!
//! These traits define the interfaces for common numerical linear-algebra
//! decompositions. No implementors exist yet — concrete implementations are
//! planned for a future `rustlab-linalg` crate, gated behind a `linalg`
//! feature flag.

use crate::error::CoreError;

/// Generic matrix decomposition. `Output` is the struct returned by the decomposition.
/// No implementors exist yet — add them in a future `rustlab-linalg` crate behind the
/// `linalg` feature flag.
pub trait Decomposable {
    type Output;
    fn decompose(&self) -> Result<Self::Output, CoreError>;
}

// Marker traits — future crates implement these on CMatrix

/// Marker trait for types that support LU decomposition.
pub trait LuDecomposable: Decomposable {}

/// Marker trait for types that support Cholesky decomposition.
pub trait CholeskyDecomposable: Decomposable {}

/// Marker trait for types that support Singular Value Decomposition (SVD).
pub trait SvdDecomposable: Decomposable {}

/// Marker trait for types that support eigenvalue decomposition.
pub trait EigenDecomposable: Decomposable {}
