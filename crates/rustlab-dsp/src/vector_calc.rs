//! Vector-calculus operators on uniform 2-D grids.
//!
//! Grid convention: `F(i, j)` corresponds to position `(x = j*dx, y = i*dy)` —
//! rows index `y`, columns index `x`. Same as MATLAB / NumPy.
//!
//! All kernels accept complex inputs — EM fields are routinely complex in the
//! frequency domain.
//!
//! Stencils:
//! - Interior: 2nd-order central differences.
//! - Boundary: 2nd-order one-sided (forward at i=0, backward at i=n-1).
//!
//! Each differentiation axis must have length ≥ 3.

use crate::error::DspError;
use num_complex::Complex;
use rustlab_core::CMatrix;
#[cfg(test)]
use rustlab_core::C64;

fn check_axis_len(name: &str, axis: &str, len: usize) -> Result<(), DspError> {
    if len < 3 {
        return Err(DspError::InvalidParameter(format!(
            "{name}: axis {axis} length {len} < 3 (need at least 3 samples for 2nd-order stencils)"
        )));
    }
    Ok(())
}

fn check_step(name: &str, label: &str, h: f64) -> Result<(), DspError> {
    if !h.is_finite() || h <= 0.0 {
        return Err(DspError::InvalidParameter(format!(
            "{name}: {label} must be a positive finite number, got {h}"
        )));
    }
    Ok(())
}

fn check_same_shape(
    name: &str,
    a: &CMatrix,
    b: &CMatrix,
    a_lbl: &str,
    b_lbl: &str,
) -> Result<(), DspError> {
    if a.dim() != b.dim() {
        return Err(DspError::InvalidParameter(format!(
            "{name}: {a_lbl} shape {:?} ≠ {b_lbl} shape {:?}",
            a.dim(),
            b.dim()
        )));
    }
    Ok(())
}

/// `∂F/∂x` along columns (axis 1). Step `dx` between adjacent columns.
fn d_dx(f: &CMatrix, dx: f64) -> CMatrix {
    let (ny, nx) = f.dim();
    let mut out = CMatrix::zeros((ny, nx));
    let two_dx = Complex::new(2.0 * dx, 0.0);
    let three = Complex::new(3.0, 0.0);
    let four = Complex::new(4.0, 0.0);
    for i in 0..ny {
        // Left boundary: 2nd-order forward
        out[[i, 0]] = (-three * f[[i, 0]] + four * f[[i, 1]] - f[[i, 2]]) / two_dx;
        // Interior: 2nd-order central
        for j in 1..nx - 1 {
            out[[i, j]] = (f[[i, j + 1]] - f[[i, j - 1]]) / two_dx;
        }
        // Right boundary: 2nd-order backward
        out[[i, nx - 1]] =
            (three * f[[i, nx - 1]] - four * f[[i, nx - 2]] + f[[i, nx - 3]]) / two_dx;
    }
    out
}

/// `∂F/∂y` along rows (axis 0). Step `dy` between adjacent rows.
fn d_dy(f: &CMatrix, dy: f64) -> CMatrix {
    let (ny, nx) = f.dim();
    let mut out = CMatrix::zeros((ny, nx));
    let two_dy = Complex::new(2.0 * dy, 0.0);
    let three = Complex::new(3.0, 0.0);
    let four = Complex::new(4.0, 0.0);
    for j in 0..nx {
        out[[0, j]] = (-three * f[[0, j]] + four * f[[1, j]] - f[[2, j]]) / two_dy;
        for i in 1..ny - 1 {
            out[[i, j]] = (f[[i + 1, j]] - f[[i - 1, j]]) / two_dy;
        }
        out[[ny - 1, j]] =
            (three * f[[ny - 1, j]] - four * f[[ny - 2, j]] + f[[ny - 3, j]]) / two_dy;
    }
    out
}

/// 2-D gradient of scalar field `F` on a uniform grid.
///
/// Returns `(Fx, Fy)` with the same shape as `F`. `Fx` is `∂F/∂x` (along
/// columns, step `dx`); `Fy` is `∂F/∂y` (along rows, step `dy`).
pub fn gradient_2d(f: &CMatrix, dx: f64, dy: f64) -> Result<(CMatrix, CMatrix), DspError> {
    check_step("gradient", "dx", dx)?;
    check_step("gradient", "dy", dy)?;
    let (ny, nx) = f.dim();
    check_axis_len("gradient", "x (columns)", nx)?;
    check_axis_len("gradient", "y (rows)", ny)?;
    Ok((d_dx(f, dx), d_dy(f, dy)))
}

/// 2-D divergence `∂Fx/∂x + ∂Fy/∂y`. Output has the same shape as the inputs.
pub fn divergence_2d(fx: &CMatrix, fy: &CMatrix, dx: f64, dy: f64) -> Result<CMatrix, DspError> {
    check_step("divergence", "dx", dx)?;
    check_step("divergence", "dy", dy)?;
    check_same_shape("divergence", fx, fy, "Fx", "Fy")?;
    let (ny, nx) = fx.dim();
    check_axis_len("divergence", "x (columns)", nx)?;
    check_axis_len("divergence", "y (rows)", ny)?;
    Ok(d_dx(fx, dx) + d_dy(fy, dy))
}

/// 2-D scalar curl `∂Fy/∂x − ∂Fx/∂y` (the z-component of `∇×F` in 3-space).
pub fn curl_2d(fx: &CMatrix, fy: &CMatrix, dx: f64, dy: f64) -> Result<CMatrix, DspError> {
    check_step("curl", "dx", dx)?;
    check_step("curl", "dy", dy)?;
    check_same_shape("curl", fx, fy, "Fx", "Fy")?;
    let (ny, nx) = fx.dim();
    check_axis_len("curl", "x (columns)", nx)?;
    check_axis_len("curl", "y (rows)", ny)?;
    Ok(d_dx(fy, dx) - d_dy(fx, dy))
}

/// Convenience constructor for filling a CMatrix from a real-valued closure.
#[cfg(test)]
pub(crate) fn from_real_fn<F: Fn(usize, usize) -> f64>(ny: usize, nx: usize, f: F) -> CMatrix {
    CMatrix::from_shape_fn((ny, nx), |(i, j)| Complex::new(f(i, j), 0.0))
}

/// Convenience constructor for filling a CMatrix from a complex-valued closure.
#[cfg(test)]
pub(crate) fn from_complex_fn<F: Fn(usize, usize) -> C64>(ny: usize, nx: usize, f: F) -> CMatrix {
    CMatrix::from_shape_fn((ny, nx), |(i, j)| f(i, j))
}
