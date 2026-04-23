//! Vector-calculus operators on uniform 2-D and 3-D grids.
//!
//! 2-D grid convention: `F(i, j)` corresponds to position `(x = j*dx, y = i*dy)` —
//! rows index `y`, columns index `x`. Same as Octave / NumPy.
//!
//! 3-D grid convention extends the 2-D one with the page axis as `z`:
//! `F(i, j, k)` ↔ `(x = j*dx, y = i*dy, z = k*dz)`. Axis 0 = y (rows),
//! axis 1 = x (cols), axis 2 = z (pages).
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
#[cfg(test)]
use rustlab_core::C64;
use rustlab_core::{CMatrix, CTensor3};

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

// ─── 3-D operators on Tensor3 ────────────────────────────────────────────────

fn check_same_shape_3d(
    name: &str,
    a: &CTensor3,
    b: &CTensor3,
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

/// Differentiate `f` along `axis` using a 2nd-order stencil (central interior,
/// one-sided boundaries). `axis`: 0 = y (rows), 1 = x (cols), 2 = z (pages).
fn d_along_axis_3d(f: &CTensor3, axis: usize, h: f64) -> CTensor3 {
    let s = f.shape();
    let (m, n, p) = (s[0], s[1], s[2]);
    let mut out = CTensor3::zeros((m, n, p));
    let two_h = Complex::new(2.0 * h, 0.0);
    let three = Complex::new(3.0, 0.0);
    let four = Complex::new(4.0, 0.0);
    match axis {
        0 => {
            for j in 0..n {
                for k in 0..p {
                    out[[0, j, k]] =
                        (-three * f[[0, j, k]] + four * f[[1, j, k]] - f[[2, j, k]]) / two_h;
                    for i in 1..m - 1 {
                        out[[i, j, k]] = (f[[i + 1, j, k]] - f[[i - 1, j, k]]) / two_h;
                    }
                    out[[m - 1, j, k]] = (three * f[[m - 1, j, k]] - four * f[[m - 2, j, k]]
                        + f[[m - 3, j, k]])
                        / two_h;
                }
            }
        }
        1 => {
            for i in 0..m {
                for k in 0..p {
                    out[[i, 0, k]] =
                        (-three * f[[i, 0, k]] + four * f[[i, 1, k]] - f[[i, 2, k]]) / two_h;
                    for j in 1..n - 1 {
                        out[[i, j, k]] = (f[[i, j + 1, k]] - f[[i, j - 1, k]]) / two_h;
                    }
                    out[[i, n - 1, k]] = (three * f[[i, n - 1, k]] - four * f[[i, n - 2, k]]
                        + f[[i, n - 3, k]])
                        / two_h;
                }
            }
        }
        2 => {
            for i in 0..m {
                for j in 0..n {
                    out[[i, j, 0]] =
                        (-three * f[[i, j, 0]] + four * f[[i, j, 1]] - f[[i, j, 2]]) / two_h;
                    for k in 1..p - 1 {
                        out[[i, j, k]] = (f[[i, j, k + 1]] - f[[i, j, k - 1]]) / two_h;
                    }
                    out[[i, j, p - 1]] = (three * f[[i, j, p - 1]] - four * f[[i, j, p - 2]]
                        + f[[i, j, p - 3]])
                        / two_h;
                }
            }
        }
        _ => unreachable!("axis must be 0, 1, or 2"),
    }
    out
}

fn check_3d_axes(name: &str, t: &CTensor3) -> Result<(), DspError> {
    let s = t.shape();
    check_axis_len(name, "y (rows)", s[0])?;
    check_axis_len(name, "x (cols)", s[1])?;
    check_axis_len(name, "z (pages)", s[2])?;
    Ok(())
}

/// 3-D gradient of scalar field `F` on a uniform grid.
///
/// Returns `(Fx, Fy, Fz)` with the same shape as `F`. `Fx` is `∂F/∂x` (along
/// columns / axis 1, step `dx`); `Fy` is `∂F/∂y` (along rows / axis 0, step
/// `dy`); `Fz` is `∂F/∂z` (along pages / axis 2, step `dz`).
pub fn gradient_3d(
    f: &CTensor3,
    dx: f64,
    dy: f64,
    dz: f64,
) -> Result<(CTensor3, CTensor3, CTensor3), DspError> {
    check_step("gradient3", "dx", dx)?;
    check_step("gradient3", "dy", dy)?;
    check_step("gradient3", "dz", dz)?;
    check_3d_axes("gradient3", f)?;
    let fx = d_along_axis_3d(f, 1, dx);
    let fy = d_along_axis_3d(f, 0, dy);
    let fz = d_along_axis_3d(f, 2, dz);
    Ok((fx, fy, fz))
}

/// 3-D divergence `∂Fx/∂x + ∂Fy/∂y + ∂Fz/∂z`. Output has the same shape as the inputs.
pub fn divergence_3d(
    fx: &CTensor3,
    fy: &CTensor3,
    fz: &CTensor3,
    dx: f64,
    dy: f64,
    dz: f64,
) -> Result<CTensor3, DspError> {
    check_step("divergence3", "dx", dx)?;
    check_step("divergence3", "dy", dy)?;
    check_step("divergence3", "dz", dz)?;
    check_same_shape_3d("divergence3", fx, fy, "Fx", "Fy")?;
    check_same_shape_3d("divergence3", fx, fz, "Fx", "Fz")?;
    check_3d_axes("divergence3", fx)?;
    Ok(d_along_axis_3d(fx, 1, dx) + d_along_axis_3d(fy, 0, dy) + d_along_axis_3d(fz, 2, dz))
}

/// 3-D curl `∇×F`. Returns `(Cx, Cy, Cz)` with each component having the same
/// shape as the inputs.
///
/// - `Cx = ∂Fz/∂y − ∂Fy/∂z`
/// - `Cy = ∂Fx/∂z − ∂Fz/∂x`
/// - `Cz = ∂Fy/∂x − ∂Fx/∂y`
pub fn curl_3d(
    fx: &CTensor3,
    fy: &CTensor3,
    fz: &CTensor3,
    dx: f64,
    dy: f64,
    dz: f64,
) -> Result<(CTensor3, CTensor3, CTensor3), DspError> {
    check_step("curl3", "dx", dx)?;
    check_step("curl3", "dy", dy)?;
    check_step("curl3", "dz", dz)?;
    check_same_shape_3d("curl3", fx, fy, "Fx", "Fy")?;
    check_same_shape_3d("curl3", fx, fz, "Fx", "Fz")?;
    check_3d_axes("curl3", fx)?;
    let cx = d_along_axis_3d(fz, 0, dy) - d_along_axis_3d(fy, 2, dz);
    let cy = d_along_axis_3d(fx, 2, dz) - d_along_axis_3d(fz, 1, dx);
    let cz = d_along_axis_3d(fy, 1, dx) - d_along_axis_3d(fx, 0, dy);
    Ok((cx, cy, cz))
}

// ─── Test helpers ────────────────────────────────────────────────────────────

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

/// Convenience constructor for filling a CTensor3 from a real-valued closure.
#[cfg(test)]
pub(crate) fn from_real_fn_3d<F: Fn(usize, usize, usize) -> f64>(
    m: usize,
    n: usize,
    p: usize,
    f: F,
) -> CTensor3 {
    CTensor3::from_shape_fn((m, n, p), |(i, j, k)| Complex::new(f(i, j, k), 0.0))
}
