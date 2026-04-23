//! Marching-squares contour extraction and Wilkinson-style auto level placement.
//!
//! Pure-functional helpers used by `contour` / `contourf` builtins to turn a
//! scalar `z(x, y)` grid into a set of line segments per level (for
//! `contour`) or per-cell band classification (for `contourf`).
//!
//! Grid convention: `z[row][col]` with `row` indexing y and `col` indexing x.
//! `x.len() == ncols`, `y.len() == nrows`.

/// A single line segment from marching squares.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub p0: (f64, f64),
    pub p1: (f64, f64),
}

/// Extract line segments where `z == level` using marching squares.
///
/// Cells with NaN corners are skipped. Saddle-point ambiguity is resolved by
/// the cell-center value (mean of the four corners).
///
/// `z[row][col]` is the scalar field, `x[col]` and `y[row]` give world
/// coordinates of grid points.
pub fn marching_squares(z: &[Vec<f64>], x: &[f64], y: &[f64], level: f64) -> Vec<LineSegment> {
    let nrows = z.len();
    if nrows < 2 {
        return Vec::new();
    }
    let ncols = z[0].len();
    if ncols < 2 || x.len() < ncols || y.len() < nrows {
        return Vec::new();
    }

    let mut segments = Vec::new();

    for r in 0..nrows - 1 {
        for c in 0..ncols - 1 {
            // Corner values, labelled like a unit square:
            //   tl ─── tr
            //    │      │
            //   bl ─── br
            // (top = larger y index)
            let bl = z[r][c];
            let br = z[r][c + 1];
            let tl = z[r + 1][c];
            let tr = z[r + 1][c + 1];

            if !bl.is_finite() || !br.is_finite() || !tl.is_finite() || !tr.is_finite() {
                continue;
            }

            // Bit code: bl=1, br=2, tr=4, tl=8.
            let mut code = 0u8;
            if bl >= level {
                code |= 1;
            }
            if br >= level {
                code |= 2;
            }
            if tr >= level {
                code |= 4;
            }
            if tl >= level {
                code |= 8;
            }
            if code == 0 || code == 15 {
                continue;
            }

            // Coordinates of the four corners.
            let xl = x[c];
            let xr = x[c + 1];
            let yb = y[r];
            let yt = y[r + 1];

            // Edge interpolation points.
            //   bottom edge (bl→br): y = yb,  x = lerp(xl, xr, level, bl, br)
            //   right  edge (br→tr): x = xr,  y = lerp(yb, yt, level, br, tr)
            //   top    edge (tl→tr): y = yt,  x = lerp(xl, xr, level, tl, tr)
            //   left   edge (bl→tl): x = xl,  y = lerp(yb, yt, level, bl, tl)
            let pb = || (lerp_x(xl, xr, level, bl, br), yb);
            let pr = || (xr, lerp_x(yb, yt, level, br, tr));
            let pt = || (lerp_x(xl, xr, level, tl, tr), yt);
            let pl = || (xl, lerp_x(yb, yt, level, bl, tl));

            // 16-case dispatch. Saddle cases (5 and 10) are split using the
            // cell-center value to choose connectivity.
            match code {
                1 => segments.push(LineSegment { p0: pb(), p1: pl() }),
                2 => segments.push(LineSegment { p0: pb(), p1: pr() }),
                3 => segments.push(LineSegment { p0: pl(), p1: pr() }),
                4 => segments.push(LineSegment { p0: pr(), p1: pt() }),
                5 => {
                    // Saddle: bl & tr above, br & tl below (or vice versa).
                    let center = 0.25 * (bl + br + tl + tr);
                    if center >= level {
                        // "Above" diagonal: connect br→pt and bl→pr (corners
                        // bl and tr are linked into one polyline).
                        segments.push(LineSegment { p0: pl(), p1: pt() });
                        segments.push(LineSegment { p0: pb(), p1: pr() });
                    } else {
                        segments.push(LineSegment { p0: pl(), p1: pb() });
                        segments.push(LineSegment { p0: pt(), p1: pr() });
                    }
                }
                6 => segments.push(LineSegment { p0: pb(), p1: pt() }),
                7 => segments.push(LineSegment { p0: pl(), p1: pt() }),
                8 => segments.push(LineSegment { p0: pl(), p1: pt() }),
                9 => segments.push(LineSegment { p0: pb(), p1: pt() }),
                10 => {
                    let center = 0.25 * (bl + br + tl + tr);
                    if center >= level {
                        segments.push(LineSegment { p0: pl(), p1: pb() });
                        segments.push(LineSegment { p0: pt(), p1: pr() });
                    } else {
                        segments.push(LineSegment { p0: pl(), p1: pt() });
                        segments.push(LineSegment { p0: pb(), p1: pr() });
                    }
                }
                11 => segments.push(LineSegment { p0: pr(), p1: pt() }),
                12 => segments.push(LineSegment { p0: pl(), p1: pr() }),
                13 => segments.push(LineSegment { p0: pb(), p1: pr() }),
                14 => segments.push(LineSegment { p0: pl(), p1: pb() }),
                _ => unreachable!(),
            }
        }
    }
    segments
}

/// Linear interpolation: find the value `v` such that the field equals `level`
/// when ramping from `f0` at coord `a` to `f1` at coord `b`.
#[inline]
fn lerp_x(a: f64, b: f64, level: f64, f0: f64, f1: f64) -> f64 {
    let denom = f1 - f0;
    if denom.abs() < 1e-300 {
        a
    } else {
        a + (b - a) * (level - f0) / denom
    }
}

/// Pick `n` round-number levels covering the finite range of `z`.
///
/// Step size is chosen from `{1, 2, 2.5, 5} × 10^k` so labels read cleanly
/// (matplotlib / Octave convention). The first level is `ceil(zmin / step) *
/// step`; subsequent levels step up by `step` until they exceed `zmax`.
///
/// Returns an empty vector if all values are non-finite or `n == 0`.
pub fn auto_levels(z: &[Vec<f64>], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    let (mut zmin, mut zmax) = (f64::INFINITY, f64::NEG_INFINITY);
    for row in z {
        for &v in row {
            if v.is_finite() {
                if v < zmin {
                    zmin = v;
                }
                if v > zmax {
                    zmax = v;
                }
            }
        }
    }
    if !zmin.is_finite() || !zmax.is_finite() || zmax - zmin < 1e-300 {
        return Vec::new();
    }
    let range = zmax - zmin;
    let rough_step = range / n as f64;
    let exponent = rough_step.log10().floor();
    let pow10 = 10f64.powf(exponent);
    let mantissa = rough_step / pow10;
    // Snap to next round mantissa from {1, 2, 2.5, 5, 10}. Pick the smallest
    // candidate ≥ mantissa so we don't generate too many levels.
    let nice = if mantissa <= 1.0 {
        1.0
    } else if mantissa <= 2.0 {
        2.0
    } else if mantissa <= 2.5 {
        2.5
    } else if mantissa <= 5.0 {
        5.0
    } else {
        10.0
    };
    let step = nice * pow10;

    let first = (zmin / step).ceil() * step;
    let mut levels = Vec::new();
    let mut v = first;
    // Generous cap to avoid runaway loops on weird input.
    let cap = (n + 1) * 4;
    while v <= zmax + 0.5 * step && levels.len() < cap {
        if v >= zmin - 0.5 * step {
            levels.push(v);
        }
        v += step;
    }
    levels
}

/// Classify the four corners of cell `(r, c)` against a banded levels list and
/// return the index of the band each corner lies in (0 = below first level,
/// `levels.len()` = at or above last level).
///
/// Used by the SVG `contourf` renderer for cell-by-cell band-coloring.
pub fn band_index(value: f64, levels: &[f64]) -> usize {
    // Half-open [levels[i], levels[i+1]) bands; final band is closed.
    let mut idx = 0usize;
    for (i, &lv) in levels.iter().enumerate() {
        if value >= lv {
            idx = i + 1;
        } else {
            break;
        }
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Build a quadratic field `f(x, y) = x² + y²` on a uniform grid spanning
    /// `[-1, 1] × [-1, 1]` with `n` samples per axis.
    fn radial_field(n: usize) -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
        let xs: Vec<f64> = (0..n)
            .map(|i| -1.0 + 2.0 * i as f64 / (n as f64 - 1.0))
            .collect();
        let ys = xs.clone();
        let z: Vec<Vec<f64>> = (0..n)
            .map(|r| (0..n).map(|c| xs[c] * xs[c] + ys[r] * ys[r]).collect())
            .collect();
        (z, xs, ys)
    }

    #[test]
    fn marching_squares_circle_radius_half() {
        // x² + y² = 0.25  is a circle of radius 0.5. With a fine enough grid,
        // every emitted segment endpoint should lie on that circle (within
        // discretisation error).
        let (z, x, y) = radial_field(41);
        let segs = marching_squares(&z, &x, &y, 0.25);
        assert!(!segs.is_empty(), "expected non-empty contour");
        for s in &segs {
            for &(px, py) in &[s.p0, s.p1] {
                let r2 = px * px + py * py;
                assert!(
                    (r2 - 0.25).abs() < 5e-3,
                    "point ({px}, {py}) off circle: r² = {r2}"
                );
            }
        }
    }

    #[test]
    fn marching_squares_no_segments_outside_field_range() {
        let (z, x, y) = radial_field(11);
        // Field max is 2.0 (at corners). Level 5 is far above.
        assert!(marching_squares(&z, &x, &y, 5.0).is_empty());
        // Field min is 0.0 (at origin). Level -1 is below.
        assert!(marching_squares(&z, &x, &y, -1.0).is_empty());
    }

    #[test]
    fn marching_squares_skips_nan_cells() {
        let mut z = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, f64::NAN, 3.0],
            vec![2.0, 3.0, 4.0],
        ];
        let x = vec![0.0, 1.0, 2.0];
        let y = x.clone();
        let segs = marching_squares(&z, &x, &y, 1.5);
        // Nothing should crash; some segments may exist outside the NaN cell.
        // The four cells touching the NaN must contribute nothing.
        for s in &segs {
            for &(px, py) in &[s.p0, s.p1] {
                // Any segment touching the inner cell (1<x<2 and 1<y<2 quadrant)
                // would imply use of NaN — assert it's not.
                let in_dead = px > 0.999 && px < 2.001 && py > 0.999 && py < 2.001;
                assert!(
                    !in_dead || (px <= 1.001 || px >= 1.999 || py <= 1.001 || py >= 1.999),
                    "segment used NaN-corner cell at ({px}, {py})"
                );
            }
        }
        // Sanity: also fill the NaN with finite value and confirm we now get more segments.
        z[1][1] = 1.5;
        let segs2 = marching_squares(&z, &x, &y, 1.5);
        assert!(segs2.len() >= segs.len());
    }

    #[test]
    fn marching_squares_constant_field_yields_nothing() {
        let z = vec![vec![1.0; 5]; 5];
        let x: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let y = x.clone();
        assert!(marching_squares(&z, &x, &y, 1.0).is_empty());
        assert!(marching_squares(&z, &x, &y, 0.5).is_empty());
        assert!(marching_squares(&z, &x, &y, 2.0).is_empty());
    }

    #[test]
    fn auto_levels_round_numbers_for_unit_range() {
        // Range 0..1, n=10 → step should be 0.1; levels 0.1, 0.2, ..., 1.0 (or 0.9).
        let z = vec![vec![0.0, 0.5, 1.0]];
        let lv = auto_levels(&z, 10);
        assert!(!lv.is_empty(), "expected non-empty levels");
        // Step between consecutive levels must equal one of the round mantissas.
        let step = lv[1] - lv[0];
        assert!(close(step, 0.1), "step should be 0.1, got {step}");
        // All levels should be multiples of step (round numbers).
        for v in &lv {
            let r = v / step;
            assert!((r - r.round()).abs() < 1e-9, "level {v} not on grid {step}");
        }
    }

    #[test]
    fn auto_levels_count_close_to_target() {
        // For range 0..1 with n=10, expect ≈ 10 levels (within ±2).
        let z = vec![vec![0.0, 1.0]];
        let lv = auto_levels(&z, 10);
        assert!(
            lv.len() >= 8 && lv.len() <= 12,
            "expected ≈10 levels, got {}",
            lv.len()
        );
    }

    #[test]
    fn auto_levels_handles_negative_range() {
        let z = vec![vec![-3.0, 0.0, 3.0]];
        let lv = auto_levels(&z, 6);
        assert!(!lv.is_empty());
        // Should span the full field range.
        assert!(lv[0] >= -3.0 - 1e-9 && lv[0] <= 0.0);
        assert!(*lv.last().unwrap() <= 3.0 + 1e-9 && *lv.last().unwrap() >= 0.0);
    }

    #[test]
    fn auto_levels_zero_range_returns_empty() {
        let z = vec![vec![1.0; 4]; 4];
        assert!(auto_levels(&z, 5).is_empty());
    }

    #[test]
    fn auto_levels_zero_count_returns_empty() {
        let z = vec![vec![0.0, 1.0]];
        assert!(auto_levels(&z, 0).is_empty());
    }

    #[test]
    fn band_index_classifies_correctly() {
        let levels = vec![0.0, 1.0, 2.0];
        assert_eq!(band_index(-0.5, &levels), 0);
        assert_eq!(band_index(0.0, &levels), 1);
        assert_eq!(band_index(0.5, &levels), 1);
        assert_eq!(band_index(1.0, &levels), 2);
        assert_eq!(band_index(1.5, &levels), 2);
        assert_eq!(band_index(2.0, &levels), 3);
        assert_eq!(band_index(99.0, &levels), 3);
    }
}
