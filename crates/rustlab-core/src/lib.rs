pub mod error;
pub mod types;
pub mod traits;

pub use error::CoreError;
pub use types::{C64, CMatrix, CVector, RMatrix, RVector, RoundMode, OverflowMode, SparseVec, SparseMat};
pub use traits::{
    decompose::{
        CholeskyDecomposable, Decomposable, EigenDecomposable, LuDecomposable, SvdDecomposable,
    },
    filter::Filter,
    transform::Transform,
};
