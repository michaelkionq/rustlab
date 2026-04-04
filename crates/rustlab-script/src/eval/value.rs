use std::collections::HashMap;
use std::fmt;
use ndarray::{Array1, Array2};
use num_complex::Complex;
use rustlab_core::{C64, CMatrix, CVector};
use rustlab_dsp::fixed::QFmtSpec;
use crate::ast::BinOp;

#[derive(Debug, Clone)]
pub enum Value {
    Scalar(f64),
    Complex(C64),
    Vector(CVector),
    Matrix(CMatrix),
    Bool(bool),
    Str(String),
    QFmt(QFmtSpec),
    /// Key-value struct: `s.x`, created with `struct("x", 1, "y", 2)` or `s.x = 1`
    Struct(HashMap<String, Value>),
    /// Multiple return values from builtins; consumed by `[a, b] = f()` assignment.
    /// Should never appear as a standalone value in the environment.
    Tuple(Vec<Value>),
    /// Sentinel for `:` used as an index ("all elements in this dimension").
    /// Only meaningful inside indexing expressions; errors elsewhere.
    All,
    None,
    /// Continuous-time transfer function G(s) = num(s) / den(s).
    /// Coefficients in descending-power order (index 0 = highest power).
    TransferFn { num: Vec<f64>, den: Vec<f64> },
    /// Continuous-time state-space model: ẋ = Ax + Bu, y = Cx + Du.
    StateSpace { A: CMatrix, B: CMatrix, C: CMatrix, D: CMatrix },
}

impl Value {
    pub fn negate(self) -> Result<Value, String> {
        match self {
            Value::Scalar(n) => Ok(Value::Scalar(-n)),
            Value::Complex(c) => Ok(Value::Complex(-c)),
            Value::Vector(v) => Ok(Value::Vector(-v)),
            Value::Matrix(m) => Ok(Value::Matrix(-m)),
            Value::TransferFn { num, den } => Ok(Value::TransferFn {
                num: num.iter().map(|&x| -x).collect(),
                den,
            }),
            other => Err(format!("cannot negate {}", other.type_name())),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Scalar(_) => "scalar",
            Value::Complex(_) => "complex",
            Value::Vector(_) => "vector",
            Value::Matrix(_) => "matrix",
            Value::Bool(_) => "bool",
            Value::Str(_) => "string",
            Value::QFmt(_) => "qfmt",
            Value::Struct(_) => "struct",
            Value::Tuple(_)  => "tuple",
            Value::All => "all-index",
            Value::None => "none",
            Value::TransferFn { .. } => "tf",
            Value::StateSpace { .. } => "ss",
        }
    }

    /// Extract a QFmtSpec — errors with a descriptive message if not a QFmt value.
    pub fn to_qfmt(&self) -> Result<QFmtSpec, String> {
        match self {
            Value::QFmt(spec) => Ok(spec.clone()),
            other => Err(format!("expected qfmt spec (from qfmt()), got {}", other.type_name())),
        }
    }

    /// Promote a scalar f64 to C64
    fn scalar_to_c64(n: f64) -> C64 {
        Complex::new(n, 0.0)
    }

    /// Promote a Scalar value to Complex
    fn promote_to_complex(v: Value) -> Result<C64, String> {
        match v {
            Value::Scalar(n) => Ok(Self::scalar_to_c64(n)),
            Value::Complex(c) => Ok(c),
            other => Err(format!("cannot promote {} to complex", other.type_name())),
        }
    }

    /// Promote a value to CVector
    #[allow(dead_code)]
    fn promote_to_cvector(v: Value) -> Result<CVector, String> {
        match v {
            Value::Scalar(n) => Ok(Array1::from_vec(vec![Self::scalar_to_c64(n)])),
            Value::Complex(c) => Ok(Array1::from_vec(vec![c])),
            Value::Vector(v) => Ok(v),
            other => Err(format!("cannot promote {} to vector", other.type_name())),
        }
    }

    /// Conjugate transpose: `A'`
    /// A row vector (1×n) becomes a column vector stored as Matrix(n×1).
    /// A Matrix is conjugate-transposed normally.
    pub fn transpose(self) -> Result<Value, String> {
        match self {
            Value::Vector(v) => {
                // Row vector (1×n) → column vector (n×1), conjugated
                let n = v.len();
                let data: Vec<C64> = v.iter().map(|c| c.conj()).collect();
                let col = Array2::from_shape_vec((n, 1), data)
                    .map_err(|e| e.to_string())?;
                Ok(Value::Matrix(col))
            }
            Value::Matrix(m) => {
                let t = m.t().mapv(|c| c.conj());
                Ok(Value::Matrix(t.to_owned()))
            }
            Value::Scalar(n)  => Ok(Value::Scalar(n)),
            Value::Complex(c) => Ok(Value::Complex(c.conj())),
            other => Err(format!("cannot transpose {}", other.type_name())),
        }
    }

    /// Non-conjugate transpose: `A.'`
    /// A row vector (1×n) becomes a column vector stored as Matrix(n×1), without conjugation.
    pub fn non_conj_transpose(self) -> Result<Value, String> {
        match self {
            Value::Vector(v) => {
                let n = v.len();
                let data: Vec<C64> = v.iter().copied().collect();
                let col = Array2::from_shape_vec((n, 1), data)
                    .map_err(|e| e.to_string())?;
                Ok(Value::Matrix(col))
            }
            Value::Matrix(m) => Ok(Value::Matrix(m.t().to_owned())),
            Value::Scalar(n) => Ok(Value::Scalar(n)),
            Value::Complex(c) => Ok(Value::Complex(c)),
            other => Err(format!("cannot transpose {}", other.type_name())),
        }
    }

    /// Resolve an index value to a list of 0-based indices for a dimension of `dim_len`.
    fn resolve_index_dim(idx: &Value, dim_len: usize) -> Result<Vec<usize>, String> {
        match idx {
            Value::All => Ok((0..dim_len).collect()),
            Value::Scalar(n) => {
                let i = (*n as usize).saturating_sub(1);
                if i >= dim_len {
                    return Err(format!("index {} out of bounds (size {})", n, dim_len));
                }
                Ok(vec![i])
            }
            Value::Vector(v) => {
                v.iter().map(|c| {
                    let i = (c.re as usize).saturating_sub(1);
                    if i >= dim_len {
                        Err(format!("index {} out of bounds (size {})", c.re as usize, dim_len))
                    } else {
                        Ok(i)
                    }
                }).collect()
            }
            other => Err(format!("invalid index type: {}", other.type_name())),
        }
    }

    /// 1-based indexing into a Vector or Matrix.
    /// Single index: 1D selection. Two indices: 2D selection with `:` (All) support.
    pub fn index(self, indices: Vec<Value>) -> Result<Value, String> {
        match indices.len() {
            1 => self.index_1d(indices.into_iter().next().unwrap()),
            2 => {
                let mut it = indices.into_iter();
                let row_idx = it.next().unwrap();
                let col_idx = it.next().unwrap();
                self.index_2d(row_idx, col_idx)
            }
            n => Err(format!("indexing requires 1 or 2 arguments, got {}", n)),
        }
    }

    fn index_1d(self, idx: Value) -> Result<Value, String> {
        match self {
            Value::Vector(v) => {
                match &idx {
                    Value::All => Ok(Value::Vector(v)),
                    Value::Scalar(n) => {
                        let i = (*n as usize).saturating_sub(1);
                        if i >= v.len() {
                            return Err(format!("index {} out of bounds (length {})", n, v.len()));
                        }
                        let c = v[i];
                        if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }
                    }
                    Value::Vector(idx_v) => {
                        let result: Result<Vec<_>, _> = idx_v.iter().map(|c| {
                            let i = (c.re as usize).saturating_sub(1);
                            if i >= v.len() {
                                Err(format!("index {} out of bounds (length {})", c.re as usize, v.len()))
                            } else {
                                Ok(v[i])
                            }
                        }).collect();
                        Ok(Value::Vector(Array1::from_vec(result?)))
                    }
                    other => Err(format!("invalid index type: {}", other.type_name())),
                }
            }
            Value::Matrix(m) => {
                // Single index selects a row (1-based)
                match &idx {
                    Value::Scalar(n) => {
                        let i = (*n as usize).saturating_sub(1);
                        if i >= m.nrows() {
                            return Err(format!("row index {} out of bounds ({} rows)", n, m.nrows()));
                        }
                        Ok(Value::Vector(m.row(i).to_owned()))
                    }
                    Value::All => {
                        // M(:) — linearize to column vector (column-major order)
                        let mut flat_data: Vec<C64> = Vec::with_capacity(m.nrows() * m.ncols());
                        for c in 0..m.ncols() {
                            for r in 0..m.nrows() {
                                flat_data.push(m[[r, c]]);
                            }
                        }
                        Ok(Value::Vector(Array1::from_vec(flat_data)))
                    }
                    other => Err(format!("matrix single-index with {} not supported; use M(i,j) for element access", other.type_name())),
                }
            }
            other => Err(format!("cannot index into {}", other.type_name())),
        }
    }

    fn index_2d(self, row_idx: Value, col_idx: Value) -> Result<Value, String> {
        match self {
            Value::Matrix(m) => {
                let rows = Self::resolve_index_dim(&row_idx, m.nrows())?;
                let cols = Self::resolve_index_dim(&col_idx, m.ncols())?;

                if rows.len() == 1 && cols.len() == 1 {
                    // Single element
                    let c = m[[rows[0], cols[0]]];
                    if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }
                } else if rows.len() == 1 {
                    // Single row → Vector
                    let r = rows[0];
                    Ok(Value::Vector(Array1::from_iter(cols.iter().map(|&c| m[[r, c]]))))
                } else if cols.len() == 1 {
                    // Single column → Vector
                    let c = cols[0];
                    Ok(Value::Vector(Array1::from_iter(rows.iter().map(|&r| m[[r, c]]))))
                } else {
                    // Submatrix
                    let nr = rows.len();
                    let nc = cols.len();
                    let mut data: Vec<C64> = Vec::with_capacity(nr * nc);
                    for &r in &rows {
                        for &c in &cols {
                            data.push(m[[r, c]]);
                        }
                    }
                    Ok(Value::Matrix(Array2::from_shape_vec((nr, nc), data).map_err(|e| e.to_string())?))
                }
            }
            Value::Vector(v) => {
                // Allow v(i, 1) or v(1, j) for column/row vector indexing
                match (&row_idx, &col_idx) {
                    (_, Value::Scalar(c)) if (*c as usize) == 1 => {
                        Value::Vector(v).index_1d(row_idx)
                    }
                    (Value::Scalar(r), _) if (*r as usize) == 1 => {
                        Value::Vector(v).index_1d(col_idx)
                    }
                    _ => Err("2D indexing on a vector requires one dimension to be 1".to_string()),
                }
            }
            other => Err(format!("2D indexing requires a matrix, got {}", other.type_name())),
        }
    }

    /// Logical NOT — only valid for Bool.
    pub fn not(self) -> Result<Value, String> {
        match self {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            other => Err(format!("'!' operator requires bool, got {}", other.type_name())),
        }
    }

    pub fn binop(op: BinOp, lhs: Value, rhs: Value) -> Result<Value, String> {
        use BinOp::*;

        // Logical operators: both sides must be Bool
        if matches!(op, And | Or) {
            return match (&lhs, &rhs) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(match op {
                    And => *a && *b,
                    Or  => *a || *b,
                    _   => unreachable!(),
                })),
                _ => Err(format!(
                    "'{}' requires bool operands, got {} and {}",
                    if op == And { "&&" } else { "||" },
                    lhs.type_name(), rhs.type_name()
                )),
            };
        }

        // Comparison operators: compare scalar/complex values, return Bool
        if matches!(op, Eq | Ne | Lt | Le | Gt | Ge) {
            return match (&lhs, &rhs) {
                (Value::Scalar(a), Value::Scalar(b)) => {
                    Ok(Value::Bool(match op {
                        Eq => a == b, Ne => a != b,
                        Lt => a < b,  Le => a <= b,
                        Gt => a > b,  Ge => a >= b,
                        _ => unreachable!(),
                    }))
                }
                (Value::Bool(a), Value::Bool(b)) => {
                    Ok(Value::Bool(match op {
                        Eq => a == b,
                        Ne => a != b,
                        _ => return Err("ordered comparison not defined for bool".to_string()),
                    }))
                }
                (Value::Str(a), Value::Str(b)) => {
                    Ok(Value::Bool(match op {
                        Eq => a == b, Ne => a != b,
                        _ => return Err("ordered comparison not defined for strings".to_string()),
                    }))
                }
                // Element-wise comparison: Vector op Scalar or Scalar op Vector → real 0/1 vector
                (Value::Vector(v), Value::Scalar(b)) => {
                    let b = *b;
                    let data: Vec<C64> = v.iter().map(|c| {
                        let a = c.re;
                        let flag = match op {
                            Eq => a == b, Ne => a != b,
                            Lt => a < b,  Le => a <= b,
                            Gt => a > b,  Ge => a >= b,
                            _ => unreachable!(),
                        };
                        Complex::new(if flag { 1.0 } else { 0.0 }, 0.0)
                    }).collect();
                    Ok(Value::Vector(Array1::from_vec(data)))
                }
                (Value::Scalar(a), Value::Vector(v)) => {
                    let a = *a;
                    let data: Vec<C64> = v.iter().map(|c| {
                        let b = c.re;
                        let flag = match op {
                            Eq => a == b, Ne => a != b,
                            Lt => a < b,  Le => a <= b,
                            Gt => a > b,  Ge => a >= b,
                            _ => unreachable!(),
                        };
                        Complex::new(if flag { 1.0 } else { 0.0 }, 0.0)
                    }).collect();
                    Ok(Value::Vector(Array1::from_vec(data)))
                }
                _ => Err(format!(
                    "comparison requires two scalars (or vector op scalar), got {} and {}",
                    lhs.type_name(), rhs.type_name()
                )),
            };
        }

        // String concatenation
        if let (Value::Str(a), Value::Str(b)) = (&lhs, &rhs) {
            if op == Add {
                return Ok(Value::Str(format!("{}{}", a, b)));
            }
        }

        match (&lhs, &rhs) {
            // ── Scalar op Scalar ──────────────────────────────────────────────
            (Value::Scalar(a), Value::Scalar(b)) => {
                let (a, b) = (*a, *b);
                let result = match op {
                    Add           => a + b,
                    Sub           => a - b,
                    Mul | ElemMul => a * b,
                    Div | ElemDiv => a / b,
                    Pow | ElemPow => a.powf(b),
                    _ => unreachable!(),
                };
                Ok(Value::Scalar(result))
            }

            // ── Complex arithmetic ────────────────────────────────────────────
            (Value::Scalar(_), Value::Complex(_))
            | (Value::Complex(_), Value::Scalar(_))
            | (Value::Complex(_), Value::Complex(_)) => {
                let a = Self::promote_to_complex(lhs)?;
                let b = Self::promote_to_complex(rhs)?;
                let result = match op {
                    Add           => a + b,
                    Sub           => a - b,
                    Mul | ElemMul => a * b,
                    Div | ElemDiv => a / b,
                    Pow | ElemPow => {
                        let ln_a = Complex::new(a.norm().ln(), a.arg());
                        (b * ln_a).exp()
                    }
                    _ => unreachable!(),
                };
                Ok(Value::Complex(result))
            }

            // ── Vector op Vector ──────────────────────────────────────────────
            (Value::Vector(a), Value::Vector(b)) => {
                match op {
                    // `*` between two row vectors is a dimension error (both are 1×n).
                    // Use `dot(u,v)` for dot product, or `u * v'` (v' makes a column n×1).
                    Mul => {
                        return Err(format!(
                            "cannot multiply two row vectors [1×{}] * [1×{}]\n  hint: use `u * v'` for dot product (v' is a [{}×1] column)\n  hint: use `dot(u,v)` for the dot product directly\n  hint: use `.*` for element-wise multiply",
                            a.len(), b.len(), b.len()
                        ));
                    }
                    // Element-wise ops
                    Add | Sub | ElemMul | ElemDiv | ElemPow => {
                        if a.len() != b.len() {
                            return Err(format!("vector length mismatch: {} vs {}", a.len(), b.len()));
                        }
                        let result: CVector = match op {
                            Add     => a + b,
                            Sub     => a - b,
                            ElemMul => a * b,
                            ElemDiv => a / b,
                            ElemPow => Array1::from_iter(
                                a.iter().zip(b.iter()).map(|(&x, &y)| {
                                    let ln_x = Complex::new(x.norm().ln(), x.arg());
                                    (y * ln_x).exp()
                                })
                            ),
                            _ => unreachable!(),
                        };
                        Ok(Value::Vector(result))
                    }
                    Div => Err("cannot divide two vectors with /; use ./ for element-wise division".to_string()),
                    Pow => Err("cannot raise two vectors with ^; use .^ for element-wise power".to_string()),
                    _ => unreachable!(),
                }
            }

            // ── Scalar/Complex broadcast onto Vector ──────────────────────────
            (Value::Scalar(_), Value::Vector(_))
            | (Value::Complex(_), Value::Vector(_)) => {
                let scalar = Self::promote_to_complex(lhs)?;
                let vec = match rhs { Value::Vector(v) => v, _ => unreachable!() };
                let result: CVector = match op {
                    Add           => Array1::from_iter(vec.iter().map(|&x| scalar + x)),
                    Sub           => Array1::from_iter(vec.iter().map(|&x| scalar - x)),
                    Mul | ElemMul => Array1::from_iter(vec.iter().map(|&x| scalar * x)),
                    Div | ElemDiv => Array1::from_iter(vec.iter().map(|&x| scalar / x)),
                    Pow | ElemPow => Array1::from_iter(vec.iter().map(|&x| {
                        let ln_s = Complex::new(scalar.norm().ln(), scalar.arg());
                        (x * ln_s).exp()
                    })),
                    _ => unreachable!(),
                };
                Ok(Value::Vector(result))
            }

            // ── Vector broadcast with Scalar/Complex ──────────────────────────
            (Value::Vector(_), Value::Scalar(_))
            | (Value::Vector(_), Value::Complex(_)) => {
                let vec = match lhs { Value::Vector(v) => v, _ => unreachable!() };
                let scalar = Self::promote_to_complex(rhs)?;
                let result: CVector = match op {
                    Add           => Array1::from_iter(vec.iter().map(|&x| x + scalar)),
                    Sub           => Array1::from_iter(vec.iter().map(|&x| x - scalar)),
                    Mul | ElemMul => Array1::from_iter(vec.iter().map(|&x| x * scalar)),
                    Div | ElemDiv => Array1::from_iter(vec.iter().map(|&x| x / scalar)),
                    Pow | ElemPow => Array1::from_iter(vec.iter().map(|&x| {
                        let ln_x = Complex::new(x.norm().ln(), x.arg());
                        (scalar * ln_x).exp()
                    })),
                    _ => unreachable!(),
                };
                Ok(Value::Vector(result))
            }

            // ── Matrix * Matrix (matrix multiply for `*`, element-wise for `.*`) ──
            (Value::Matrix(a), Value::Matrix(b)) => {
                match op {
                    Mul => {
                        if a.ncols() != b.nrows() {
                            return Err(format!(
                                "matrix multiply: inner dimensions must match\n  left:  [{}×{}]\n  right: [{}×{}]\n  hint:  use .* for element-wise multiply",
                                a.nrows(), a.ncols(), b.nrows(), b.ncols()
                            ));
                        }
                        Ok(Value::Matrix(a.dot(b)))
                    }
                    Add     => {
                        if a.shape() != b.shape() {
                            return Err(format!("matrix size mismatch for +: [{}×{}] vs [{}×{}]",
                                a.nrows(), a.ncols(), b.nrows(), b.ncols()));
                        }
                        Ok(Value::Matrix(a + b))
                    }
                    Sub     => {
                        if a.shape() != b.shape() {
                            return Err(format!("matrix size mismatch for -: [{}×{}] vs [{}×{}]",
                                a.nrows(), a.ncols(), b.nrows(), b.ncols()));
                        }
                        Ok(Value::Matrix(a - b))
                    }
                    ElemMul => {
                        if a.shape() != b.shape() {
                            return Err(format!("matrix size mismatch for .*: [{}×{}] vs [{}×{}]",
                                a.nrows(), a.ncols(), b.nrows(), b.ncols()));
                        }
                        Ok(Value::Matrix(a * b))
                    }
                    ElemDiv => {
                        if a.shape() != b.shape() {
                            return Err(format!("matrix size mismatch for ./: [{}×{}] vs [{}×{}]",
                                a.nrows(), a.ncols(), b.nrows(), b.ncols()));
                        }
                        Ok(Value::Matrix(a / b))
                    }
                    Div => Err("use ./ for element-wise matrix division; or inv(A)*B for left-divide".to_string()),
                    ElemPow | Pow => {
                        if a.shape() != b.shape() {
                            return Err(format!("matrix size mismatch for .^: [{}×{}] vs [{}×{}]",
                                a.nrows(), a.ncols(), b.nrows(), b.ncols()));
                        }
                        let rows = a.nrows(); let cols = a.ncols();
                        let data: Vec<C64> = a.iter().zip(b.iter()).map(|(&x, &y)| {
                            let ln_x = Complex::new(x.norm().ln(), x.arg());
                            (y * ln_x).exp()
                        }).collect();
                        Ok(Value::Matrix(Array2::from_shape_vec((rows, cols), data).map_err(|e| e.to_string())?))
                    }
                    _ => unreachable!(),
                }
            }

            // ── Matrix * Vector (matrix × row-vector) ─────────────────────────
            // Vector is a row (1×n). Matrix(m×k) * row(1×n) requires k==1.
            // The common case M*v where v is meant as a column: write M * v' instead.
            (Value::Matrix(m), Value::Vector(v)) => {
                match op {
                    Mul => {
                        if m.ncols() != 1 {
                            return Err(format!(
                                "cannot multiply matrix [{}×{}] by row vector [1×{}]\n  hint: transpose v with v' to make a column vector [{}×1], then use M * v'",
                                m.nrows(), m.ncols(), v.len(), v.len()
                            ));
                        }
                        // Matrix(m×1) * row(1×n) = outer product Matrix(m×n)
                        let (nrows, ncols) = (m.nrows(), v.len());
                        let mut data: Vec<C64> = Vec::with_capacity(nrows * ncols);
                        for r in 0..nrows {
                            for c in 0..ncols {
                                data.push(m[[r, 0]] * v[c]);
                            }
                        }
                        Ok(Value::Matrix(Array2::from_shape_vec((nrows, ncols), data)
                            .map_err(|e| e.to_string())?))
                    }
                    _ => Err(format!(
                        "operator {:?} not defined for matrix and vector; use .* ./ .^ for element-wise ops",
                        op
                    )),
                }
            }

            // ── Vector * Matrix (row-vector × matrix) ─────────────────────────
            (Value::Vector(v), Value::Matrix(m)) => {
                match op {
                    Mul => {
                        if v.len() != m.nrows() {
                            return Err(format!(
                                "vector-matrix multiply: row vector [1×{}] * matrix [{}×{}]: inner dimension mismatch\n  hint: use .* for element-wise multiply",
                                v.len(), m.nrows(), m.ncols()
                            ));
                        }
                        // Row vector (1×n) * matrix (n×k) → row vector (1×k)
                        let k = m.ncols();
                        let result: CVector = Array1::from_iter((0..k).map(|j| {
                            v.iter().zip(m.column(j).iter()).map(|(&vi, &mij)| vi * mij).sum::<C64>()
                        }));
                        // Collapse 1×1 result to scalar (e.g. v * v' gives scalar)
                        if k == 1 {
                            let c = result[0];
                            if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }
                        } else {
                            Ok(Value::Vector(result))
                        }
                    }
                    _ => Err(format!(
                        "operator {:?} not defined for vector and matrix; use .* ./ .^ for element-wise ops",
                        op
                    )),
                }
            }

            // ── Scalar/Complex broadcast onto Matrix ──────────────────────────
            (Value::Scalar(_), Value::Matrix(_))
            | (Value::Complex(_), Value::Matrix(_)) => {
                let scalar = Self::promote_to_complex(lhs)?;
                let mat = match rhs { Value::Matrix(m) => m, _ => unreachable!() };
                let rows = mat.nrows(); let cols = mat.ncols();
                let data: Vec<C64> = mat.iter().map(|&x| match op {
                    Add           => scalar + x,
                    Sub           => scalar - x,
                    Mul | ElemMul => scalar * x,
                    Div | ElemDiv => scalar / x,
                    Pow | ElemPow => { let ln_s = Complex::new(scalar.norm().ln(), scalar.arg()); (x * ln_s).exp() }
                    _ => unreachable!(),
                }).collect();
                Ok(Value::Matrix(Array2::from_shape_vec((rows, cols), data).map_err(|e| e.to_string())?))
            }

            // ── Matrix broadcast with Scalar/Complex ──────────────────────────
            (Value::Matrix(_), Value::Scalar(_))
            | (Value::Matrix(_), Value::Complex(_)) => {
                let mat = match lhs { Value::Matrix(m) => m, _ => unreachable!() };
                let scalar = Self::promote_to_complex(rhs)?;
                let rows = mat.nrows(); let cols = mat.ncols();
                let data: Vec<C64> = mat.iter().map(|&x| match op {
                    Add           => x + scalar,
                    Sub           => x - scalar,
                    Mul | ElemMul => x * scalar,
                    Div | ElemDiv => x / scalar,
                    Pow | ElemPow => { let ln_x = Complex::new(x.norm().ln(), x.arg()); (scalar * ln_x).exp() }
                    _ => unreachable!(),
                }).collect();
                Ok(Value::Matrix(Array2::from_shape_vec((rows, cols), data).map_err(|e| e.to_string())?))
            }

            // ── TransferFn arithmetic ─────────────────────────────────────────
            (Value::TransferFn { num: n1, den: d1 }, Value::TransferFn { num: n2, den: d2 }) => {
                match op {
                    Add => Ok(Value::TransferFn {
                        num: poly_add(&poly_mul(n1, d2), &poly_mul(n2, d1)),
                        den: poly_mul(d1, d2),
                    }),
                    Sub => Ok(Value::TransferFn {
                        num: poly_sub(&poly_mul(n1, d2), &poly_mul(n2, d1)),
                        den: poly_mul(d1, d2),
                    }),
                    Mul | ElemMul => Ok(Value::TransferFn {
                        num: poly_mul(n1, n2),
                        den: poly_mul(d1, d2),
                    }),
                    _ => Err(format!("operator {:?} not defined between two tf values", op)),
                }
            }

            (Value::TransferFn { num, den }, Value::Scalar(s)) => {
                let s = *s;
                match op {
                    Mul | ElemMul => Ok(Value::TransferFn {
                        num: poly_scale(num, s),
                        den: den.clone(),
                    }),
                    Div | ElemDiv => Ok(Value::TransferFn {
                        num: poly_scale(num, 1.0 / s),
                        den: den.clone(),
                    }),
                    Add => Ok(Value::TransferFn {
                        num: poly_add(num, &poly_scale(den, s)),
                        den: den.clone(),
                    }),
                    Sub => Ok(Value::TransferFn {
                        num: poly_sub(num, &poly_scale(den, s)),
                        den: den.clone(),
                    }),
                    Pow | ElemPow => {
                        if s.fract() != 0.0 || s < 0.0 {
                            return Err(format!("tf ^ n requires a non-negative integer exponent, got {}", s));
                        }
                        let n = s as usize;
                        if n == 0 {
                            return Ok(Value::TransferFn { num: vec![1.0], den: vec![1.0] });
                        }
                        let mut rn = num.clone();
                        let mut rd = den.clone();
                        for _ in 1..n {
                            rn = poly_mul(&rn, num);
                            rd = poly_mul(&rd, den);
                        }
                        Ok(Value::TransferFn { num: rn, den: rd })
                    }
                    _ => Err(format!("operator {:?} not defined for tf and scalar", op)),
                }
            }

            (Value::Scalar(s), Value::TransferFn { num, den }) => {
                let s = *s;
                match op {
                    Mul | ElemMul => Ok(Value::TransferFn {
                        num: poly_scale(num, s),
                        den: den.clone(),
                    }),
                    Div | ElemDiv => Ok(Value::TransferFn {
                        // s / (num/den) = (s * den) / num
                        num: poly_scale(den, s),
                        den: num.clone(),
                    }),
                    Add => Ok(Value::TransferFn {
                        num: poly_add(&poly_scale(den, s), num),
                        den: den.clone(),
                    }),
                    Sub => Ok(Value::TransferFn {
                        num: poly_sub(&poly_scale(den, s), num),
                        den: den.clone(),
                    }),
                    _ => Err(format!("operator {:?} not defined for scalar and tf", op)),
                }
            }

            (a, b) => Err(format!(
                "unsupported operand types for {:?}: {} and {}",
                op, a.type_name(), b.type_name()
            )),
        }
    }

    /// Build a Value from evaluated matrix literal rows.
    ///
    /// Each element in a row may be Scalar, Complex, Vector, or Matrix.
    /// Matrices in a row are concatenated horizontally; rows separated by `;`
    /// are concatenated vertically. Rows separated by `;`, elements by space or `,`.
    pub fn from_matrix_rows(rows: Vec<Vec<Value>>) -> Result<Value, String> {
        if rows.is_empty() {
            return Ok(Value::Vector(Array1::zeros(0)));
        }

        let mut all_rows: Vec<Vec<C64>> = Vec::new();

        for row in &rows {
            if row.is_empty() {
                continue;
            }

            // Determine how many actual matrix rows this "visual row" contributes.
            let mut height = 1usize;
            for val in row {
                if let Value::Matrix(m) = val {
                    if height == 1 {
                        height = m.nrows();
                    } else if height != m.nrows() {
                        return Err(format!(
                            "matrix concat: vertical dimension mismatch ({} vs {} rows)",
                            height, m.nrows()
                        ));
                    }
                }
            }

            let mut actual_rows: Vec<Vec<C64>> = vec![Vec::new(); height];
            for val in row {
                match val {
                    Value::Scalar(n) => {
                        for r in 0..height {
                            actual_rows[r].push(Complex::new(*n, 0.0));
                        }
                    }
                    Value::Complex(c) => {
                        for r in 0..height {
                            actual_rows[r].push(*c);
                        }
                    }
                    Value::Vector(v) => {
                        if height != 1 {
                            return Err(format!(
                                "matrix concat: cannot mix a vector with matrix blocks of height {}",
                                height
                            ));
                        }
                        actual_rows[0].extend(v.iter().copied());
                    }
                    Value::Matrix(m) => {
                        for r in 0..height {
                            actual_rows[r].extend(m.row(r).iter().copied());
                        }
                    }
                    other => return Err(format!(
                        "matrix elements must be scalar, complex, vector, or matrix; got {}",
                        other.type_name()
                    )),
                }
            }
            all_rows.extend(actual_rows);
        }

        if all_rows.is_empty() {
            return Ok(Value::Vector(Array1::zeros(0)));
        }

        if all_rows.len() == 1 {
            Ok(Value::Vector(Array1::from_vec(all_rows.into_iter().next().unwrap())))
        } else {
            let ncols = all_rows[0].len();
            for (i, row) in all_rows.iter().enumerate() {
                if row.len() != ncols {
                    return Err(format!(
                        "matrix concat: row {} has {} columns, expected {}",
                        i + 1, row.len(), ncols
                    ));
                }
            }
            let nrows = all_rows.len();
            let flat: Vec<C64> = all_rows.into_iter().flatten().collect();
            let mat = Array2::from_shape_vec((nrows, ncols), flat)
                .map_err(|e| e.to_string())?;
            Ok(Value::Matrix(mat))
        }
    }

    /// Convert to CVector (for passing to DSP functions).
    /// Accepts 1D vectors, scalars, and n×1 or 1×n matrices (column/row vectors).
    pub fn to_cvector(&self) -> Result<CVector, String> {
        match self {
            Value::Vector(v) => Ok(v.clone()),
            Value::Scalar(n) => Ok(Array1::from_vec(vec![Complex::new(*n, 0.0)])),
            Value::Complex(c) => Ok(Array1::from_vec(vec![*c])),
            Value::Matrix(m) if m.ncols() == 1 => Ok(m.column(0).to_owned()),
            Value::Matrix(m) if m.nrows() == 1 => Ok(m.row(0).to_owned()),
            other => Err(format!("expected vector, got {}", other.type_name())),
        }
    }

    /// Convert to real f64.
    pub fn to_scalar(&self) -> Result<f64, String> {
        match self {
            Value::Scalar(n) => Ok(*n),
            Value::Complex(c) if c.im.abs() < 1e-10 => Ok(c.re),
            other => Err(format!("expected scalar, got {}", other.type_name())),
        }
    }

    /// Convert to usize.
    pub fn to_usize(&self) -> Result<usize, String> {
        let n = self.to_scalar()?;
        if n < 0.0 || n.fract() != 0.0 {
            return Err(format!("expected non-negative integer, got {}", n));
        }
        Ok(n as usize)
    }

    /// Convert to String.
    pub fn to_str(&self) -> Result<String, String> {
        match self {
            Value::Str(s) => Ok(s.clone()),
            other => Err(format!("expected string, got {}", other.type_name())),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MAX_ELEMS: usize = 8;

        match self {
            Value::Scalar(n) => write!(f, "{}", n),
            Value::Complex(c) => {
                if c.im >= 0.0 {
                    write!(f, "{}+{}j", c.re, c.im)
                } else {
                    write!(f, "{}{}j", c.re, c.im)
                }
            }
            Value::Vector(v) => {
                let n = v.len();
                write!(f, "[1×{}]", n)?;
                if n == 0 { return Ok(()); }
                write!(f, "  ")?;
                let show = n.min(MAX_ELEMS);
                for (i, c) in v.iter().take(show).enumerate() {
                    if i > 0 { write!(f, "  ")?; }
                    if c.im.abs() < 1e-12 {
                        write!(f, "{:.6}", c.re)?;
                    } else if c.im >= 0.0 {
                        write!(f, "{:.6}+{:.6}j", c.re, c.im)?;
                    } else {
                        write!(f, "{:.6}{:.6}j", c.re, c.im)?;
                    }
                }
                if n > MAX_ELEMS {
                    write!(f, "  ... ({} total)", n)?;
                }
                Ok(())
            }
            Value::Matrix(m) => {
                let nrows = m.nrows();
                let ncols = m.ncols();
                write!(f, "Matrix({}x{})", nrows, ncols)?;
                let show_rows = nrows.min(MAX_ELEMS);
                for r in 0..show_rows {
                    write!(f, "\n  [")?;
                    let show_cols = ncols.min(MAX_ELEMS);
                    for c_idx in 0..show_cols {
                        if c_idx > 0 { write!(f, ", ")?; }
                        let c = m[[r, c_idx]];
                        if c.im.abs() < 1e-12 {
                            write!(f, "{:.6}", c.re)?;
                        } else if c.im >= 0.0 {
                            write!(f, "{:.6}+{:.6}j", c.re, c.im)?;
                        } else {
                            write!(f, "{:.6}{:.6}j", c.re, c.im)?;
                        }
                    }
                    if ncols > MAX_ELEMS {
                        write!(f, ", ...")?;
                    }
                    write!(f, "]")?;
                }
                if nrows > MAX_ELEMS {
                    write!(f, "\n  ... ({} rows total)", nrows)?;
                }
                Ok(())
            }
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Str(s) => write!(f, "{}", s),
            Value::QFmt(spec) => {
                let int_bits = (spec.word - 1).saturating_sub(spec.frac);
                write!(f, "QFmt<{}-bit Q{}.{}, round={}, overflow={}>",
                    spec.word, int_bits, spec.frac,
                    spec.round.as_str(), spec.overflow.as_str())
            }
            Value::Tuple(vals) => {
                write!(f, "(")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Struct(fields) => {
                write!(f, "struct {{")?;
                let mut sorted: Vec<_> = fields.iter().collect();
                sorted.sort_by_key(|(k, _)| k.as_str());
                for (i, (key, val)) in sorted.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, "}}")
            }
            Value::All  => write!(f, ":"),
            Value::None => write!(f, "None"),
            Value::StateSpace { A, B, C, D } => {
                write!(f, "ss<{}-state, {} input, {} output>",
                    A.nrows(), B.ncols(), C.nrows())?;
                write!(f, "\n  A: {}x{}", A.nrows(), A.ncols())?;
                write!(f, "  B: {}x{}", B.nrows(), B.ncols())?;
                write!(f, "  C: {}x{}", C.nrows(), C.ncols())?;
                write!(f, "  D: {}x{}", D.nrows(), D.ncols())
            }
            Value::TransferFn { num, den } => {
                let ns = format_poly(num);
                let ds = format_poly(den);
                // Suppress denominator if it is exactly 1
                if den.len() == 1 && (den[0] - 1.0).abs() < 1e-12 {
                    write!(f, "{}", ns)
                } else if den.len() == 1 {
                    write!(f, "{} / {}", ns, ds)
                } else {
                    write!(f, "{} / ({})", ns, ds)
                }
            }
        }
    }
}

// ── Polynomial helpers (used by Value::TransferFn arithmetic and Display) ────

/// Multiply two polynomials (descending-power coefficients).
pub(crate) fn poly_mul(a: &[f64], b: &[f64]) -> Vec<f64> {
    if a.is_empty() || b.is_empty() { return vec![0.0]; }
    let n = a.len() + b.len() - 1;
    let mut out = vec![0.0f64; n];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            out[i + j] += ai * bj;
        }
    }
    out
}

/// Add two polynomials, aligning by degree.
pub(crate) fn poly_add(a: &[f64], b: &[f64]) -> Vec<f64> {
    let na = a.len();
    let nb = b.len();
    let n  = na.max(nb);
    let mut out = vec![0.0f64; n];
    for (i, &ai) in a.iter().enumerate() { out[i + (n - na)] += ai; }
    for (i, &bi) in b.iter().enumerate() { out[i + (n - nb)] += bi; }
    out
}

/// Subtract polynomial `b` from `a`, aligning by degree.
pub(crate) fn poly_sub(a: &[f64], b: &[f64]) -> Vec<f64> {
    let neg: Vec<f64> = b.iter().map(|&x| -x).collect();
    poly_add(a, &neg)
}

/// Scale all coefficients of a polynomial by a constant.
pub(crate) fn poly_scale(a: &[f64], s: f64) -> Vec<f64> {
    a.iter().map(|&x| x * s).collect()
}

/// Format a polynomial (descending-power) as a human-readable string.
/// e.g. [1.0, 2.0, 10.0] → "s^2 + 2s + 10"
fn format_poly(coeffs: &[f64]) -> String {
    if coeffs.is_empty() { return "0".to_string(); }
    let deg = coeffs.len() - 1;
    let mut out = String::new();
    let mut first = true;
    for (i, &c) in coeffs.iter().enumerate() {
        let power = deg - i;
        if c.abs() < 1e-12 { continue; }
        let neg = c < 0.0;
        let ac  = c.abs();
        if first {
            if neg { out.push('-'); }
            first = false;
        } else if neg {
            out.push_str(" - ");
        } else {
            out.push_str(" + ");
        }
        match power {
            0 => out.push_str(&fmt_f64(ac)),
            1 => {
                if (ac - 1.0).abs() > 1e-12 { out.push_str(&fmt_f64(ac)); }
                out.push('s');
            }
            p => {
                if (ac - 1.0).abs() > 1e-12 { out.push_str(&fmt_f64(ac)); }
                out.push_str(&format!("s^{}", p));
            }
        }
    }
    if out.is_empty() { "0".to_string() } else { out }
}

/// Format an f64 without a trailing `.0` when it is a whole number.
fn fmt_f64(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}
