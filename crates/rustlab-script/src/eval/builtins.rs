use std::collections::HashMap;
use rand::Rng;
use rand_distr::{Normal, Uniform, Distribution};
use rustlab_core::{C64, CVector, CMatrix};
use rustlab_dsp::{
    fir_lowpass, fir_highpass, fir_bandpass,
    fir_lowpass_kaiser, fir_highpass_kaiser, fir_bandpass_kaiser,
    fir_notch, freqz, firpm,
    fft, ifft, fftshift, fftfreq,
    butterworth_lowpass, butterworth_highpass,
    WindowFunction,
    QFmtSpec, quantize_scalar, snr_db,
};
use rustlab_dsp::fixed::{qadd as fixed_qadd, qmul as fixed_qmul, qconv as fixed_qconv};
use rustlab_core::{RoundMode, OverflowMode};
use rustlab_dsp::convolution::convolve;
use rustlab_plot::{
    plot_db, plot_histogram,
    save_plot, save_stem, save_db, save_histogram,
    compute_histogram, histogram_matrix,
    render_figure_terminal, render_figure_file,
    imagesc_terminal, save_imagesc_cmap,
    push_xy_line, push_xy_stem,
    LineStyle, SeriesColor, FIGURE,
};
use ndarray::{Array1, Array2};
use num_complex::Complex;
use crate::eval::value::Value;
use crate::error::ScriptError;

pub type BuiltinFn = fn(Vec<Value>) -> Result<Value, ScriptError>;

pub struct BuiltinRegistry {
    map: HashMap<String, BuiltinFn>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn with_defaults() -> Self {
        let mut r = Self::new();
        // DSP
        r.register("fir_lowpass",          builtin_fir_lowpass);
        r.register("fir_highpass",         builtin_fir_highpass);
        r.register("fir_bandpass",         builtin_fir_bandpass);
        r.register("butterworth_lowpass",  builtin_butterworth_lowpass);
        r.register("butterworth_highpass", builtin_butterworth_highpass);
        r.register("convolve",             builtin_convolve);
        r.register("window",               builtin_window);
        // FFT
        r.register("fft",      builtin_fft);
        r.register("ifft",     builtin_ifft);
        r.register("fftshift", builtin_fftshift);
        r.register("fftfreq",  builtin_fftfreq);
        r.register("spectrum", builtin_spectrum);
        // Kaiser FIR
        r.register("fir_lowpass_kaiser",  builtin_fir_lowpass_kaiser);
        r.register("fir_highpass_kaiser", builtin_fir_highpass_kaiser);
        r.register("fir_bandpass_kaiser", builtin_fir_bandpass_kaiser);
        r.register("fir_notch",           builtin_fir_notch);
        r.register("freqz",               builtin_freqz);
        // Parks-McClellan optimal FIR
        r.register("firpm", builtin_firpm);
        // Fixed-point quantization
        r.register("qfmt",     builtin_qfmt);
        r.register("quantize", builtin_quantize);
        r.register("qadd",     builtin_qadd);
        r.register("qmul",     builtin_qmul);
        r.register("qconv",    builtin_qconv);
        r.register("snr",      builtin_snr);
        // Math
        r.register("abs",   builtin_abs);
        r.register("angle", builtin_angle);
        r.register("real",  builtin_real);
        r.register("imag",  builtin_imag);
        r.register("cos",   builtin_cos);
        r.register("sin",   builtin_sin);
        r.register("sqrt",  builtin_sqrt);
        r.register("exp",   builtin_exp);
        r.register("log",    builtin_log);
        r.register("log10",  builtin_log10);
        r.register("log2",   builtin_log2);
        r.register("atan2",  builtin_atan2);
        r.register("meshgrid", builtin_meshgrid);
        // Array construction
        r.register("zeros",    builtin_zeros);
        r.register("ones",     builtin_ones);
        r.register("linspace", builtin_linspace);
        r.register("rand",      builtin_rand);
        r.register("randn",     builtin_randn);
        r.register("randi",     builtin_randi);
        r.register("histogram", builtin_histogram);
        r.register("savehist",  builtin_savehist);
        r.register("mean",     builtin_mean);
        r.register("std",      builtin_std);
        r.register("min",      builtin_min);
        r.register("max",      builtin_max);
        r.register("len",      builtin_len);
        r.register("length",   builtin_len);   // alias for len
        r.register("numel",    builtin_numel);
        r.register("size",     builtin_size);
        // I/O
        r.register("print", builtin_print);
        r.register("plot",  builtin_plot);
        r.register("stem",  builtin_stem);
        r.register("plotdb",   builtin_plotdb);
        r.register("savefig",  builtin_savefig);
        r.register("savestem", builtin_savestem);
        r.register("savedb",   builtin_savedb);
        // Figure state control
        r.register("figure",      builtin_figure);
        r.register("hold",        builtin_hold);
        r.register("grid",        builtin_grid);
        r.register("xlabel",      builtin_xlabel);
        r.register("ylabel",      builtin_ylabel);
        r.register("title",       builtin_title);
        r.register("xlim",        builtin_xlim);
        r.register("ylim",        builtin_ylim);
        r.register("subplot",     builtin_subplot);
        r.register("legend",      builtin_legend);
        r.register("imagesc",     builtin_imagesc);
        r.register("saveimagesc", builtin_saveimagesc);
        // Import / export
        r.register("save", builtin_save);
        r.register("load", builtin_load);
        r.register("whos", builtin_whos_file);
        // Matrix construction
        r.register("eye",       builtin_eye);
        // Matrix operations
        r.register("transpose", builtin_transpose);
        r.register("diag",      builtin_diag);
        r.register("trace",     builtin_trace);
        r.register("reshape",   builtin_reshape);
        r.register("repmat",    builtin_repmat);
        r.register("horzcat",   builtin_horzcat);
        r.register("vertcat",   builtin_vertcat);
        // Linear algebra
        r.register("dot",       builtin_dot);
        r.register("cross",     builtin_cross);
        r.register("norm",      builtin_norm);
        r.register("det",       builtin_det);
        r.register("inv",       builtin_inv);
        r.register("linsolve",  builtin_linsolve);
        r.register("eig",       builtin_eig);
        // Number theory
        r.register("factor",    builtin_factor);
        // Output
        r.register("disp",    builtin_disp);
        r.register("fprintf", builtin_fprintf);
        // Aggregates
        r.register("all", builtin_all);
        r.register("any", builtin_any);
        // Matrix analysis
        r.register("rank",  builtin_rank);
        r.register("roots", builtin_roots);
        // Transfer function (Phase 2)
        r.register("tf",   builtin_tf);
        r.register("pole", builtin_pole);
        r.register("zero", builtin_zero);
        // State-space (Phase 3)
        r.register("ss",   builtin_ss);
        r.register("ctrb", builtin_ctrb);
        r.register("obsv", builtin_obsv);
        // Frequency & time-domain analysis (Phase 4)
        r.register("bode",   builtin_bode);
        r.register("step",   builtin_step);
        r.register("margin", builtin_margin);
        // Optimal control (Phase 5)
        r.register("lqr",    builtin_lqr);
        r.register("rlocus", builtin_rlocus);
        // Struct construction
        r.register("struct",    builtin_struct);
        // Type inspection
        r.register("isstruct",  builtin_isstruct);
        r.register("fieldnames", builtin_fieldnames);
        r.register("isfield",   builtin_isfield);
        r.register("rmfield",   builtin_rmfield);
        r
    }

    pub fn register(&mut self, name: impl Into<String>, f: BuiltinFn) {
        self.map.insert(name.into(), f);
    }

    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value, ScriptError> {
        match self.map.get(name) {
            Some(f) => f(args),
            None    => Err(ScriptError::UndefinedFn(name.to_string())),
        }
    }
}

// ─── Helper macros / functions ─────────────────────────────────────────────

fn check_args(name: &str, args: &[Value], expected: usize) -> Result<(), ScriptError> {
    if args.len() != expected {
        Err(ScriptError::ArgCount {
            name: name.to_string(),
            expected,
            got: args.len(),
        })
    } else {
        Ok(())
    }
}

fn check_args_range(name: &str, args: &[Value], min: usize, max: usize) -> Result<(), ScriptError> {
    if args.len() < min || args.len() > max {
        Err(ScriptError::ArgCount {
            name: name.to_string(),
            expected: min,
            got: args.len(),
        })
    } else {
        Ok(())
    }
}

fn parse_window(val: &Value) -> Result<WindowFunction, ScriptError> {
    let s = val.to_str().map_err(ScriptError::Type)?;
    WindowFunction::from_str(&s, None).map_err(ScriptError::Dsp)
}

fn cvector_to_value(v: CVector) -> Value {
    Value::Vector(v)
}

// ─── DSP builtins ──────────────────────────────────────────────────────────

fn builtin_fir_lowpass(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_lowpass", &args, 4)?;
    let num_taps  = args[0].to_usize().map_err(ScriptError::Type)?;
    let cutoff_hz = args[1].to_scalar().map_err(ScriptError::Type)?;
    let sr        = args[2].to_scalar().map_err(ScriptError::Type)?;
    let win       = parse_window(&args[3])?;
    let filter    = fir_lowpass(num_taps, cutoff_hz, sr, win)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_fir_highpass(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_highpass", &args, 4)?;
    let num_taps  = args[0].to_usize().map_err(ScriptError::Type)?;
    let cutoff_hz = args[1].to_scalar().map_err(ScriptError::Type)?;
    let sr        = args[2].to_scalar().map_err(ScriptError::Type)?;
    let win       = parse_window(&args[3])?;
    let filter    = fir_highpass(num_taps, cutoff_hz, sr, win)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_fir_bandpass(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_bandpass", &args, 5)?;
    let num_taps = args[0].to_usize().map_err(ScriptError::Type)?;
    let low_hz   = args[1].to_scalar().map_err(ScriptError::Type)?;
    let high_hz  = args[2].to_scalar().map_err(ScriptError::Type)?;
    let sr       = args[3].to_scalar().map_err(ScriptError::Type)?;
    let win      = parse_window(&args[4])?;
    let filter   = fir_bandpass(num_taps, low_hz, high_hz, sr, win)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_butterworth_lowpass(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("butterworth_lowpass", &args, 3)?;
    let order     = args[0].to_usize().map_err(ScriptError::Type)?;
    let cutoff_hz = args[1].to_scalar().map_err(ScriptError::Type)?;
    let sr        = args[2].to_scalar().map_err(ScriptError::Type)?;
    let filter    = butterworth_lowpass(order, cutoff_hz, sr)?;
    // Return b coefficients as a complex vector for script use
    let coeffs: CVector = Array1::from_iter(
        filter.b.iter().map(|&x| Complex::new(x, 0.0))
    );
    Ok(Value::Vector(coeffs))
}

fn builtin_butterworth_highpass(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("butterworth_highpass", &args, 3)?;
    let order     = args[0].to_usize().map_err(ScriptError::Type)?;
    let cutoff_hz = args[1].to_scalar().map_err(ScriptError::Type)?;
    let sr        = args[2].to_scalar().map_err(ScriptError::Type)?;
    let filter    = butterworth_highpass(order, cutoff_hz, sr)?;
    let coeffs: CVector = Array1::from_iter(
        filter.b.iter().map(|&x| Complex::new(x, 0.0))
    );
    Ok(Value::Vector(coeffs))
}

fn builtin_convolve(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("convolve", &args, 2)?;
    let x = args[0].to_cvector().map_err(ScriptError::Type)?;
    let h = args[1].to_cvector().map_err(ScriptError::Type)?;
    let result = convolve(&x, &h)?;
    Ok(Value::Vector(result))
}

fn builtin_window(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("window", &args, 2)?;
    let win = parse_window(&args[0])?;
    let n   = args[1].to_usize().map_err(ScriptError::Type)?;
    let w   = win.generate(n);
    // Convert RVector to CVector
    let cv: CVector = Array1::from_iter(w.iter().map(|&x| Complex::new(x, 0.0)));
    Ok(Value::Vector(cv))
}

// ─── Math builtins ─────────────────────────────────────────────────────────

fn builtin_abs(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("abs", &args, 1)?;
    match &args[0] {
        Value::Scalar(n) => Ok(Value::Scalar(n.abs())),
        Value::Complex(c) => Ok(Value::Scalar(c.norm())),
        Value::Vector(v) => {
            let result: CVector = Array1::from_iter(
                v.iter().map(|&c| Complex::new(c.norm(), 0.0))
            );
            Ok(Value::Vector(result))
        }
        other => Err(ScriptError::Type(format!("abs: unsupported type {}", other))),
    }
}

fn builtin_angle(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("angle", &args, 1)?;
    match &args[0] {
        Value::Scalar(n) => Ok(Value::Scalar(if *n >= 0.0 { 0.0 } else { std::f64::consts::PI })),
        Value::Complex(c) => Ok(Value::Scalar(c.arg())),
        Value::Vector(v) => {
            let result: CVector = Array1::from_iter(
                v.iter().map(|&c| Complex::new(c.arg(), 0.0))
            );
            Ok(Value::Vector(result))
        }
        other => Err(ScriptError::Type(format!("angle: unsupported type {}", other))),
    }
}

fn builtin_real(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("real", &args, 1)?;
    match &args[0] {
        Value::Scalar(n) => Ok(Value::Scalar(*n)),
        Value::Complex(c) => Ok(Value::Scalar(c.re)),
        Value::Vector(v) => {
            let result: CVector = Array1::from_iter(
                v.iter().map(|&c| Complex::new(c.re, 0.0))
            );
            Ok(Value::Vector(result))
        }
        other => Err(ScriptError::Type(format!("real: unsupported type {}", other))),
    }
}

fn builtin_imag(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("imag", &args, 1)?;
    match &args[0] {
        Value::Scalar(n) => Ok(Value::Scalar(if *n == 0.0 { 0.0 } else { 0.0 })),
        Value::Complex(c) => Ok(Value::Scalar(c.im)),
        Value::Vector(v) => {
            let result: CVector = Array1::from_iter(
                v.iter().map(|&c| Complex::new(c.im, 0.0))
            );
            Ok(Value::Vector(result))
        }
        other => Err(ScriptError::Type(format!("imag: unsupported type {}", other))),
    }
}

fn apply_scalar_fn_to_value(
    name: &str,
    args: Vec<Value>,
    f: impl Fn(f64) -> f64,
    fc: impl Fn(Complex<f64>) -> Complex<f64>,
) -> Result<Value, ScriptError> {
    check_args(name, &args, 1)?;
    match &args[0] {
        Value::Scalar(n) => Ok(Value::Scalar(f(*n))),
        Value::Complex(c) => Ok(Value::Complex(fc(*c))),
        Value::Vector(v) => {
            let result: CVector = Array1::from_iter(v.iter().map(|&c| fc(c)));
            Ok(Value::Vector(result))
        }
        other => Err(ScriptError::Type(format!("{}: unsupported type {}", name, other))),
    }
}

fn builtin_cos(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("cos", args, f64::cos, |c: Complex<f64>| c.cos())
}

fn builtin_sin(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("sin", args, f64::sin, |c: Complex<f64>| c.sin())
}

fn builtin_sqrt(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("sqrt", args, f64::sqrt, |c: Complex<f64>| c.sqrt())
}

fn builtin_exp(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("exp", args, f64::exp, |c: Complex<f64>| c.exp())
}

fn builtin_log(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("log", args, f64::ln, |c: Complex<f64>| c.ln())
}

fn builtin_log10(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("log10", args, f64::log10, |c: Complex<f64>| c.ln() / f64::ln(10.0))
}

fn builtin_log2(args: Vec<Value>) -> Result<Value, ScriptError> {
    apply_scalar_fn_to_value("log2", args, f64::log2, |c: Complex<f64>| c.ln() / f64::ln(2.0))
}

// ─── atan2(y, x) ──────────────────────────────────────────────────────────────

/// Element-wise four-quadrant arctangent: atan2(y, x) → angle in radians.
/// Both arguments may be scalar, vector, or matrix; shapes must match (or one scalar).
fn builtin_atan2(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("atan2", &args, 2)?;

    /// Extract real part of a C64, ignoring imaginary (atan2 is real-valued).
    fn re(c: C64) -> f64 { c.re }

    match (&args[0], &args[1]) {
        // scalar × scalar
        (Value::Scalar(y), Value::Scalar(x)) =>
            Ok(Value::Scalar(y.atan2(*x))),

        // scalar × vector
        (Value::Scalar(y), Value::Vector(xv)) => {
            let v = Array1::from_iter(xv.iter().map(|&xc| Complex::new(y.atan2(re(xc)), 0.0)));
            Ok(Value::Vector(v))
        }
        (Value::Vector(yv), Value::Scalar(x)) => {
            let v = Array1::from_iter(yv.iter().map(|&yc| Complex::new(re(yc).atan2(*x), 0.0)));
            Ok(Value::Vector(v))
        }

        // vector × vector
        (Value::Vector(yv), Value::Vector(xv)) => {
            if yv.len() != xv.len() {
                return Err(ScriptError::Type(format!(
                    "atan2: vector lengths must match ({} vs {})", yv.len(), xv.len()
                )));
            }
            let v = Array1::from_iter(
                yv.iter().zip(xv.iter()).map(|(&yc, &xc)| Complex::new(re(yc).atan2(re(xc)), 0.0))
            );
            Ok(Value::Vector(v))
        }

        // scalar × matrix
        (Value::Scalar(y), Value::Matrix(xm)) => {
            let m = xm.mapv(|xc| Complex::new(y.atan2(re(xc)), 0.0));
            Ok(Value::Matrix(m))
        }
        (Value::Matrix(ym), Value::Scalar(x)) => {
            let m = ym.mapv(|yc| Complex::new(re(yc).atan2(*x), 0.0));
            Ok(Value::Matrix(m))
        }

        // matrix × matrix
        (Value::Matrix(ym), Value::Matrix(xm)) => {
            if ym.shape() != xm.shape() {
                return Err(ScriptError::Type(format!(
                    "atan2: matrix shapes must match ({}×{} vs {}×{})",
                    ym.nrows(), ym.ncols(), xm.nrows(), xm.ncols()
                )));
            }
            let m = Array2::from_shape_fn(ym.raw_dim(), |(i, j)| {
                Complex::new(re(ym[[i, j]]).atan2(re(xm[[i, j]])), 0.0)
            });
            Ok(Value::Matrix(m))
        }

        (y, x) => Err(ScriptError::Type(format!(
            "atan2: unsupported types {} and {}", y.type_name(), x.type_name()
        ))),
    }
}

// ─── meshgrid(x, y) ───────────────────────────────────────────────────────────

/// `[X, Y] = meshgrid(x, y)`
///
/// Given row vector x (length m) and row vector y (length n), return two n×m matrices:
///   X[i, j] = x[j]   (x varies along columns)
///   Y[i, j] = y[i]   (y varies along rows)
///
/// Uses 'xy' indexing: x varies along columns, y varies along rows.
fn builtin_meshgrid(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("meshgrid", &args, 2)?;

    let xv = args[0].to_cvector().map_err(|e| ScriptError::Type(format!("meshgrid: x: {}", e)))?;
    let yv = args[1].to_cvector().map_err(|e| ScriptError::Type(format!("meshgrid: y: {}", e)))?;

    let (m, n) = (xv.len(), yv.len()); // m cols, n rows

    let x_mat = Array2::from_shape_fn((n, m), |(_, j)| xv[j]);
    let y_mat = Array2::from_shape_fn((n, m), |(i, _)| yv[i]);

    Ok(Value::Tuple(vec![
        Value::Matrix(x_mat),
        Value::Matrix(y_mat),
    ]))
}

// ─── Array construction ────────────────────────────────────────────────────

fn builtin_zeros(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("zeros", &args, 1, 2)?;
    if args.len() == 2 {
        let m = args[0].to_usize().map_err(ScriptError::Type)?;
        let n = args[1].to_usize().map_err(ScriptError::Type)?;
        Ok(Value::Matrix(Array2::zeros((m, n))))
    } else {
        let n = args[0].to_usize().map_err(ScriptError::Type)?;
        Ok(Value::Vector(Array1::zeros(n)))
    }
}

fn builtin_ones(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("ones", &args, 1, 2)?;
    if args.len() == 2 {
        let m = args[0].to_usize().map_err(ScriptError::Type)?;
        let n = args[1].to_usize().map_err(ScriptError::Type)?;
        Ok(Value::Matrix(Array2::from_elem((m, n), Complex::new(1.0, 0.0))))
    } else {
        let n = args[0].to_usize().map_err(ScriptError::Type)?;
        Ok(Value::Vector(Array1::from_elem(n, Complex::new(1.0, 0.0))))
    }
}

fn builtin_linspace(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("linspace", &args, 3)?;
    let start = args[0].to_scalar().map_err(ScriptError::Type)?;
    let stop  = args[1].to_scalar().map_err(ScriptError::Type)?;
    let n     = args[2].to_usize().map_err(ScriptError::Type)?;
    if n == 0 {
        return Ok(Value::Vector(Array1::zeros(0)));
    }
    if n == 1 {
        return Ok(Value::Vector(Array1::from_vec(vec![Complex::new(start, 0.0)])));
    }
    let step = (stop - start) / (n - 1) as f64;
    let v: CVector = Array1::from_iter(
        (0..n).map(|i| Complex::new(start + step * i as f64, 0.0))
    );
    Ok(Value::Vector(v))
}

fn builtin_rand(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("rand", &args, 1, 2)?;
    let mut rng = rand::thread_rng();
    let dist = Uniform::new(0.0_f64, 1.0);
    if args.len() == 2 {
        let m = args[0].to_usize().map_err(ScriptError::Type)?;
        let n = args[1].to_usize().map_err(ScriptError::Type)?;
        let data: Vec<C64> = (0..m*n).map(|_| Complex::new(dist.sample(&mut rng), 0.0)).collect();
        Ok(Value::Matrix(Array2::from_shape_vec((m, n), data).unwrap()))
    } else {
        let n = args[0].to_usize().map_err(ScriptError::Type)?;
        Ok(Value::Vector(Array1::from_iter((0..n).map(|_| Complex::new(dist.sample(&mut rng), 0.0)))))
    }
}

fn builtin_randn(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("randn", &args, 1, 2)?;
    let mut rng = rand::thread_rng();
    let dist = Normal::new(0.0_f64, 1.0)
        .map_err(|e| ScriptError::Type(format!("randn: {e}")))?;
    if args.len() == 2 {
        let m = args[0].to_usize().map_err(ScriptError::Type)?;
        let n = args[1].to_usize().map_err(ScriptError::Type)?;
        let data: Vec<C64> = (0..m*n).map(|_| Complex::new(dist.sample(&mut rng), 0.0)).collect();
        Ok(Value::Matrix(Array2::from_shape_vec((m, n), data).unwrap()))
    } else {
        let n = args[0].to_usize().map_err(ScriptError::Type)?;
        Ok(Value::Vector(Array1::from_iter((0..n).map(|_| Complex::new(dist.sample(&mut rng), 0.0)))))
    }
}

fn builtin_randi(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() || args.len() > 2 {
        return Err(ScriptError::Type("randi: expected randi(imax) or randi(imax, n) or randi([lo,hi], n)".to_string()));
    }
    // First arg: scalar imax → range [1, imax], or 2-element vector [lo, hi]
    let (lo, hi) = match &args[0] {
        Value::Vector(v) if v.len() >= 2 => (v[0].re as i64, v[1].re as i64),
        Value::Vector(v) if v.len() == 1 => (1i64, v[0].re as i64),
        _ => {
            let imax = args[0].to_scalar().map_err(ScriptError::Type)? as i64;
            (1i64, imax)
        }
    };
    if lo > hi {
        return Err(ScriptError::Type(format!("randi: lo ({lo}) must be <= hi ({hi})")));
    }
    let mut rng = rand::thread_rng();
    if args.len() == 1 {
        // Return a single scalar integer
        Ok(Value::Scalar(rng.gen_range(lo..=hi) as f64))
    } else {
        let n = args[1].to_usize().map_err(ScriptError::Type)?;
        let v: CVector = Array1::from_iter(
            (0..n).map(|_| Complex::new(rng.gen_range(lo..=hi) as f64, 0.0))
        );
        Ok(Value::Vector(v))
    }
}

fn builtin_histogram(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() || args.len() > 2 {
        return Err(ScriptError::Type("histogram: expected histogram(v) or histogram(v, n_bins)".to_string()));
    }
    let data = to_real_vector(&args[0])?;
    let n_bins = if args.len() == 2 {
        args[1].to_usize().map_err(ScriptError::Type)?
    } else {
        10
    };
    plot_histogram(&data, n_bins, "Histogram")
        .map_err(|e| ScriptError::Type(e.to_string()))?;
    let (centers, counts, _) = compute_histogram(&data, n_bins);
    Ok(Value::Matrix(histogram_matrix(&centers, &counts)))
}

fn builtin_savehist(args: Vec<Value>) -> Result<Value, ScriptError> {
    // savehist(v, "file")                  → 10 bins, empty title
    // savehist(v, "file", "title")         → 10 bins
    // savehist(v, n, "file")               → n bins, empty title
    // savehist(v, n, "file", "title")      → n bins
    if args.len() < 2 || args.len() > 4 {
        return Err(ScriptError::Type(
            "savehist: expected savehist(v, file) or savehist(v, n, file) or savehist(v, n, file, title)".to_string()
        ));
    }
    let data = to_real_vector(&args[0])?;
    // Detect whether arg[1] is n_bins (scalar) or a file path (string)
    let (n_bins, path, title) = if let Value::Str(s) = &args[1] {
        let t = args.get(2).and_then(|a| if let Value::Str(t) = a { Some(t.as_str()) } else { None }).unwrap_or("");
        (10usize, s.as_str(), t)
    } else {
        let n = args[1].to_usize().map_err(ScriptError::Type)?;
        let path = match args.get(2) {
            Some(Value::Str(s)) => s.as_str(),
            _ => return Err(ScriptError::Type("savehist: file path must be a string".to_string())),
        };
        let t = args.get(3).and_then(|a| if let Value::Str(t) = a { Some(t.as_str()) } else { None }).unwrap_or("");
        (n, path, t)
    };
    save_histogram(&data, n_bins, title, path)
        .map_err(|e| ScriptError::Type(e.to_string()))?;
    Ok(Value::None)
}

fn builtin_min(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("min", &args, 1)?;
    match &args[0] {
        Value::Vector(v) if !v.is_empty() => {
            let m = v.iter().map(|c| c.re).fold(f64::INFINITY, f64::min);
            Ok(Value::Scalar(m))
        }
        Value::Scalar(s) => Ok(Value::Scalar(*s)),
        _ => Err(ScriptError::Type("min: argument must be a non-empty vector or scalar".to_string())),
    }
}

fn builtin_max(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("max", &args, 1)?;
    match &args[0] {
        Value::Vector(v) if !v.is_empty() => {
            let m = v.iter().map(|c| c.re).fold(f64::NEG_INFINITY, f64::max);
            Ok(Value::Scalar(m))
        }
        Value::Scalar(s) => Ok(Value::Scalar(*s)),
        _ => Err(ScriptError::Type("max: argument must be a non-empty vector or scalar".to_string())),
    }
}

fn builtin_mean(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("mean", &args, 1)?;
    match &args[0] {
        Value::Vector(v) if !v.is_empty() => {
            let sum: Complex<f64> = v.iter().copied().sum();
            Ok(Value::Complex(sum / v.len() as f64))
        }
        Value::Scalar(s) => Ok(Value::Scalar(*s)),
        Value::Complex(c) => Ok(Value::Complex(*c)),
        _ => Err(ScriptError::Type("mean: argument must be a non-empty vector or scalar".to_string())),
    }
}

fn builtin_std(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("std", &args, 1)?;
    match &args[0] {
        Value::Vector(v) if v.len() > 1 => {
            let n = v.len() as f64;
            let mean: Complex<f64> = v.iter().copied().sum::<Complex<f64>>() / n;
            let variance: f64 = v.iter().map(|&x| (x - mean).norm_sqr()).sum::<f64>() / (n - 1.0);
            Ok(Value::Scalar(variance.sqrt()))
        }
        Value::Vector(v) if v.len() == 1 => Ok(Value::Scalar(0.0)),
        Value::Scalar(_) | Value::Complex(_) => Ok(Value::Scalar(0.0)),
        _ => Err(ScriptError::Type("std: argument must be a non-empty vector or scalar".to_string())),
    }
}

fn builtin_len(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("len", &args, 1)?;
    match &args[0] {
        Value::Vector(v) => Ok(Value::Scalar(v.len() as f64)),
        Value::Matrix(m) => Ok(Value::Scalar(m.nrows() as f64)),
        Value::Str(s) => Ok(Value::Scalar(s.len() as f64)),
        other => Err(ScriptError::Type(format!("len: unsupported type {}", other))),
    }
}

fn builtin_numel(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("numel", &args, 1)?;
    let n = match &args[0] {
        Value::Vector(v) => v.len(),
        Value::Matrix(m) => m.nrows() * m.ncols(),
        Value::Scalar(_) | Value::Complex(_) => 1,
        other => return Err(ScriptError::Type(format!("numel: unsupported type {}", other))),
    };
    Ok(Value::Scalar(n as f64))
}

fn builtin_size(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("size", &args, 1, 2)?;
    let (nrows, ncols) = match &args[0] {
        Value::Vector(v) => (1usize, v.len()),
        Value::Matrix(m) => (m.nrows(), m.ncols()),
        Value::Scalar(_) | Value::Complex(_) => (1, 1),
        other => return Err(ScriptError::Type(format!("size: unsupported type {}", other.type_name()))),
    };
    if args.len() == 2 {
        let dim = args[1].to_usize().map_err(ScriptError::Type)?;
        match dim {
            1 => Ok(Value::Scalar(nrows as f64)),
            2 => Ok(Value::Scalar(ncols as f64)),
            _ => Err(ScriptError::Type(format!("size: dim must be 1 or 2, got {}", dim))),
        }
    } else {
        Ok(Value::Vector(Array1::from_vec(vec![
            Complex::new(nrows as f64, 0.0),
            Complex::new(ncols as f64, 0.0),
        ])))
    }
}

// ─── FFT builtins ──────────────────────────────────────────────────────────

fn builtin_fft(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fft", &args, 1)?;
    let v = args[0].to_cvector().map_err(ScriptError::Type)?;
    let result = fft(&v).map_err(ScriptError::Dsp)?;
    Ok(Value::Vector(result))
}

fn builtin_ifft(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("ifft", &args, 1)?;
    let v = args[0].to_cvector().map_err(ScriptError::Type)?;
    let result = ifft(&v).map_err(ScriptError::Dsp)?;
    Ok(Value::Vector(result))
}

fn builtin_fftshift(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fftshift", &args, 1)?;
    let v = args[0].to_cvector().map_err(ScriptError::Type)?;
    Ok(Value::Vector(fftshift(&v)))
}

fn builtin_fftfreq(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fftfreq", &args, 2)?;
    let n  = args[0].to_usize().map_err(ScriptError::Type)?;
    let sr = args[1].to_scalar().map_err(ScriptError::Type)?;
    let freqs = fftfreq(n, sr);
    let cv: CVector = Array1::from_iter(freqs.iter().map(|&f| Complex::new(f, 0.0)));
    Ok(Value::Vector(cv))
}

fn builtin_spectrum(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("spectrum", &args, 2)?;
    let x  = args[0].to_cvector().map_err(ScriptError::Type)?;
    let sr = args[1].to_scalar().map_err(ScriptError::Type)?;
    let n  = x.len();
    if n == 0 {
        return Err(ScriptError::Type("spectrum: input vector is empty".to_string()));
    }
    // DC-centered spectrum via fftshift
    let xs = fftshift(&x);
    // DC-centered frequency axis: same rotation as fftshift
    let raw_freqs: Vec<f64> = fftfreq(n, sr).to_vec();
    let split = (n + 1) / 2;
    let shifted_freqs: Vec<f64> = raw_freqs[split..].iter()
        .chain(raw_freqs[..split].iter())
        .copied()
        .collect();
    // Pack into 2×n matrix (row 0 = Hz axis, row 1 = complex spectrum)
    use ndarray::Array2;
    let mut m = Array2::zeros((2, n));
    for i in 0..n {
        m[(0, i)] = Complex::new(shifted_freqs[i], 0.0);
        m[(1, i)] = xs[i];
    }
    Ok(Value::Matrix(m))
}

// ─── Kaiser FIR builtins ───────────────────────────────────────────────────

fn builtin_fir_lowpass_kaiser(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_lowpass_kaiser", &args, 4)?;
    let cutoff = args[0].to_scalar().map_err(ScriptError::Type)?;
    let tbw    = args[1].to_scalar().map_err(ScriptError::Type)?;
    let attn   = args[2].to_scalar().map_err(ScriptError::Type)?;
    let sr     = args[3].to_scalar().map_err(ScriptError::Type)?;
    let filter = fir_lowpass_kaiser(cutoff, tbw, attn, sr)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_fir_highpass_kaiser(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_highpass_kaiser", &args, 4)?;
    let cutoff = args[0].to_scalar().map_err(ScriptError::Type)?;
    let tbw    = args[1].to_scalar().map_err(ScriptError::Type)?;
    let attn   = args[2].to_scalar().map_err(ScriptError::Type)?;
    let sr     = args[3].to_scalar().map_err(ScriptError::Type)?;
    let filter = fir_highpass_kaiser(cutoff, tbw, attn, sr)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_fir_bandpass_kaiser(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_bandpass_kaiser", &args, 5)?;
    let low    = args[0].to_scalar().map_err(ScriptError::Type)?;
    let high   = args[1].to_scalar().map_err(ScriptError::Type)?;
    let tbw    = args[2].to_scalar().map_err(ScriptError::Type)?;
    let attn   = args[3].to_scalar().map_err(ScriptError::Type)?;
    let sr     = args[4].to_scalar().map_err(ScriptError::Type)?;
    let filter = fir_bandpass_kaiser(low, high, tbw, attn, sr)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_fir_notch(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fir_notch", &args, 5)?;
    let center = args[0].to_scalar().map_err(ScriptError::Type)?;
    let bw     = args[1].to_scalar().map_err(ScriptError::Type)?;
    let sr     = args[2].to_scalar().map_err(ScriptError::Type)?;
    let taps   = args[3].to_usize().map_err(ScriptError::Type)?;
    let win    = parse_window(&args[4])?;
    let filter = fir_notch(center, bw, sr, taps, win)?;
    Ok(cvector_to_value(filter.coefficients))
}

fn builtin_freqz(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("freqz", &args, 3)?;
    let h  = args[0].to_cvector().map_err(ScriptError::Type)?;
    let n  = args[1].to_usize().map_err(ScriptError::Type)?;
    let sr = args[2].to_scalar().map_err(ScriptError::Type)?;
    let (freqs, h_out) = freqz(&h, n, sr).map_err(ScriptError::Dsp)?;
    // Return as 2×n matrix: row 0 = freq axis (real), row 1 = complex response
    use ndarray::Array2;
    let mut mat: ndarray::Array2<rustlab_core::C64> = Array2::zeros((2, n));
    for k in 0..n {
        mat[[0, k]] = Complex::new(freqs[k], 0.0);
        mat[[1, k]] = h_out[k];
    }
    Ok(Value::Matrix(mat))
}

// ─── Parks-McClellan FIR builtins ─────────────────────────────────────────

/// firpm(n_taps, bands, desired)  or  firpm(n_taps, bands, desired, weights)
///
/// bands and desired are vectors of normalized frequencies in [0,1] (1 = Nyquist).
/// weights is an optional vector with one value per band pair.
fn builtin_firpm(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.len() < 3 || args.len() > 4 {
        return Err(ScriptError::ArgCount {
            name: "firpm".into(),
            expected: 3,
            got: args.len(),
        });
    }
    let n_taps  = args[0].to_usize().map_err(ScriptError::Type)?;
    let bands   = args[1].to_cvector().map_err(ScriptError::Type)?;
    let desired = args[2].to_cvector().map_err(ScriptError::Type)?;

    let bands_f: Vec<f64>   = bands.iter().map(|c| c.re).collect();
    let desired_f: Vec<f64> = desired.iter().map(|c| c.re).collect();

    let weights_f: Vec<f64> = if args.len() == 4 {
        let w = args[3].to_cvector().map_err(ScriptError::Type)?;
        w.iter().map(|c| c.re).collect()
    } else {
        vec![]
    };

    let filter = firpm(n_taps, &bands_f, &desired_f, &weights_f)
        .map_err(ScriptError::Dsp)?;
    Ok(cvector_to_value(filter.coefficients))
}

// ─── Fixed-point quantization builtins ────────────────────────────────────

/// Parse a round-mode string, returning a ScriptError on failure.
fn parse_round_mode(s: &str) -> Result<RoundMode, ScriptError> {
    RoundMode::from_str(s).ok_or_else(|| ScriptError::Runtime(
        format!("unknown rounding mode '{s}'; valid: floor, ceil, zero, round, round_even")
    ))
}

/// Parse an overflow-mode string.
fn parse_overflow_mode(s: &str) -> Result<OverflowMode, ScriptError> {
    OverflowMode::from_str(s).ok_or_else(|| ScriptError::Runtime(
        format!("unknown overflow mode '{s}'; valid: saturate, wrap")
    ))
}

/// qfmt(word_bits, frac_bits)
/// qfmt(word_bits, frac_bits, round_mode)
/// qfmt(word_bits, frac_bits, round_mode, overflow_mode)
fn builtin_qfmt(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.len() < 2 || args.len() > 4 {
        return Err(ScriptError::ArgCount { name: "qfmt".into(), expected: 2, got: args.len() });
    }
    let word = args[0].to_usize().map_err(ScriptError::Type)? as u8;
    let frac = args[1].to_usize().map_err(ScriptError::Type)? as u8;
    let round    = if args.len() >= 3 { parse_round_mode(&args[2].to_str().map_err(ScriptError::Type)?)? }
                   else { RoundMode::Floor };
    let overflow = if args.len() == 4 { parse_overflow_mode(&args[3].to_str().map_err(ScriptError::Type)?)? }
                   else { OverflowMode::Saturate };
    let spec = QFmtSpec::new(word, frac, round, overflow).map_err(ScriptError::Dsp)?;
    Ok(Value::QFmt(spec))
}

/// quantize(x, fmt)  — snap every element of x to the Q grid defined by fmt.
/// Works on scalars, complex, vectors, and matrices (real/imag quantized independently).
fn builtin_quantize(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("quantize", &args, 2)?;
    let spec = args[1].to_qfmt().map_err(ScriptError::Type)?;

    match &args[0] {
        Value::Scalar(n) => Ok(Value::Scalar(quantize_scalar(*n, &spec))),
        Value::Complex(c) => Ok(Value::Complex(Complex::new(
            quantize_scalar(c.re, &spec),
            quantize_scalar(c.im, &spec),
        ))),
        Value::Vector(v) => {
            let re: Vec<f64> = v.iter().map(|c| quantize_scalar(c.re, &spec)).collect();
            let im: Vec<f64> = v.iter().map(|c| quantize_scalar(c.im, &spec)).collect();
            Ok(Value::Vector(Array1::from_iter(
                re.iter().zip(im.iter()).map(|(&r, &i)| Complex::new(r, i))
            )))
        }
        Value::Matrix(m) => {
            let rows = m.nrows(); let cols = m.ncols();
            let data: Vec<_> = m.iter().map(|&c| Complex::new(
                quantize_scalar(c.re, &spec),
                quantize_scalar(c.im, &spec),
            )).collect();
            Ok(Value::Matrix(Array2::from_shape_vec((rows, cols), data)
                .map_err(|e| ScriptError::Runtime(e.to_string()))?))
        }
        other => Err(ScriptError::Type(format!(
            "quantize: cannot quantize {}", other.type_name()
        ))),
    }
}

/// Extract a real f64 vector from a Value (scalar broadcast, vector, or real matrix row).
fn to_real_vec(v: &Value, name: &str) -> Result<Vec<f64>, ScriptError> {
    match v {
        Value::Scalar(n) => Ok(vec![*n]),
        Value::Vector(v) => Ok(v.iter().map(|c| c.re).collect()),
        other => Err(ScriptError::Type(format!(
            "{name}: expected real scalar or vector, got {}", other.type_name()
        ))),
    }
}

/// qadd(a, b, fmt)  — element-wise add then quantize to fmt.
fn builtin_qadd(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("qadd", &args, 3)?;
    let a    = to_real_vec(&args[0], "qadd")?;
    let b    = to_real_vec(&args[1], "qadd")?;
    let spec = args[2].to_qfmt().map_err(ScriptError::Type)?;
    let y    = fixed_qadd(&a, &b, &spec).map_err(ScriptError::Dsp)?;
    Ok(cvector_to_value(Array1::from_iter(y.iter().map(|&v| Complex::new(v, 0.0)))))
}

/// qmul(a, b, fmt)  — element-wise multiply then quantize to fmt.
fn builtin_qmul(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("qmul", &args, 3)?;
    let a    = to_real_vec(&args[0], "qmul")?;
    let b    = to_real_vec(&args[1], "qmul")?;
    let spec = args[2].to_qfmt().map_err(ScriptError::Type)?;
    let y    = fixed_qmul(&a, &b, &spec).map_err(ScriptError::Dsp)?;
    Ok(cvector_to_value(Array1::from_iter(y.iter().map(|&v| Complex::new(v, 0.0)))))
}

/// qconv(x, h, fmt)  — fixed-point FIR convolution, output quantized to fmt.
fn builtin_qconv(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("qconv", &args, 3)?;
    let x    = to_real_vec(&args[0], "qconv")?;
    let h    = to_real_vec(&args[1], "qconv")?;
    let spec = args[2].to_qfmt().map_err(ScriptError::Type)?;
    let y    = fixed_qconv(&x, &h, &spec);
    Ok(cvector_to_value(Array1::from_iter(y.iter().map(|&v| Complex::new(v, 0.0)))))
}

/// snr(x_ref, x_quantized)  — signal-to-noise ratio in dB.
fn builtin_snr(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("snr", &args, 2)?;
    let x_ref = to_real_vec(&args[0], "snr")?;
    let x_q   = to_real_vec(&args[1], "snr")?;
    let db = snr_db(&x_ref, &x_q).map_err(ScriptError::Dsp)?;
    Ok(Value::Scalar(db))
}

// ─── I/O builtins ──────────────────────────────────────────────────────────

fn builtin_print(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("print", &args, 0, 16)?;
    for (i, v) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
        print!("{}", v);
    }
    println!();
    Ok(Value::None)
}

/// Check if a CVector is "real" (all imaginary parts < 1e-10)
fn is_real_vector(v: &CVector) -> bool {
    v.iter().all(|c| c.im.abs() < 1e-10)
}

// ─── Plot options helper ────────────────────────────────────────────────────

struct PlotOpts {
    color: Option<SeriesColor>,
    label: Option<String>,
    style: LineStyle,
}

impl Default for PlotOpts {
    fn default() -> Self {
        Self { color: None, label: None, style: LineStyle::Solid }
    }
}

/// Parse trailing key-value string pairs from args slice.
/// Returns (opts, number_of_args_consumed).
fn parse_plot_opts(args: &[Value]) -> PlotOpts {
    let mut opts = PlotOpts::default();
    let mut i = 0;
    while i + 1 < args.len() {
        if let (Ok(k), Ok(v)) = (args[i].to_str(), args[i + 1].to_str()) {
            match k.to_lowercase().as_str() {
                "color" | "colour" => { opts.color = SeriesColor::parse(&v); i += 2; }
                "label" => { opts.label = Some(v); i += 2; }
                "style" => {
                    opts.style = if v.to_lowercase() == "dashed" { LineStyle::Dashed } else { LineStyle::Solid };
                    i += 2;
                }
                _ => break,
            }
        } else {
            break;
        }
    }
    opts
}

// ─── plot builtin ──────────────────────────────────────────────────────────

fn builtin_plot(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::ArgCount { name: "plot".to_string(), expected: 1, got: 0 });
    }

    // Determine if first two args are both data (x, y) or single data + options
    let (x_opt, y_val, opts_start) = match (&args[0], args.get(1)) {
        (Value::Vector(_) | Value::Matrix(_), Some(Value::Vector(_) | Value::Matrix(_))) => {
            // plot(x, y, ...) or plot(x, M, ...)
            (Some(&args[0]), &args[1], 2)
        }
        _ => {
            // plot(v, ...) or plot(M, ...)
            (None, &args[0], 1)
        }
    };

    let opts = parse_plot_opts(&args[opts_start..]);
    let label = opts.label.as_deref().unwrap_or("").to_string();

    // Title: check if last remaining string arg is not a key-value pair
    let title = {
        let rem = &args[opts_start..];
        if rem.len() == 1 {
            if let Ok(s) = rem[0].to_str() { s } else { String::new() }
        } else {
            String::new()
        }
    };

    match y_val {
        Value::Matrix(m) => {
            // Each column is a series
            let x_data: Vec<f64> = if let Some(Value::Vector(xv)) = x_opt {
                xv.iter().map(|c| c.re).collect()
            } else {
                (0..m.nrows()).map(|i| i as f64).collect()
            };
            let ncols = m.ncols();
            for col in 0..ncols {
                let y_data: Vec<f64> = m.column(col).iter().map(|c| c.re).collect();
                let col_label = if label.is_empty() { format!("col{}", col + 1) } else { label.clone() };
                let col_color = opts.color; // all columns same color if specified, else cycle
                push_xy_line(x_data.clone(), y_data, &col_label, &title, col_color, opts.style.clone());
            }
            render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))
        }
        Value::Vector(v) => {
            let x_data: Vec<f64> = if let Some(Value::Vector(xv)) = x_opt {
                xv.iter().map(|c| c.re).collect()
            } else {
                (0..v.len()).map(|i| i as f64).collect()
            };
            if is_real_vector(v) {
                let y_data: Vec<f64> = v.iter().map(|c| c.re).collect();
                let lbl = if label.is_empty() { "value" } else { label.as_str() };
                push_xy_line(x_data, y_data, lbl, &title, opts.color, opts.style);
                render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))
            } else {
                // Complex: push magnitude + real
                FIGURE.with(|fig| {
                    let mut fig = fig.borrow_mut();
                    if !fig.hold { fig.current_mut().series.clear(); }
                    let sp = fig.current_mut();
                    if !title.is_empty() && sp.title.is_empty() { sp.title = title.clone(); }
                    let mag_color = opts.color.unwrap_or(SeriesColor::Blue);
                    sp.series.push(rustlab_plot::Series {
                        label: "magnitude".to_string(),
                        x_data: x_data.clone(),
                        y_data: v.iter().map(|c| c.norm()).collect(),
                        color: mag_color,
                        style: opts.style.clone(),
                        kind: rustlab_plot::PlotKind::Line,
                    });
                    sp.series.push(rustlab_plot::Series {
                        label: "real".to_string(),
                        x_data,
                        y_data: v.iter().map(|c| c.re).collect(),
                        color: SeriesColor::Green,
                        style: opts.style.clone(),
                        kind: rustlab_plot::PlotKind::Line,
                    });
                });
                render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))
            }
        }
        Value::Scalar(n) => {
            let x_data = vec![0.0f64];
            let y_data = vec![*n];
            push_xy_line(x_data, y_data, "value", &title, opts.color, opts.style);
            render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))
        }
        other => Err(ScriptError::Type(format!("plot: cannot plot {}", other))),
    }?;
    Ok(Value::None)
}

fn builtin_stem(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::ArgCount { name: "stem".to_string(), expected: 1, got: 0 });
    }

    let (x_opt, y_val, opts_start) = match (&args[0], args.get(1)) {
        (Value::Vector(_), Some(Value::Vector(_))) => (Some(&args[0]), &args[1], 2),
        _ => (None, &args[0], 1),
    };

    let opts = parse_plot_opts(&args[opts_start..]);
    let label = opts.label.as_deref().unwrap_or("stem").to_string();
    let title = {
        let rem = &args[opts_start..];
        if rem.len() == 1 {
            if let Ok(s) = rem[0].to_str() { s } else { String::new() }
        } else {
            String::new()
        }
    };

    match y_val {
        Value::Vector(v) => {
            let x_data: Vec<f64> = if let Some(Value::Vector(xv)) = x_opt {
                xv.iter().map(|c| c.re).collect()
            } else {
                (0..v.len()).map(|i| i as f64).collect()
            };
            let y_data: Vec<f64> = v.iter().map(|c| c.re).collect();
            push_xy_stem(x_data, y_data, &label, &title, opts.color);
        }
        Value::Scalar(n) => {
            push_xy_stem(vec![0.0], vec![*n], &label, &title, opts.color);
        }
        other => return Err(ScriptError::Type(format!("stem: cannot plot {}", other))),
    }
    render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

fn builtin_plotdb(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("plotdb", &args, 1, 2)?;
    let title = if args.len() == 2 {
        args[1].to_str().map_err(ScriptError::Type)?
    } else {
        "Frequency Response".to_string()
    };
    let (freqs, h) = extract_freq_response(&args[0])?;
    plot_db(&freqs, &h, &title).map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

fn builtin_savefig(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("savefig", &args, 1, 3)?;
    // 1-arg form: savefig(path) — render current FIGURE state to file
    if args.len() == 1 {
        let path = args[0].to_str().map_err(ScriptError::Type)?;
        render_figure_file(&path).map_err(|e| ScriptError::Runtime(e.to_string()))?;
        return Ok(Value::None);
    }
    // 2–3 arg form: savefig(data, path) or savefig(data, path, title)
    let path  = args[1].to_str().map_err(ScriptError::Type)?;
    let title = if args.len() == 3 {
        args[2].to_str().map_err(ScriptError::Type)?
    } else {
        "Plot".to_string()
    };
    let real_v = to_real_vector(&args[0])?;
    save_plot(&real_v, &title, &path).map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

fn builtin_savestem(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("savestem", &args, 2, 3)?;
    let path  = args[1].to_str().map_err(ScriptError::Type)?;
    let title = if args.len() == 3 {
        args[2].to_str().map_err(ScriptError::Type)?
    } else {
        "Stem Plot".to_string()
    };
    let real_v = to_real_vector(&args[0])?;
    save_stem(&real_v, &title, &path).map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

fn builtin_savedb(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("savedb", &args, 2, 3)?;
    let path  = args[1].to_str().map_err(ScriptError::Type)?;
    let title = if args.len() == 3 {
        args[2].to_str().map_err(ScriptError::Type)?
    } else {
        "Frequency Response".to_string()
    };
    let (freqs, h) = extract_freq_response(&args[0])?;
    save_db(&freqs, &h, &title, &path).map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

// ─── Figure state builtins ─────────────────────────────────────────────────

/// figure() — reset figure state to blank.
fn builtin_figure(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("figure", &args, 0)?;
    FIGURE.with(|fig| fig.borrow_mut().reset());
    Ok(Value::None)
}

/// hold("on"|1) / hold("off"|0) — set hold on/off.
fn builtin_hold(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("hold", &args, 1)?;
    let on = match &args[0] {
        Value::Scalar(n) => *n != 0.0,
        _ => {
            let s = args[0].to_str().map_err(ScriptError::Type)?;
            s.to_lowercase() == "on" || s == "1"
        }
    };
    FIGURE.with(|fig| fig.borrow_mut().hold = on);
    Ok(Value::None)
}

/// grid("on"|1) / grid("off"|0) — enable/disable grid on current subplot.
fn builtin_grid(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("grid", &args, 1)?;
    let on = match &args[0] {
        Value::Scalar(n) => *n != 0.0,
        _ => {
            let s = args[0].to_str().map_err(ScriptError::Type)?;
            s.to_lowercase() == "on" || s == "1"
        }
    };
    FIGURE.with(|fig| fig.borrow_mut().current_mut().grid = on);
    Ok(Value::None)
}

/// xlabel("text") — set x-axis label on current subplot.
fn builtin_xlabel(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("xlabel", &args, 1)?;
    let label = args[0].to_str().map_err(ScriptError::Type)?;
    FIGURE.with(|fig| fig.borrow_mut().current_mut().xlabel = label);
    Ok(Value::None)
}

/// ylabel("text") — set y-axis label on current subplot.
fn builtin_ylabel(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("ylabel", &args, 1)?;
    let label = args[0].to_str().map_err(ScriptError::Type)?;
    FIGURE.with(|fig| fig.borrow_mut().current_mut().ylabel = label);
    Ok(Value::None)
}

/// title("text") — set title on current subplot.
fn builtin_title(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("title", &args, 1)?;
    let t = args[0].to_str().map_err(ScriptError::Type)?;
    FIGURE.with(|fig| fig.borrow_mut().current_mut().title = t);
    Ok(Value::None)
}

/// xlim([lo, hi]) — set x-axis bounds on current subplot.
fn builtin_xlim(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("xlim", &args, 1)?;
    let v = match &args[0] {
        Value::Vector(v) if v.len() >= 2 => v.clone(),
        _ => return Err(ScriptError::Type("xlim: expected [lo, hi] vector".to_string())),
    };
    FIGURE.with(|fig| fig.borrow_mut().current_mut().xlim = (Some(v[0].re), Some(v[1].re)));
    Ok(Value::None)
}

/// ylim([lo, hi]) — set y-axis bounds on current subplot.
fn builtin_ylim(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("ylim", &args, 1)?;
    let v = match &args[0] {
        Value::Vector(v) if v.len() >= 2 => v.clone(),
        _ => return Err(ScriptError::Type("ylim: expected [lo, hi] vector".to_string())),
    };
    FIGURE.with(|fig| fig.borrow_mut().current_mut().ylim = (Some(v[0].re), Some(v[1].re)));
    Ok(Value::None)
}

/// subplot(rows, cols, idx) — switch to subplot panel (1-based index).
fn builtin_subplot(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("subplot", &args, 3)?;
    let rows = args[0].to_usize().map_err(ScriptError::Type)?;
    let cols = args[1].to_usize().map_err(ScriptError::Type)?;
    let idx  = args[2].to_usize().map_err(ScriptError::Type)?;
    FIGURE.with(|fig| fig.borrow_mut().set_subplot(rows, cols, idx));
    Ok(Value::None)
}

/// legend("s1", "s2", ...) — retroactively label series in current subplot.
fn builtin_legend(args: Vec<Value>) -> Result<Value, ScriptError> {
    // legend() — enable legend using series labels already set via plot(..., "label", "name")
    // legend("l1", "l2", ...) — override series labels in order
    if !args.is_empty() {
        let labels: Vec<String> = args.iter()
            .map(|a| a.to_str().unwrap_or_default())
            .collect();
        FIGURE.with(|fig| {
            let mut fig = fig.borrow_mut();
            let sp = fig.current_mut();
            for (i, label) in labels.iter().enumerate() {
                if i < sp.series.len() {
                    sp.series[i].label = label.clone();
                }
            }
        });
    }
    Ok(Value::None)
}

/// imagesc(M) / imagesc(M, colormap) — display matrix as heatmap.
fn builtin_imagesc(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("imagesc", &args, 1, 2)?;
    let colormap = if args.len() == 2 {
        args[1].to_str().map_err(ScriptError::Type)?
    } else {
        "viridis".to_string()
    };
    let matrix = match &args[0] {
        Value::Matrix(m) => m.clone(),
        Value::Vector(v) => {
            // Treat as column vector matrix
            let n = v.len();
            ndarray::Array2::from_shape_fn((n, 1), |(i, _)| v[i])
        }
        other => return Err(ScriptError::Type(format!("imagesc: expected matrix, got {}", other))),
    };
    imagesc_terminal(&matrix, "", &colormap)
        .map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

/// saveimagesc(M, path) / saveimagesc(M, path, title) / saveimagesc(M, path, title, colormap)
fn builtin_saveimagesc(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("saveimagesc", &args, 2, 4)?;
    let matrix = match &args[0] {
        Value::Matrix(m) => m.clone(),
        Value::Vector(v) => {
            let n = v.len();
            ndarray::Array2::from_shape_fn((n, 1), |(i, _)| v[i])
        }
        other => return Err(ScriptError::Type(format!("saveimagesc: expected matrix, got {}", other))),
    };
    let path = args[1].to_str().map_err(ScriptError::Type)?;
    let title = if args.len() >= 3 { args[2].to_str().map_err(ScriptError::Type)? } else { String::new() };
    let colormap = if args.len() >= 4 { args[3].to_str().map_err(ScriptError::Type)? } else { "viridis".to_string() };
    save_imagesc_cmap(&matrix, &title, &colormap, &path)
        .map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

/// Extract (freqs: RVector, H: CVector) from a 2×n Matrix (as returned by freqz).
fn extract_freq_response(val: &Value) -> Result<(rustlab_core::RVector, CVector), ScriptError> {
    match val {
        Value::Matrix(m) => {
            if m.nrows() < 2 {
                return Err(ScriptError::Type(
                    "plotdb/savedb: expected a 2×n matrix from freqz".to_string()
                ));
            }
            let freqs = ndarray::Array1::from_iter(m.row(0).iter().map(|c| c.re));
            let h     = ndarray::Array1::from_iter(m.row(1).iter().copied());
            Ok((freqs, h))
        }
        other => Err(ScriptError::Type(format!(
            "plotdb/savedb: expected matrix from freqz, got {other}"
        ))),
    }
}

/// Extract the real part of any numeric Value as an RVector.
fn to_real_vector(val: &Value) -> Result<rustlab_core::RVector, ScriptError> {
    match val {
        Value::Vector(v) => Ok(ndarray::Array1::from_iter(v.iter().map(|c| c.re))),
        Value::Scalar(n) => Ok(ndarray::Array1::from_vec(vec![*n])),
        other => Err(ScriptError::Type(format!("savefig: cannot plot {other}"))),
    }
}

// ─── Save / Load / whos builtins ──────────────────────────────────────────

// ── NPY helpers ────────────────────────────────────────────────────────────

/// Flatten a Value into (data, shape) for NPY serialisation.
fn value_to_c64_array(val: &Value) -> Result<(Vec<Complex<f64>>, Vec<usize>), String> {
    match val {
        Value::Scalar(n)  => Ok((vec![Complex::new(*n, 0.0)], vec![1])),
        Value::Complex(c) => Ok((vec![*c], vec![1])),
        Value::Vector(v)  => Ok((v.iter().copied().collect(), vec![v.len()])),
        Value::Matrix(m)  => {
            // ndarray Array2 is row-major (C order) — iter() gives row-major order
            let data: Vec<Complex<f64>> = m.iter().copied().collect();
            Ok((data, vec![m.nrows(), m.ncols()]))
        }
        other => Err(format!("save: cannot serialise {} to NPY", other)),
    }
}

/// Build the raw bytes of an NPY v1.0 file.
fn build_npy_bytes(data: &[Complex<f64>], shape: &[usize]) -> Vec<u8> {
    let real_only = data.iter().all(|c| c.im.abs() < 1e-12);
    let descr = if real_only { "<f8" } else { "<c16" };

    let shape_str = match shape {
        [n]       => format!("({n},)"),
        [r, c]    => format!("({r}, {c})"),
        other     => {
            let parts: Vec<String> = other.iter().map(|d| d.to_string()).collect();
            format!("({})", parts.join(", "))
        }
    };
    let raw = format!(
        "{{'descr': '{descr}', 'fortran_order': False, 'shape': {shape_str}, }}"
    );

    // Total = 10 (prefix) + header_len; must be divisible by 64.
    let needed = 10 + raw.len() + 1; // +1 for the trailing '\n'
    let padded = ((needed + 63) / 64) * 64;
    let header = format!("{}{}\n", raw, " ".repeat(padded - needed));
    let hlen   = header.len() as u16;

    let mut out = Vec::with_capacity(padded + data.len() * if real_only { 8 } else { 16 });
    out.extend_from_slice(b"\x93NUMPY");
    out.push(1);
    out.push(0);
    out.extend_from_slice(&hlen.to_le_bytes());
    out.extend_from_slice(header.as_bytes());
    if real_only {
        for c in data { out.extend_from_slice(&c.re.to_le_bytes()); }
    } else {
        for c in data {
            out.extend_from_slice(&c.re.to_le_bytes());
            out.extend_from_slice(&c.im.to_le_bytes());
        }
    }
    out
}

/// Parse the shape tuple from an NPY header string.
fn parse_npy_shape(header: &str) -> Result<Vec<usize>, String> {
    let key = header.find("'shape':")
        .or_else(|| header.find("\"shape\":"))
        .ok_or_else(|| "NPY header missing 'shape' field".to_string())?;
    let after = &header[key..];
    let open  = after.find('(').ok_or_else(|| "NPY header: bad shape (no '(')".to_string())?;
    let close = after.find(')').ok_or_else(|| "NPY header: bad shape (no ')')".to_string())?;
    let inner = after[open + 1..close].trim();
    if inner.is_empty() {
        return Ok(vec![]); // 0-d array
    }
    inner
        .split(',')
        .filter_map(|s| { let t = s.trim(); if t.is_empty() { None } else { Some(t.parse::<usize>()) } })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("NPY shape parse error: {e}"))
}

/// Reconstruct a Value from a flat array + shape.
fn array_to_value(values: Vec<Complex<f64>>, shape: &[usize]) -> Result<Value, String> {
    match shape {
        [] | [1] => {
            let c = *values.first().ok_or("NPY: empty array")?;
            if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }
        }
        [_n] => Ok(Value::Vector(Array1::from_vec(values))),
        [nrows, ncols] => {
            let mat = Array2::from_shape_vec((*nrows, *ncols), values)
                .map_err(|e| e.to_string())?;
            Ok(Value::Matrix(mat))
        }
        other => Err(format!("NPY: unsupported shape rank {}", other.len())),
    }
}

/// Parse an in-memory NPY byte buffer into a Value.
fn parse_npy_bytes(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() < 10 || &bytes[0..6] != b"\x93NUMPY" {
        return Err("not a valid NPY file".to_string());
    }
    let hlen  = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let hend  = 10 + hlen;
    if bytes.len() < hend {
        return Err("NPY file truncated in header".to_string());
    }
    let header = std::str::from_utf8(&bytes[10..hend]).map_err(|e| e.to_string())?;
    let is_c16 = header.contains("<c16") || header.contains(">c16");
    let is_f8  = header.contains("<f8")  || header.contains(">f8");
    let shape  = parse_npy_shape(header)?;
    let data   = &bytes[hend..];

    if is_c16 {
        if data.len() % 16 != 0 {
            return Err("NPY complex128: data length is not a multiple of 16".to_string());
        }
        let values: Vec<Complex<f64>> = (0..data.len() / 16).map(|i| {
            let re = f64::from_le_bytes(data[i*16     ..i*16 +  8].try_into().unwrap());
            let im = f64::from_le_bytes(data[i*16 + 8 ..i*16 + 16].try_into().unwrap());
            Complex::new(re, im)
        }).collect();
        array_to_value(values, &shape)
    } else if is_f8 {
        if data.len() % 8 != 0 {
            return Err("NPY float64: data length is not a multiple of 8".to_string());
        }
        let values: Vec<Complex<f64>> = (0..data.len() / 8).map(|i| {
            let f = f64::from_le_bytes(data[i*8..i*8+8].try_into().unwrap());
            Complex::new(f, 0.0)
        }).collect();
        array_to_value(values, &shape)
    } else {
        Err(format!(
            "unsupported NPY dtype (only <f8 and <c16 are supported): {}",
            header.chars().take(100).collect::<String>()
        ))
    }
}

// ── CSV helpers ────────────────────────────────────────────────────────────

fn fmt_csv_cell(c: Complex<f64>) -> String {
    if c.im.abs() < 1e-12 {
        format!("{}", c.re)
    } else if c.im >= 0.0 {
        format!("{}+{}i", c.re, c.im)
    } else {
        format!("{}{}i", c.re, c.im) // im already negative
    }
}

/// Parse a single CSV cell as a real or complex number.
fn parse_csv_cell(s: &str) -> Result<Complex<f64>, String> {
    let s = s.trim();
    // No imaginary suffix → plain real
    if !s.ends_with('i') && !s.ends_with('j') {
        return s.parse::<f64>()
            .map(|f| Complex::new(f, 0.0))
            .map_err(|_| format!("cannot parse '{}' as a number", s));
    }
    // Strip 'i'/'j' suffix and find the split between re and im parts.
    let body  = &s[..s.len() - 1];
    let bytes = body.as_bytes();
    // Scan right-to-left for + or - that is not the very first character
    let split = (1..bytes.len()).rev().find(|&i| bytes[i] == b'+' || bytes[i] == b'-');
    if let Some(i) = split {
        let re: f64 = body[..i].parse()
            .map_err(|_| format!("invalid real part in '{}'", s))?;
        let im: f64 = match &body[i..] {
            "+" => 1.0,
            "-" => -1.0,
            t   => t.parse().map_err(|_| format!("invalid imaginary part in '{}'", s))?,
        };
        Ok(Complex::new(re, im))
    } else {
        // Pure imaginary: body is e.g. "2.5" or "-2.5"
        let im: f64 = match body {
            "" | "+" => 1.0,
            "-"      => -1.0,
            t        => t.parse().map_err(|_| format!("cannot parse imaginary '{}' in '{}'", t, s))?,
        };
        Ok(Complex::new(0.0, im))
    }
}

fn write_csv(path: &str, val: &Value) -> Result<(), String> {
    use std::io::Write;
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::new(file);
    match val {
        Value::Scalar(n)  => writeln!(w, "{n}").map_err(|e| e.to_string())?,
        Value::Complex(c) => writeln!(w, "{}", fmt_csv_cell(*c)).map_err(|e| e.to_string())?,
        Value::Vector(v)  => {
            for c in v.iter() {
                writeln!(w, "{}", fmt_csv_cell(*c)).map_err(|e| e.to_string())?;
            }
        }
        Value::Matrix(m)  => {
            for r in 0..m.nrows() {
                for ci in 0..m.ncols() {
                    if ci > 0 { write!(w, ",").map_err(|e| e.to_string())?; }
                    write!(w, "{}", fmt_csv_cell(m[[r, ci]])).map_err(|e| e.to_string())?;
                }
                writeln!(w).map_err(|e| e.to_string())?;
            }
        }
        other => return Err(format!("save: cannot serialise {} to CSV", other)),
    }
    Ok(())
}

fn load_csv(path: &str) -> Result<Value, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Ok(Value::Vector(Array1::zeros(0)));
    }
    let mut rows: Vec<Vec<Complex<f64>>> = Vec::with_capacity(lines.len());
    for line in &lines {
        let cells: Result<Vec<_>, _> = line.split(',').map(parse_csv_cell).collect();
        rows.push(cells.map_err(|e| format!("CSV parse error: {e}"))?);
    }
    let ncols = rows[0].len();
    if rows.iter().any(|r| r.len() != ncols) {
        return Err("CSV load: rows have inconsistent column counts".to_string());
    }
    match (rows.len(), ncols) {
        (1, 1) => {
            let c = rows[0][0];
            if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }
        }
        (_, 1) => {
            // Column vector
            Ok(Value::Vector(Array1::from_vec(rows.into_iter().map(|r| r[0]).collect())))
        }
        (1, _) => {
            // Row vector
            Ok(Value::Vector(Array1::from_vec(rows.into_iter().next().unwrap())))
        }
        (nrows, ncols) => {
            let flat: Vec<Complex<f64>> = rows.into_iter().flatten().collect();
            let mat = Array2::from_shape_vec((nrows, ncols), flat).map_err(|e| e.to_string())?;
            Ok(Value::Matrix(mat))
        }
    }
}

// ── NPZ helpers ────────────────────────────────────────────────────────────

fn save_npz(path: &str, pairs: &[Value]) -> Result<(), String> {
    use zip::write::{ZipWriter, SimpleFileOptions};
    use std::io::Write;

    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for chunk in pairs.chunks(2) {
        let name = chunk[0].to_str().map_err(|e| format!("save NPZ: {e}"))?;
        let (data, shape) = value_to_c64_array(&chunk[1])?;
        let npy = build_npy_bytes(&data, &shape);
        zip.start_file(format!("{name}.npy"), opts).map_err(|e| e.to_string())?;
        zip.write_all(&npy).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Load all variables from an NPZ file. Returns (var_name, value) pairs in zip order.
pub fn load_all_from_npz(path: &str) -> Result<Vec<(String, Value)>, String> {
    use zip::ZipArchive;
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let entry_name = entry.name().to_string();
        let var_name = entry_name.strip_suffix(".npy").unwrap_or(&entry_name).to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        result.push((var_name, parse_npy_bytes(&buf)?));
    }
    Ok(result)
}

fn load_from_npz(path: &str, name: &str) -> Result<Value, String> {
    use zip::ZipArchive;
    use std::io::Read;

    let file  = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let entry_name = format!("{name}.npy");
    let mut entry = zip.by_name(&entry_name)
        .map_err(|_| format!("'{}' not found in {}", name, path))?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    parse_npy_bytes(&buf)
}

// ── Builtins ───────────────────────────────────────────────────────────────

fn builtin_save(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.len() < 2 {
        return Err(ScriptError::Type(
            "save: usage:\n  save(\"file.npy\", x)\n  save(\"file.csv\", x)\n  save(\"file.npz\", \"name1\", x1, \"name2\", x2, ...)".to_string()
        ));
    }
    let path = args[0].to_str().map_err(ScriptError::Type)?;

    if path.ends_with(".npz") {
        let pairs = &args[1..];
        if pairs.is_empty() || pairs.len() % 2 != 0 {
            return Err(ScriptError::Type(
                "save: NPZ requires alternating name/value pairs after the filename".to_string()
            ));
        }
        save_npz(&path, pairs).map_err(ScriptError::Runtime)?;
    } else if path.ends_with(".csv") {
        if args.len() != 2 {
            return Err(ScriptError::Type("save: CSV format takes exactly one value".to_string()));
        }
        write_csv(&path, &args[1]).map_err(ScriptError::Runtime)?;
    } else {
        // .npy (or any other extension — default to NPY)
        if args.len() != 2 {
            return Err(ScriptError::Type("save: NPY format takes exactly one value".to_string()));
        }
        let (data, shape) = value_to_c64_array(&args[1]).map_err(ScriptError::Runtime)?;
        let bytes = build_npy_bytes(&data, &shape);
        std::fs::write(&path, bytes).map_err(|e| ScriptError::Runtime(e.to_string()))?;
    }
    Ok(Value::None)
}

fn builtin_load(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("load", &args, 1, 2)?;
    let path = args[0].to_str().map_err(ScriptError::Type)?;

    if path.ends_with(".npz") {
        if args.len() != 2 {
            return Err(ScriptError::Type(
                "load: to load all variables use bare load(\"file.npz\") without assignment;\n  to extract one use: x = load(\"file.npz\", \"varname\")".to_string()
            ));
        }
        let name = args[1].to_str().map_err(ScriptError::Type)?;
        load_from_npz(&path, &name).map_err(ScriptError::Runtime)
    } else if path.ends_with(".csv") {
        load_csv(&path).map_err(ScriptError::Runtime)
    } else {
        // .npy or any other extension
        let bytes = std::fs::read(&path).map_err(|e| ScriptError::Runtime(e.to_string()))?;
        parse_npy_bytes(&bytes).map_err(ScriptError::Runtime)
    }
}

// ─── Matrix construction ───────────────────────────────────────────────────

/// eye(n) — n×n identity matrix
fn builtin_eye(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("eye", &args, 1)?;
    let n = args[0].to_usize().map_err(ScriptError::Type)?;
    let mut m: CMatrix = Array2::zeros((n, n));
    for i in 0..n {
        m[[i, i]] = Complex::new(1.0, 0.0);
    }
    Ok(Value::Matrix(m))
}

// ─── Matrix operations ─────────────────────────────────────────────────────

/// transpose(A) — non-conjugate transpose (function form of `.'`)
fn builtin_transpose(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("transpose", &args, 1)?;
    args.into_iter().next().unwrap()
        .non_conj_transpose()
        .map_err(ScriptError::Type)
}

/// diag(v)    — create diagonal matrix from vector v
/// diag(M)    — extract main diagonal of matrix M as a vector
/// diag(M, k) — extract k-th diagonal (k>0 superdiagonal, k<0 subdiagonal)
fn builtin_diag(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("diag", &args, 1, 2)?;
    let k: i64 = if args.len() == 2 {
        args[1].to_scalar().map_err(ScriptError::Type)? as i64
    } else { 0 };

    match &args[0] {
        Value::Vector(v) => {
            // Create diagonal matrix
            let n = v.len();
            let size = n + k.unsigned_abs() as usize;
            let mut m: CMatrix = Array2::zeros((size, size));
            for (i, &val) in v.iter().enumerate() {
                let (r, c) = if k >= 0 { (i, i + k as usize) } else { (i + (-k) as usize, i) };
                m[[r, c]] = val;
            }
            Ok(Value::Matrix(m))
        }
        Value::Matrix(m) => {
            // Extract diagonal
            let nrows = m.nrows() as i64;
            let ncols = m.ncols() as i64;
            let len = if k >= 0 {
                (ncols - k).max(0).min(nrows) as usize
            } else {
                (nrows + k).max(0).min(ncols) as usize
            };
            let diag: CVector = Array1::from_iter((0..len).map(|i| {
                let (r, c) = if k >= 0 { (i, i + k as usize) } else { (i + (-k) as usize, i) };
                m[[r, c]]
            }));
            Ok(Value::Vector(diag))
        }
        other => Err(ScriptError::Type(format!("diag: expected vector or matrix, got {}", other.type_name()))),
    }
}

/// trace(M) — sum of main diagonal
fn builtin_trace(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("trace", &args, 1)?;
    match &args[0] {
        Value::Matrix(m) => {
            let n = m.nrows().min(m.ncols());
            let t: C64 = (0..n).map(|i| m[[i, i]]).sum();
            if t.im.abs() < 1e-12 { Ok(Value::Scalar(t.re)) } else { Ok(Value::Complex(t)) }
        }
        Value::Scalar(n) => Ok(Value::Scalar(*n)),
        other => Err(ScriptError::Type(format!("trace: expected matrix, got {}", other.type_name()))),
    }
}

/// reshape(A, m, n) — reshape A (vector or matrix) into an m×n matrix (column-major order)
fn builtin_reshape(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("reshape", &args, 3)?;
    let m = args[1].to_usize().map_err(ScriptError::Type)?;
    let n = args[2].to_usize().map_err(ScriptError::Type)?;
    let flat: Vec<C64> = match &args[0] {
        Value::Vector(v) => v.iter().copied().collect(),
        Value::Matrix(mat) => {
            // column-major order (standard for matrix languages): collect column by column
            (0..mat.ncols()).flat_map(|c| (0..mat.nrows()).map(move |r| mat[[r, c]])).collect()
        }
        Value::Scalar(s) => vec![Complex::new(*s, 0.0)],
        Value::Complex(c) => vec![*c],
        other => return Err(ScriptError::Type(format!("reshape: cannot reshape {}", other.type_name()))),
    };
    if flat.len() != m * n {
        return Err(ScriptError::Type(format!(
            "reshape: cannot reshape {} elements into {}×{} (= {} elements)",
            flat.len(), m, n, m * n
        )));
    }
    if m == 1 || n == 1 {
        Ok(Value::Vector(Array1::from_vec(flat)))
    } else {
        // Build matrix column-major: element [i*n+j] comes from flat[r + c*m] in col-major
        let mut mat: CMatrix = Array2::zeros((m, n));
        for c in 0..n {
            for r in 0..m {
                mat[[r, c]] = flat[r + c * m];
            }
        }
        Ok(Value::Matrix(mat))
    }
}

/// repmat(A, m, n) — tile matrix A m times vertically, n times horizontally
fn builtin_repmat(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("repmat", &args, 3)?;
    let reps_r = args[1].to_usize().map_err(ScriptError::Type)?;
    let reps_c = args[2].to_usize().map_err(ScriptError::Type)?;
    // Normalise to a matrix block
    let block: CMatrix = match &args[0] {
        Value::Matrix(m) => m.clone(),
        Value::Vector(v) => {
            let n = v.len();
            let data: Vec<C64> = v.iter().copied().collect();
            Array2::from_shape_vec((1, n), data).map_err(|e| ScriptError::Type(e.to_string()))?
        }
        Value::Scalar(s) => Array2::from_elem((1, 1), Complex::new(*s, 0.0)),
        Value::Complex(c) => Array2::from_elem((1, 1), *c),
        other => return Err(ScriptError::Type(format!("repmat: cannot tile {}", other.type_name()))),
    };
    let br = block.nrows();
    let bc = block.ncols();
    let out_r = br * reps_r;
    let out_c = bc * reps_c;
    let mut out: CMatrix = Array2::zeros((out_r, out_c));
    for ri in 0..reps_r {
        for ci in 0..reps_c {
            let r0 = ri * br;
            let c0 = ci * bc;
            for r in 0..br {
                for c in 0..bc {
                    out[[r0 + r, c0 + c]] = block[[r, c]];
                }
            }
        }
    }
    Ok(Value::Matrix(out))
}

/// horzcat(A, B, ...) — horizontal concatenation (same as [A B])
fn builtin_horzcat(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() {
        return Ok(Value::Vector(Array1::zeros(0)));
    }
    Value::from_matrix_rows(vec![args]).map_err(ScriptError::Type)
}

/// vertcat(A, B, ...) — vertical concatenation (same as [A; B])
fn builtin_vertcat(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() {
        return Ok(Value::Vector(Array1::zeros(0)));
    }
    Value::from_matrix_rows(args.into_iter().map(|v| vec![v]).collect())
        .map_err(ScriptError::Type)
}

// ─── Linear algebra ────────────────────────────────────────────────────────

/// dot(u, v) — inner (dot) product of two vectors
fn builtin_dot(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("dot", &args, 2)?;
    let u = args[0].to_cvector().map_err(ScriptError::Type)?;
    let v = args[1].to_cvector().map_err(ScriptError::Type)?;
    if u.len() != v.len() {
        return Err(ScriptError::Type(format!(
            "dot: vectors must have the same length ({} vs {})", u.len(), v.len()
        )));
    }
    let result: C64 = u.iter().zip(v.iter()).map(|(&a, &b)| a * b).sum();
    if result.im.abs() < 1e-12 { Ok(Value::Scalar(result.re)) } else { Ok(Value::Complex(result)) }
}

/// cross(u, v) — 3D cross product
fn builtin_cross(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("cross", &args, 2)?;
    let u = args[0].to_cvector().map_err(ScriptError::Type)?;
    let v = args[1].to_cvector().map_err(ScriptError::Type)?;
    if u.len() != 3 || v.len() != 3 {
        return Err(ScriptError::Type(format!(
            "cross: both vectors must have length 3 (got {} and {})", u.len(), v.len()
        )));
    }
    let result = Array1::from_vec(vec![
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]);
    Ok(Value::Vector(result))
}

/// norm(v)    — Euclidean (L2) norm of a vector, or Frobenius norm of a matrix
/// norm(v, p) — p-norm (p=1 or p=2 supported; p="fro" for Frobenius)
fn builtin_norm(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("norm", &args, 1, 2)?;
    match &args[0] {
        Value::Vector(v) => {
            let p: f64 = if args.len() == 2 { args[1].to_scalar().map_err(ScriptError::Type)? } else { 2.0 };
            let n = if p == 1.0 {
                v.iter().map(|c| c.norm()).sum::<f64>()
            } else if p == 2.0 {
                v.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt()
            } else if p == f64::INFINITY {
                v.iter().map(|c| c.norm()).fold(0.0_f64, f64::max)
            } else {
                v.iter().map(|c| c.norm().powf(p)).sum::<f64>().powf(1.0 / p)
            };
            Ok(Value::Scalar(n))
        }
        Value::Matrix(m) => {
            // Frobenius norm by default
            let n = m.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
            Ok(Value::Scalar(n))
        }
        Value::Scalar(n) => Ok(Value::Scalar(n.abs())),
        Value::Complex(c) => Ok(Value::Scalar(c.norm())),
        other => Err(ScriptError::Type(format!("norm: unsupported type {}", other.type_name()))),
    }
}

/// LU decomposition with partial pivoting.
/// Returns (L*U after in-place elimination, sign of permutation).
fn lu_decompose(m: &CMatrix) -> (CMatrix, C64) {
    let n = m.nrows();
    let mut lu = m.to_owned();
    let mut sign = Complex::new(1.0, 0.0);
    for k in 0..n {
        // Partial pivoting
        let mut max_idx = k;
        let mut max_val = lu[[k, k]].norm();
        for i in k + 1..n {
            let v = lu[[i, k]].norm();
            if v > max_val { max_val = v; max_idx = i; }
        }
        if max_idx != k {
            for j in 0..n {
                let tmp = lu[[k, j]];
                lu[[k, j]] = lu[[max_idx, j]];
                lu[[max_idx, j]] = tmp;
            }
            sign = -sign;
        }
        let pivot = lu[[k, k]];
        if pivot.norm() < 1e-14 { continue; }
        for i in k + 1..n {
            lu[[i, k]] /= pivot;
            for j in k + 1..n {
                let sub = lu[[i, k]] * lu[[k, j]];
                lu[[i, j]] -= sub;
            }
        }
    }
    (lu, sign)
}

/// det(M) — determinant of a square matrix
fn builtin_det(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("det", &args, 1)?;
    match &args[0] {
        Value::Matrix(m) => {
            let n = m.nrows();
            if n != m.ncols() {
                return Err(ScriptError::Type(format!(
                    "det: matrix must be square (got {}×{})", n, m.ncols()
                )));
            }
            if n == 0 { return Ok(Value::Scalar(1.0)); }
            if n == 1 { let c = m[[0,0]]; return if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }; }
            let (lu, sign) = lu_decompose(m);
            let d: C64 = sign * (0..n).map(|i| lu[[i, i]]).product::<C64>();
            if d.im.abs() < 1e-12 { Ok(Value::Scalar(d.re)) } else { Ok(Value::Complex(d)) }
        }
        Value::Scalar(n) => Ok(Value::Scalar(*n)),
        other => Err(ScriptError::Type(format!("det: expected matrix, got {}", other.type_name()))),
    }
}

/// inv(M) — inverse of a square matrix via Gauss-Jordan elimination
fn matrix_inv(m: &CMatrix) -> Result<CMatrix, String> {
    let n = m.nrows();
    if n != m.ncols() {
        return Err(format!("inv: matrix must be square (got {}×{})", n, m.ncols()));
    }
    // Augmented [A | I]
    let mut aug: Array2<C64> = Array2::zeros((n, 2 * n));
    for i in 0..n {
        for j in 0..n { aug[[i, j]] = m[[i, j]]; }
        aug[[i, n + i]] = Complex::new(1.0, 0.0);
    }
    for k in 0..n {
        // Pivot
        let mut max_idx = k;
        let mut max_val = aug[[k, k]].norm();
        for i in k + 1..n {
            let v = aug[[i, k]].norm();
            if v > max_val { max_val = v; max_idx = i; }
        }
        if max_idx != k {
            for j in 0..2 * n {
                let tmp = aug[[k, j]];
                aug[[k, j]] = aug[[max_idx, j]];
                aug[[max_idx, j]] = tmp;
            }
        }
        if aug[[k, k]].norm() < 1e-14 {
            return Err("inv: matrix is singular or nearly singular".to_string());
        }
        let pivot = aug[[k, k]];
        for j in 0..2 * n { aug[[k, j]] /= pivot; }
        for i in 0..n {
            if i != k {
                let factor = aug[[i, k]];
                for j in 0..2 * n {
                    let sub = factor * aug[[k, j]];
                    aug[[i, j]] -= sub;
                }
            }
        }
    }
    let mut result: CMatrix = Array2::zeros((n, n));
    for i in 0..n {
        for j in 0..n { result[[i, j]] = aug[[i, n + j]]; }
    }
    Ok(result)
}

fn builtin_inv(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("inv", &args, 1)?;
    match &args[0] {
        Value::Matrix(m) => {
            let result = matrix_inv(m).map_err(ScriptError::Type)?;
            Ok(Value::Matrix(result))
        }
        Value::Scalar(n) => {
            if *n == 0.0 { return Err(ScriptError::Type("inv: singular (scalar is zero)".to_string())); }
            Ok(Value::Scalar(1.0 / n))
        }
        other => Err(ScriptError::Type(format!("inv: expected matrix, got {}", other.type_name()))),
    }
}

/// linsolve(A, b) — solve the linear system A*x = b via Gaussian elimination
fn builtin_linsolve(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("linsolve", &args, 2)?;
    let a = match &args[0] {
        Value::Matrix(m) => m.clone(),
        other => return Err(ScriptError::Type(format!("linsolve: A must be a matrix, got {}", other.type_name()))),
    };
    let b = args[1].to_cvector().map_err(ScriptError::Type)?;
    let n = a.nrows();
    if n != a.ncols() {
        return Err(ScriptError::Type(format!(
            "linsolve: A must be square (got {}×{})", n, a.ncols()
        )));
    }
    if n != b.len() {
        return Err(ScriptError::Type(format!(
            "linsolve: A is {}×{} but b has length {}", n, n, b.len()
        )));
    }
    // Augmented [A | b]
    let mut aug: Array2<C64> = Array2::zeros((n, n + 1));
    for i in 0..n {
        for j in 0..n { aug[[i, j]] = a[[i, j]]; }
        aug[[i, n]] = b[i];
    }
    // Forward elimination with partial pivoting
    for k in 0..n {
        let mut max_idx = k;
        let mut max_val = aug[[k, k]].norm();
        for i in k + 1..n {
            let v = aug[[i, k]].norm();
            if v > max_val { max_val = v; max_idx = i; }
        }
        if max_idx != k {
            for j in 0..n + 1 {
                let tmp = aug[[k, j]];
                aug[[k, j]] = aug[[max_idx, j]];
                aug[[max_idx, j]] = tmp;
            }
        }
        if aug[[k, k]].norm() < 1e-14 {
            return Err(ScriptError::Type("linsolve: matrix is singular or nearly singular".to_string()));
        }
        for i in k + 1..n {
            let factor = aug[[i, k]] / aug[[k, k]];
            for j in k..n + 1 {
                let sub = factor * aug[[k, j]];
                aug[[i, j]] -= sub;
            }
        }
    }
    // Back substitution
    let mut x: CVector = Array1::zeros(n);
    for i in (0..n).rev() {
        let mut s = aug[[i, n]];
        for j in i + 1..n { s -= aug[[i, j]] * x[j]; }
        x[i] = s / aug[[i, i]];
    }
    // Return as scalar if 1-element, else vector
    if x.len() == 1 {
        let c = x[0];
        if c.im.abs() < 1e-12 { Ok(Value::Scalar(c.re)) } else { Ok(Value::Complex(c)) }
    } else {
        Ok(Value::Vector(x))
    }
}

// ─── factor(n) ────────────────────────────────────────────────────────────────

/// factor(n) — prime factorization of a positive integer.
/// Returns a real Vector of prime factors in ascending order (with repetition).
/// factor(12) → [2, 2, 3],  factor(17) → [17],  factor(1) → []
fn builtin_factor(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("factor", &args, 1)?;
    let n_f = match &args[0] {
        Value::Scalar(n) => *n,
        other => return Err(ScriptError::Type(format!(
            "factor: expected a positive integer scalar, got {}", other.type_name()
        ))),
    };
    if n_f <= 0.0 || n_f.fract() != 0.0 {
        return Err(ScriptError::Type(format!(
            "factor: argument must be a positive integer, got {}", n_f
        )));
    }
    let mut n = n_f as u64;
    let mut factors: Vec<C64> = Vec::new();
    let mut d = 2u64;
    while d * d <= n {
        while n % d == 0 {
            factors.push(Complex::new(d as f64, 0.0));
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        factors.push(Complex::new(n as f64, 0.0));
    }
    Ok(Value::Vector(Array1::from_vec(factors)))
}

// ─── eig(M) ───────────────────────────────────────────────────────────────────

/// Reduce a square matrix to upper Hessenberg form via Householder reflectors.
/// Returns H such that H has the same eigenvalues as the input.
fn hessenberg_reduce(a: &CMatrix) -> CMatrix {
    let n = a.nrows();
    let mut h = a.to_owned();
    for k in 0..n.saturating_sub(2) {
        // Build Householder vector from column k below the subdiagonal
        let col_len = n - k - 1;
        let mut x: Vec<C64> = (0..col_len).map(|i| h[[k + 1 + i, k]]).collect();
        let norm_x: f64 = x.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
        if norm_x < 1e-15 { continue; }
        // Phase of first element
        let phase = if x[0].norm() < 1e-15 {
            Complex::new(1.0, 0.0)
        } else {
            x[0] / x[0].norm()
        };
        x[0] += phase * norm_x;
        let norm_v: f64 = x.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
        if norm_v < 1e-15 { continue; }
        for c in &mut x { *c /= norm_v; }
        // H = (I - 2 v v*) H (I - 2 v v*)  — apply from left then right
        // Left: H[k+1:, k:] -= 2 * v * (v* H[k+1:, k:])
        for j in k..n {
            let dot: C64 = x.iter().enumerate().map(|(i, vi)| vi.conj() * h[[k+1+i, j]]).sum();
            for i in 0..col_len { h[[k+1+i, j]] -= 2.0 * x[i] * dot; }
        }
        // Right: H[:, k+1:] -= 2 * (H[:, k+1:] v) v*
        for i in 0..n {
            let dot: C64 = x.iter().enumerate().map(|(j, vj)| h[[i, k+1+j]] * *vj).sum();
            for j in 0..col_len { h[[i, k+1+j]] -= 2.0 * dot * x[j].conj(); }
        }
    }
    h
}

/// Compute eigenvalues of an upper Hessenberg matrix using shifted QR iteration.
/// Uses complex Wilkinson shifts for reliable convergence.
/// Returns eigenvalues as a Vec<C64>.
fn eig_hessenberg(h_in: &CMatrix) -> Result<Vec<C64>, String> {
    let n = h_in.nrows();
    if n == 0 { return Ok(vec![]); }
    if n == 1 { return Ok(vec![h_in[[0,0]]]); }
    if n == 2 {
        // Direct 2×2 formula
        let a = h_in[[0,0]]; let b = h_in[[0,1]];
        let c = h_in[[1,0]]; let d = h_in[[1,1]];
        let tr = a + d;
        let det = a * d - b * c;
        let disc = (tr * tr - 4.0 * det).sqrt();
        return Ok(vec![(tr + disc) / 2.0, (tr - disc) / 2.0]);
    }

    let mut h = h_in.to_owned();
    let mut eigenvalues: Vec<C64> = Vec::with_capacity(n);
    let max_iter_per = 100; // iterations per eigenvalue
    let mut p = n; // active size: working on h[0..p, 0..p]

    while p > 0 {
        if p == 1 {
            eigenvalues.push(h[[0, 0]]);
            break;
        }
        if p == 2 {
            let a = h[[0,0]]; let b = h[[0,1]];
            let c = h[[1,0]]; let d = h[[1,1]];
            let tr = a + d;
            let det = a * d - b * c;
            let disc = (tr * tr - 4.0 * det).sqrt();
            eigenvalues.push((tr + disc) / 2.0);
            eigenvalues.push((tr - disc) / 2.0);
            break;
        }

        let mut converged = false;
        for _iter in 0..max_iter_per {
            let q = p - 1;

            // ── Deflation check ────────────────────────────────────────────
            // Check all subdiagonals for small values (from bottom up)
            let mut split_at = None;
            for i in (1..p).rev() {
                let tol = 1e-12 * (h[[i-1,i-1]].norm() + h[[i,i]].norm());
                if h[[i, i-1]].norm() <= tol {
                    h[[i, i-1]] = Complex::new(0.0, 0.0);
                    split_at = Some(i);
                    break;
                }
            }
            if let Some(i) = split_at {
                if i == q {
                    // Single eigenvalue deflated at bottom
                    eigenvalues.push(h[[q, q]]);
                    p -= 1;
                    converged = true;
                    break;
                } else if i == q - 1 {
                    // 2×2 block at bottom
                    let a = h[[q-1,q-1]]; let b = h[[q-1,q]];
                    let c = h[[q,q-1]];   let d = h[[q,q]];
                    let tr = a + d; let det = a*d - b*c;
                    let disc = (tr*tr - 4.0*det).sqrt();
                    eigenvalues.push((tr + disc) / 2.0);
                    eigenvalues.push((tr - disc) / 2.0);
                    p -= 2;
                    converged = true;
                    break;
                } else {
                    // Split: recursively handle upper part later, reduce p for lower
                    // For simplicity, just continue working on 0..p
                    // (next iteration will check deflation again)
                }
            }
            if converged { break; }

            // ── Wilkinson shift: eigenvalue of bottom 2×2 closest to h[q,q] ──
            let a = h[[q-1,q-1]]; let b = h[[q-1,q]];
            let c = h[[q,q-1]];   let d = h[[q,q]];
            let tr2 = a + d;
            let det2 = a * d - b * c;
            let disc = (tr2*tr2 - 4.0*det2).sqrt();
            let e1 = (tr2 + disc) / 2.0;
            let e2 = (tr2 - disc) / 2.0;
            // Pick the eigenvalue of the 2×2 closest to h[q,q]
            let shift = if (e1 - d).norm() <= (e2 - d).norm() { e1 } else { e2 };

            // ── Single-shift QR step using Givens rotations ────────────────
            // Apply H ← G_k^* H G_k for k = 0..p-2
            // First rotation eliminates h[1,0] after shift
            let mut x = h[[0,0]] - shift;
            let mut y = h[[1,0]];

            for k in 0..p-1 {
                // Compute Givens rotation [c, s; -s*, c] to zero y using x
                let r = (x.norm_sqr() + y.norm_sqr()).sqrt();
                if r < 1e-15 { continue; }
                let gc = x / r;
                let gs = y / r;

                // Left multiply: rows k and k+1, columns k-1..p
                let jstart = if k > 0 { k - 1 } else { 0 };
                for j in jstart..p {
                    let u = h[[k, j]];
                    let v = h[[k+1, j]];
                    h[[k, j]]   = gc.conj() * u + gs.conj() * v;
                    h[[k+1, j]] = -gs * u + gc * v;
                }
                // Right multiply: rows 0..p, columns k and k+1
                // (only need rows 0..min(k+3, p) for Hessenberg, but use p for correctness)
                let iend = (k + 3).min(p);
                for i in 0..iend {
                    let u = h[[i, k]];
                    let v = h[[i, k+1]];
                    h[[i, k]]   = gc * u + gs * v;
                    h[[i, k+1]] = -gs.conj() * u + gc.conj() * v;
                }

                // Next iteration uses the subdiagonal entry created
                if k + 1 < p - 1 {
                    x = h[[k+1, k]];
                    y = h[[k+2, k]];
                }
            }
        }

        if !converged {
            // Force deflation at the bottom even if not fully converged
            // (prevents infinite loop — take best approximation)
            eigenvalues.push(h[[p-1, p-1]]);
            p -= 1;
        }
    }

    Ok(eigenvalues)
}

/// eig(M) — eigenvalues of a square matrix.
/// Returns a complex Vector of length n.
fn builtin_eig(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("eig", &args, 1)?;
    let m = match &args[0] {
        Value::Matrix(m) => m,
        Value::Scalar(n) => {
            return Ok(Value::Vector(Array1::from_vec(vec![Complex::new(*n, 0.0)])));
        }
        other => return Err(ScriptError::Type(format!(
            "eig: expected a square matrix, got {}", other.type_name()
        ))),
    };
    let rows = m.nrows();
    let cols = m.ncols();
    if rows != cols {
        return Err(ScriptError::Type(format!(
            "eig: matrix must be square (got {}×{})", rows, cols
        )));
    }
    let h = hessenberg_reduce(m);
    let vals = eig_hessenberg(&h).map_err(ScriptError::Runtime)?;
    Ok(Value::Vector(Array1::from_vec(vals)))
}

fn builtin_whos_file(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("whos", &args, 1)?;
    let path = args[0].to_str().map_err(ScriptError::Type)?;

    if !path.ends_with(".npz") {
        return Err(ScriptError::Type(
            "whos: only .npz files are supported (e.g. whos(\"data.npz\"))".to_string()
        ));
    }

    use zip::ZipArchive;
    use std::io::Read;

    let file = std::fs::File::open(&path).map_err(|e| ScriptError::Runtime(e.to_string()))?;
    let mut zip = ZipArchive::new(file).map_err(|e| ScriptError::Runtime(e.to_string()))?;

    println!("\n  {:<20} {:<10} {}", "Name", "Type", "Size");
    println!("  {}", "─".repeat(44));

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let raw_name  = entry.name().to_string();
        let name      = raw_name.trim_end_matches(".npy");

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| ScriptError::Runtime(e.to_string()))?;

        let info = if buf.len() >= 10 && &buf[0..6] == b"\x93NUMPY" {
            let hlen = u16::from_le_bytes([buf[8], buf[9]]) as usize;
            if buf.len() >= 10 + hlen {
                if let Ok(header) = std::str::from_utf8(&buf[10..10 + hlen]) {
                    let dtype = if header.contains("<c16") || header.contains(">c16") {
                        "complex"
                    } else {
                        "real"
                    };
                    let size = match parse_npy_shape(header) {
                        Ok(s)  => s.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("×"),
                        Err(_) => "?".to_string(),
                    };
                    (dtype.to_string(), size)
                } else { ("?".to_string(), "?".to_string()) }
            } else { ("?".to_string(), "?".to_string()) }
        } else { ("?".to_string(), "?".to_string()) };

        println!("  {:<20} {:<10} {}", name, info.0, info.1);
    }
    println!();
    Ok(Value::None)
}

// ─── Struct construction and inspection ───────────────────────────────────────

fn builtin_struct(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.len() % 2 != 0 {
        return Err(ScriptError::Runtime(
            "struct() requires an even number of arguments: (field, value, ...)".to_string()
        ));
    }
    let mut fields = HashMap::new();
    let mut iter = args.into_iter();
    while let (Some(key), Some(val)) = (iter.next(), iter.next()) {
        let name = key.to_str().map_err(ScriptError::Runtime)?;
        fields.insert(name, val);
    }
    Ok(Value::Struct(fields))
}

// ─── Output builtins ──────────────────────────────────────────────────────────

fn builtin_disp(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("disp", &args, 1)?;
    println!("{}", args[0]);
    Ok(Value::None)
}

fn builtin_fprintf(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.is_empty() {
        return Err(ScriptError::Runtime("fprintf: expected a format string".to_string()));
    }
    let fmt = args[0].to_str().map_err(ScriptError::Type)?;
    let output = apply_format(&fmt, &args[1..]).map_err(ScriptError::Runtime)?;
    print!("{}", output);
    Ok(Value::None)
}

/// Normalise Rust's `{:e}` exponent to C-style `e+XX` / `e-XX`.
/// e.g. `1.23e4` → `1.23e+04`,  `1e-3` → `1.00e-03`
fn normalise_exp(s: &str) -> String {
    if let Some(e_pos) = s.find('e') {
        let mantissa = &s[..e_pos];
        let exp_str  = &s[e_pos + 1..];
        let (sign, digits) = if exp_str.starts_with('-') {
            ("-", &exp_str[1..])
        } else if exp_str.starts_with('+') {
            ("+", &exp_str[1..])
        } else {
            ("+", exp_str)
        };
        // Ensure at least 2 exponent digits
        let exp_num: i32 = digits.parse().unwrap_or(0);
        format!("{}e{}{:02}", mantissa, sign, exp_num)
    } else {
        s.to_string()
    }
}

/// Apply a C-style format string with the given argument slice.
/// Supports: %d %i %f %g %e %s %%   and escape sequences \n \t \\
pub fn apply_format(fmt: &str, args: &[Value]) -> Result<String, String> {
    let mut result = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    let mut arg_idx = 0;

    while i < chars.len() {
        // Escape sequences
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n'  => { result.push('\n'); i += 2; continue; }
                't'  => { result.push('\t'); i += 2; continue; }
                '\\' => { result.push('\\'); i += 2; continue; }
                _    => { result.push(chars[i]); i += 1; continue; }
            }
        }

        if chars[i] != '%' { result.push(chars[i]); i += 1; continue; }

        i += 1; // skip '%'
        if i >= chars.len() { return Err("fprintf: trailing '%'".to_string()); }
        if chars[i] == '%' { result.push('%'); i += 1; continue; }

        // Parse optional flags, width, precision
        let mut flags = String::new();
        while i < chars.len() && "-+ 0#".contains(chars[i]) { flags.push(chars[i]); i += 1; }

        let mut width_str = String::new();
        while i < chars.len() && chars[i].is_ascii_digit() { width_str.push(chars[i]); i += 1; }

        let mut prec_str = String::new();
        if i < chars.len() && chars[i] == '.' {
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() { prec_str.push(chars[i]); i += 1; }
        }

        if i >= chars.len() { return Err("fprintf: incomplete format specifier".to_string()); }
        let spec = chars[i]; i += 1;

        let arg = args.get(arg_idx).ok_or_else(|| {
            format!("fprintf: not enough arguments (need arg {} for '%{}')", arg_idx + 1, spec)
        })?;
        arg_idx += 1;

        let w = width_str.parse::<usize>().unwrap_or(0);
        let p = prec_str.parse::<usize>().unwrap_or(6);
        let left = flags.contains('-');

        let piece = match spec {
            'd' | 'i' => {
                let n = arg.to_scalar().map_err(|e| format!("fprintf %d: {}", e))? as i64;
                if left { format!("{:<width$}", n, width = w) }
                else    { format!("{:>width$}", n, width = w) }
            }
            'f' => {
                let n = arg.to_scalar().map_err(|e| format!("fprintf %f: {}", e))?;
                if left { format!("{:<width$.prec$}", n, width = w, prec = p) }
                else    { format!("{:>width$.prec$}", n, width = w, prec = p) }
            }
            'e' => {
                let n = arg.to_scalar().map_err(|e| format!("fprintf %e: {}", e))?;
                // Rust's {:e} omits the '+' sign and leading zeros in the exponent;
                // normalise to C-style e+XX / e-XX  (e.g.  1.23e+04)
                let base = format!("{:.prec$e}", n, prec = p);
                let base = normalise_exp(&base);
                if left { format!("{:<width$}", base, width = w) }
                else    { format!("{:>width$}", base, width = w) }
            }
            'g' => {
                let n = arg.to_scalar().map_err(|e| format!("fprintf %g: {}", e))?;
                let base = if n == 0.0 || (n.abs() >= 1e-4 && n.abs() < 1e6) {
                    // Trim trailing zeros like %g
                    let s = format!("{:.prec$}", n, prec = p);
                    s.trim_end_matches('0').trim_end_matches('.').to_string()
                } else {
                    let s = format!("{:.prec$e}", n, prec = p);
                    s
                };
                if left { format!("{:<width$}", base, width = w) }
                else    { format!("{:>width$}", base, width = w) }
            }
            's' => {
                let s = arg.to_str().map_err(|e| format!("fprintf %s: {}", e))?;
                if left { format!("{:<width$}", s, width = w) }
                else    { format!("{:>width$}", s, width = w) }
            }
            other => return Err(format!("fprintf: unknown specifier '%{}'", other)),
        };
        result.push_str(&piece);
    }
    Ok(result)
}

// ─── Aggregate builtins ───────────────────────────────────────────────────────

fn builtin_all(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("all", &args, 1)?;
    match &args[0] {
        Value::Bool(b)   => Ok(Value::Bool(*b)),
        Value::Scalar(n) => Ok(Value::Bool(*n != 0.0)),
        Value::Vector(v) => Ok(Value::Bool(v.iter().all(|c| c.re != 0.0 || c.im != 0.0))),
        other => Err(ScriptError::Type(format!("all: expected vector or scalar, got {}", other.type_name()))),
    }
}

fn builtin_any(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("any", &args, 1)?;
    match &args[0] {
        Value::Bool(b)   => Ok(Value::Bool(*b)),
        Value::Scalar(n) => Ok(Value::Bool(*n != 0.0)),
        Value::Vector(v) => Ok(Value::Bool(v.iter().any(|c| c.re != 0.0 || c.im != 0.0))),
        other => Err(ScriptError::Type(format!("any: expected vector or scalar, got {}", other.type_name()))),
    }
}

// ─── rank() and roots() ───────────────────────────────────────────────────────

fn builtin_rank(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("rank", &args, 1)?;
    let m = match &args[0] {
        Value::Matrix(m) => m.clone(),
        Value::Scalar(_) => return Ok(Value::Scalar(1.0)),
        Value::Vector(v) if !v.is_empty() => return Ok(Value::Scalar(1.0)),
        other => return Err(ScriptError::Type(format!(
            "rank: expected matrix, got {}", other.type_name()
        ))),
    };
    if m.nrows() == 0 || m.ncols() == 0 { return Ok(Value::Scalar(0.0)); }

    // Singular values = sqrt(|eigenvalues of A†A|)
    let ata: CMatrix = m.t().mapv(|c| c.conj()).dot(&m);
    let h = hessenberg_reduce(&ata);
    let evals = eig_hessenberg(&h).map_err(ScriptError::Runtime)?;

    let svs: Vec<f64> = evals.iter().map(|c| c.norm().sqrt()).collect();
    let max_sv = svs.iter().cloned().fold(0.0_f64, f64::max);
    // A†A eigenvalue approach squares the matrix, so rounding errors are amplified;
    // use sqrt(eps) rather than eps to set a robust threshold.
    let tol = f64::EPSILON.sqrt() * (m.nrows().max(m.ncols()) as f64) * max_sv;

    let r = svs.iter().filter(|&&s| s > tol).count();
    Ok(Value::Scalar(r as f64))
}

fn builtin_roots(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("roots", &args, 1)?;
    let coeffs = args[0].to_cvector().map_err(ScriptError::Type)?;

    // Strip leading near-zero coefficients
    let first = match coeffs.iter().position(|c| c.norm() > 1e-15) {
        Some(i) => i,
        None    => return Ok(Value::Vector(Array1::zeros(0))),
    };
    let p: Vec<C64> = coeffs.iter().skip(first).cloned().collect();

    let deg = p.len().saturating_sub(1);
    if deg == 0 { return Ok(Value::Vector(Array1::zeros(0))); }
    if deg == 1 {
        // a*x + b = 0  →  x = -b/a
        return Ok(Value::Vector(Array1::from_vec(vec![-p[1] / p[0]])));
    }

    // Build Frobenius companion matrix (deg × deg)
    let lead = p[0];
    let mut comp: CMatrix = Array2::zeros((deg, deg));
    // First row: -p[1..] / lead
    for j in 0..deg { comp[[0, j]] = -p[j + 1] / lead; }
    // Sub-diagonal of ones
    for i in 1..deg { comp[[i, i - 1]] = Complex::new(1.0, 0.0); }

    let h  = hessenberg_reduce(&comp);
    let rs = eig_hessenberg(&h).map_err(ScriptError::Runtime)?;
    Ok(Value::Vector(Array1::from_vec(rs)))
}

fn builtin_isstruct(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("isstruct", &args, 1)?;
    Ok(Value::Bool(matches!(args[0], Value::Struct(_))))
}

fn builtin_fieldnames(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("fieldnames", &args, 1)?;
    match &args[0] {
        Value::Struct(fields) => {
            let mut names: Vec<_> = fields.keys().cloned().collect();
            names.sort();
            for name in &names {
                println!("  {}", name);
            }
            Ok(Value::None)
        }
        other => Err(ScriptError::Runtime(format!(
            "fieldnames() requires a struct, got {}", other.type_name()
        ))),
    }
}

fn builtin_isfield(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("isfield", &args, 2)?;
    let field = args[1].to_str().map_err(ScriptError::Runtime)?;
    match &args[0] {
        Value::Struct(fields) => Ok(Value::Bool(fields.contains_key(&field))),
        other => Err(ScriptError::Runtime(format!(
            "isfield() requires a struct, got {}", other.type_name()
        ))),
    }
}

fn builtin_rmfield(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("rmfield", &args, 2)?;
    let field = args[1].to_str().map_err(ScriptError::Runtime)?;
    match args.into_iter().next().unwrap() {
        Value::Struct(mut fields) => {
            if fields.remove(&field).is_none() {
                return Err(ScriptError::Runtime(format!("struct has no field '{}'", field)));
            }
            Ok(Value::Struct(fields))
        }
        other => Err(ScriptError::Runtime(format!(
            "rmfield() requires a struct, got {}", other.type_name()
        ))),
    }
}

// ── Phase 2: Transfer Function builtins ──────────────────────────────────────

fn builtin_tf(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("tf", &args, 1, 2)?;
    if args.len() == 1 {
        // tf("s") → Laplace variable s, representing the polynomial s/1
        let s = args[0].to_str().map_err(ScriptError::Runtime)?;
        if s != "s" {
            return Err(ScriptError::Runtime(format!(
                "tf: single-argument form expects \"s\", got \"{}\"", s
            )));
        }
        Ok(Value::TransferFn { num: vec![1.0, 0.0], den: vec![1.0] })
    } else {
        // tf(num_vec, den_vec) → explicit transfer function
        let num_cv = args[0].to_cvector().map_err(ScriptError::Type)?;
        let den_cv = args[1].to_cvector().map_err(ScriptError::Type)?;
        let num: Result<Vec<f64>, ScriptError> = num_cv.iter().map(|c| {
            if c.im.abs() > 1e-12 {
                Err(ScriptError::Type("tf: numerator coefficients must be real".to_string()))
            } else {
                Ok(c.re)
            }
        }).collect();
        let den: Result<Vec<f64>, ScriptError> = den_cv.iter().map(|c| {
            if c.im.abs() > 1e-12 {
                Err(ScriptError::Type("tf: denominator coefficients must be real".to_string()))
            } else {
                Ok(c.re)
            }
        }).collect();
        if den_cv.is_empty() {
            return Err(ScriptError::Runtime("tf: denominator must be non-empty".to_string()));
        }
        Ok(Value::TransferFn { num: num?, den: den? })
    }
}

/// Convert a real polynomial coefficient slice to a complex Value::Vector for roots().
fn real_poly_to_value(coeffs: &[f64]) -> Value {
    Value::Vector(Array1::from_iter(coeffs.iter().map(|&x| Complex::new(x, 0.0))))
}

fn builtin_pole(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("pole", &args, 1)?;
    match &args[0] {
        Value::TransferFn { den, .. } => builtin_roots(vec![real_poly_to_value(den)]),
        other => Err(ScriptError::Type(format!("pole: expected tf, got {}", other.type_name()))),
    }
}

fn builtin_zero(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("zero", &args, 1)?;
    match &args[0] {
        Value::TransferFn { num, .. } => builtin_roots(vec![real_poly_to_value(num)]),
        other => Err(ScriptError::Type(format!("zero: expected tf, got {}", other.type_name()))),
    }
}

// ── Phase 3: State-Space builtins ─────────────────────────────────────────────

/// Convert a TransferFn to observable canonical form StateSpace.
fn tf_to_ss(num: &[f64], den: &[f64]) -> Result<Value, String> {
    use ndarray::Array2;

    if den.is_empty() {
        return Err("ss: empty denominator".to_string());
    }
    let n = den.len() - 1;
    if n == 0 {
        return Err("ss: transfer function must have order >= 1".to_string());
    }
    let d0 = den[0];
    if d0.abs() < 1e-15 {
        return Err("ss: leading denominator coefficient is zero".to_string());
    }

    // Monic denominator: a = [1, a₁, ..., aₙ]
    let a: Vec<f64> = den.iter().map(|&x| x / d0).collect();
    let num_norm: Vec<f64> = num.iter().map(|&x| x / d0).collect();

    // Separate direct feedthrough (D) from strictly proper numerator coefficients (b)
    let (d_val, b): (f64, Vec<f64>) = if num_norm.len() == n + 1 {
        // Proper (non-strictly-proper): subtract D * monic_den
        let dv = num_norm[0];
        let bv: Vec<f64> = num_norm[1..].iter().zip(a[1..].iter())
            .map(|(&ni, &ai)| ni - dv * ai)
            .collect();
        (dv, bv)
    } else {
        // Strictly proper: pad numerator with leading zeros to length n
        let mut bv = vec![0.0f64; n];
        let offset = n.saturating_sub(num_norm.len());
        for (i, &x) in num_norm.iter().enumerate() {
            bv[offset + i] = x;
        }
        (0.0, bv)
    };

    // Observable canonical form:
    //   A[i,0] = -aᵢ₊₁  (first column)
    //   A[i,i+1] = 1     (super-diagonal)
    let mut a_mat: CMatrix = Array2::zeros((n, n));
    for i in 0..n {
        a_mat[[i, 0]] = Complex::new(-a[i + 1], 0.0);
    }
    for i in 0..n - 1 {
        a_mat[[i, i + 1]] = Complex::new(1.0, 0.0);
    }

    // B: n×1, numerator coefficients
    let b_mat: CMatrix = Array2::from_shape_fn((n, 1), |(i, _)| Complex::new(b[i], 0.0));

    // C: 1×n, [1, 0, 0, ...]
    let c_mat: CMatrix = Array2::from_shape_fn((1, n), |(_, j)| {
        if j == 0 { Complex::new(1.0, 0.0) } else { Complex::new(0.0, 0.0) }
    });

    // D: 1×1
    let d_mat: CMatrix = Array2::from_shape_vec((1, 1), vec![Complex::new(d_val, 0.0)])
        .map_err(|e| e.to_string())?;

    Ok(Value::StateSpace { A: a_mat, B: b_mat, C: c_mat, D: d_mat })
}

fn builtin_ss(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("ss", &args, 1)?;
    match &args[0] {
        Value::TransferFn { num, den } => tf_to_ss(num, den).map_err(ScriptError::Runtime),
        other => Err(ScriptError::Type(format!(
            "ss: expected tf, got {} (direct ss(A,B,C,D) construction not yet supported)",
            other.type_name()
        ))),
    }
}

fn builtin_ctrb(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("ctrb", &args, 2)?;
    let a = match &args[0] {
        Value::Matrix(m) => m.clone(),
        other => return Err(ScriptError::Type(format!(
            "ctrb: A must be a matrix, got {}", other.type_name()
        ))),
    };
    let b = match &args[1] {
        Value::Matrix(m) => m.clone(),
        Value::Vector(v) => {
            // Treat a vector as a column matrix
            let n = v.len();
            Array2::from_shape_fn((n, 1), |(i, _)| v[i])
        }
        other => return Err(ScriptError::Type(format!(
            "ctrb: B must be a matrix or vector, got {}", other.type_name()
        ))),
    };
    let n = a.nrows();
    if a.ncols() != n {
        return Err(ScriptError::Runtime("ctrb: A must be square".to_string()));
    }
    if b.nrows() != n {
        return Err(ScriptError::Runtime(format!(
            "ctrb: B has {} rows but A is {}×{}", b.nrows(), n, n
        )));
    }
    let m = b.ncols();
    // Build [B, AB, A²B, ..., A^(n-1)B] — n×(n*m)
    let mut cols: Vec<CMatrix> = Vec::with_capacity(n);
    let mut ab = b.clone();
    cols.push(ab.clone());
    for _ in 1..n {
        ab = a.dot(&ab);
        cols.push(ab.clone());
    }
    // Horizontally concatenate all columns
    let total_cols = n * m;
    let mut data: Vec<C64> = Vec::with_capacity(n * total_cols);
    for r in 0..n {
        for block in &cols {
            for c in 0..m {
                data.push(block[[r, c]]);
            }
        }
    }
    let result = Array2::from_shape_vec((n, total_cols), data)
        .map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::Matrix(result))
}

fn builtin_obsv(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("obsv", &args, 2)?;
    let a = match &args[0] {
        Value::Matrix(m) => m.clone(),
        other => return Err(ScriptError::Type(format!(
            "obsv: A must be a matrix, got {}", other.type_name()
        ))),
    };
    let c = match &args[1] {
        Value::Matrix(m) => m.clone(),
        Value::Vector(v) => {
            // Treat a vector as a row matrix
            let n = v.len();
            Array2::from_shape_fn((1, n), |(_, j)| v[j])
        }
        other => return Err(ScriptError::Type(format!(
            "obsv: C must be a matrix or vector, got {}", other.type_name()
        ))),
    };
    let n = a.nrows();
    if a.ncols() != n {
        return Err(ScriptError::Runtime("obsv: A must be square".to_string()));
    }
    if c.ncols() != n {
        return Err(ScriptError::Runtime(format!(
            "obsv: C has {} columns but A is {}×{}", c.ncols(), n, n
        )));
    }
    let p = c.nrows();
    // Build [C; CA; CA²; ...; CA^(n-1)] — (n*p)×n
    let mut rows: Vec<CMatrix> = Vec::with_capacity(n);
    let mut ca = c.clone();
    rows.push(ca.clone());
    for _ in 1..n {
        ca = ca.dot(&a);
        rows.push(ca.clone());
    }
    // Vertically concatenate all rows
    let total_rows = n * p;
    let mut data: Vec<C64> = Vec::with_capacity(total_rows * n);
    for block in &rows {
        for r in 0..p {
            for c_idx in 0..n {
                data.push(block[[r, c_idx]]);
            }
        }
    }
    let result = Array2::from_shape_vec((total_rows, n), data)
        .map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::Matrix(result))
}

// ── Phase 4: Frequency & Time-Domain Analysis ─────────────────────────────────

/// Evaluate a real polynomial at a complex point via Horner's method.
fn poly_eval_c(coeffs: &[f64], s: C64) -> C64 {
    let mut acc = Complex::new(0.0, 0.0);
    for &c in coeffs {
        acc = acc * s + Complex::new(c, 0.0);
    }
    acc
}

/// n log-spaced values from start to stop.
fn logspace(start: f64, stop: f64, n: usize) -> Vec<f64> {
    if n == 0 { return Vec::new(); }
    if n == 1 { return vec![start]; }
    let ls = start.log10();
    let le = stop.log10();
    (0..n).map(|i| {
        let t = i as f64 / (n - 1) as f64;
        10.0_f64.powf(ls + t * (le - ls))
    }).collect()
}

/// Take real part of a CMatrix.
fn to_real_mat(m: &CMatrix) -> ndarray::Array2<f64> { m.mapv(|c| c.re) }

/// Single RK4 integration step for ẋ = Ax + Bu (real, SISO u scalar).
fn rk4_step(
    a: &ndarray::Array2<f64>,
    b: &ndarray::Array2<f64>,
    x: &ndarray::Array1<f64>,
    u: f64,
    h: f64,
) -> ndarray::Array1<f64> {
    let bu: ndarray::Array1<f64> = b.column(0).mapv(|bi| bi * u);
    let f = |xk: &ndarray::Array1<f64>| -> ndarray::Array1<f64> { a.dot(xk) + &bu };
    let k1 = f(x);
    let x2: ndarray::Array1<f64> = x + &k1.mapv(|v| v * (h / 2.0));
    let k2 = f(&x2);
    let x3: ndarray::Array1<f64> = x + &k2.mapv(|v| v * (h / 2.0));
    let k3 = f(&x3);
    let x4: ndarray::Array1<f64> = x + &k3.mapv(|v| v * h);
    let k4 = f(&x4);
    let dx = (k1 + k2.mapv(|v| v * 2.0) + k3.mapv(|v| v * 2.0) + k4).mapv(|v| v * (h / 6.0));
    x + &dx
}

/// Unwrap phase (degrees) across a sequence to remove ±360° jumps.
fn unwrap_phase_deg(phase: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0f64; phase.len()];
    if phase.is_empty() { return out; }
    out[0] = phase[0];
    for i in 1..phase.len() {
        let diff = phase[i] - out[i - 1];
        let adj = if diff > 180.0 { diff - 360.0 } else if diff < -180.0 { diff + 360.0 } else { diff };
        out[i] = out[i - 1] + adj;
    }
    out
}

/// Find the x-value where y crosses `target` (first crossing, linear interpolation).
fn find_crossing(x: &[f64], y: &[f64], target: f64) -> Option<f64> {
    for i in 0..y.len().saturating_sub(1) {
        let y0 = y[i]     - target;
        let y1 = y[i + 1] - target;
        if y0 * y1 <= 0.0 && (y0 - y1).abs() > 1e-30 {
            let t = y0 / (y0 - y1);
            return Some(x[i] + t * (x[i + 1] - x[i]));
        }
    }
    None
}

/// Evaluate Bode data (mag_dB, phase_deg unwrapped) for a TF over frequency vector w.
fn bode_compute(num: &[f64], den: &[f64], w: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let h: Vec<C64> = w.iter().map(|&wi| {
        let jw = Complex::new(0.0, wi);
        let n = poly_eval_c(num, jw);
        let d = poly_eval_c(den, jw);
        if d.norm() < 1e-300 { Complex::new(f64::INFINITY, 0.0) } else { n / d }
    }).collect();
    let mag_db:    Vec<f64> = h.iter().map(|v| 20.0 * v.norm().log10()).collect();
    let phase_raw: Vec<f64> = h.iter().map(|v| v.arg().to_degrees()).collect();
    (mag_db, unwrap_phase_deg(&phase_raw))
}

/// Auto frequency range based on pole magnitudes.
fn auto_freq_range(den: &[f64]) -> Result<Vec<f64>, ScriptError> {
    let poles = builtin_roots(vec![real_poly_to_value(den)])?;
    let w_nat = match &poles {
        Value::Vector(v) if !v.is_empty() =>
            v.iter().map(|c| c.norm()).fold(0.0f64, f64::max).max(1.0),
        _ => 1.0,
    };
    Ok(logspace((w_nat * 0.01).max(1e-3), w_nat * 100.0, 200))
}

fn builtin_bode(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("bode", &args, 1, 2)?;
    let (num, den) = match &args[0] {
        Value::TransferFn { num, den } => (num.clone(), den.clone()),
        other => return Err(ScriptError::Type(format!(
            "bode: expected tf, got {}", other.type_name()
        ))),
    };

    let w_vec: Vec<f64> = if args.len() == 2 {
        match &args[1] {
            Value::Vector(v) => v.iter().map(|c| c.re).collect(),
            other => return Err(ScriptError::Type(format!(
                "bode: w must be a vector, got {}", other.type_name()
            ))),
        }
    } else {
        auto_freq_range(&den)?
    };

    let (mag_db, phase_deg) = bode_compute(&num, &den, &w_vec);

    // Plot on log10(ω) x-axis for visual log scaling
    let log_w: Vec<f64> = w_vec.iter().map(|&w| w.log10()).collect();

    FIGURE.with(|fig| fig.borrow_mut().set_subplot(2, 1, 1));
    push_xy_line(log_w.clone(), mag_db.clone(), "magnitude", "Bode Plot", None, LineStyle::Solid);
    FIGURE.with(|fig| {
        let mut f = fig.borrow_mut();
        let sp = f.current_mut();
        sp.xlabel = "log10(ω rad/s)".to_string();
        sp.ylabel = "Magnitude (dB)".to_string();
    });

    FIGURE.with(|fig| fig.borrow_mut().set_subplot(2, 1, 2));
    push_xy_line(log_w, phase_deg.clone(), "phase", "", None, LineStyle::Solid);
    FIGURE.with(|fig| {
        let mut f = fig.borrow_mut();
        let sp = f.current_mut();
        sp.xlabel = "log10(ω rad/s)".to_string();
        sp.ylabel = "Phase (deg)".to_string();
    });

    render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))?;

    let w_val   = Value::Vector(Array1::from_iter(w_vec.iter()   .map(|&x| Complex::new(x, 0.0))));
    let mag_val = Value::Vector(Array1::from_iter(mag_db.iter()  .map(|&x| Complex::new(x, 0.0))));
    let ph_val  = Value::Vector(Array1::from_iter(phase_deg.iter().map(|&x| Complex::new(x, 0.0))));
    Ok(Value::Tuple(vec![mag_val, ph_val, w_val]))
}

fn builtin_step(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args_range("step", &args, 1, 2)?;
    let (num, den) = match &args[0] {
        Value::TransferFn { num, den } => (num.clone(), den.clone()),
        other => return Err(ScriptError::Type(format!(
            "step: expected tf, got {}", other.type_name()
        ))),
    };

    // Convert TF → SS
    let (a_c, b_c, c_c, d_c) = match tf_to_ss(&num, &den).map_err(ScriptError::Runtime)? {
        Value::StateSpace { A, B, C, D } => (A, B, C, D),
        _ => unreachable!(),
    };
    let a = to_real_mat(&a_c);
    let b = to_real_mat(&b_c);
    let c = to_real_mat(&c_c);
    let d = to_real_mat(&d_c);

    // Auto t_end: 10 / slowest pole decay rate, capped at 100 s
    let t_end: f64 = if args.len() == 2 {
        args[1].to_scalar().map_err(ScriptError::Type)?
    } else {
        let poles = builtin_roots(vec![real_poly_to_value(&den)])?;
        let min_decay = match &poles {
            Value::Vector(v) if !v.is_empty() =>
                v.iter().map(|p| p.re.abs()).fold(f64::INFINITY, f64::min).max(1e-6),
            _ => 1.0,
        };
        (10.0 / min_decay).min(100.0)
    };

    let n_steps = 1000usize;
    let h = t_end / n_steps as f64;
    let n = a.nrows();

    let mut x: ndarray::Array1<f64> = ndarray::Array1::zeros(n);
    let mut t_out = Vec::with_capacity(n_steps + 1);
    let mut y_out = Vec::with_capacity(n_steps + 1);

    for k in 0..=n_steps {
        let y_k = c.dot(&x)[0] + d[[0, 0]]; // u = 1
        t_out.push(k as f64 * h);
        y_out.push(y_k);
        if k < n_steps {
            x = rk4_step(&a, &b, &x, 1.0, h);
        }
    }

    push_xy_line(t_out.clone(), y_out.clone(), "y(t)", "Step Response", None, LineStyle::Solid);
    FIGURE.with(|fig| {
        let mut f = fig.borrow_mut();
        let sp = f.current_mut();
        sp.xlabel = "Time (s)".to_string();
        sp.ylabel = "Amplitude".to_string();
    });
    render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))?;

    let y_val = Value::Vector(Array1::from_iter(y_out.iter().map(|&v| Complex::new(v, 0.0))));
    let t_val = Value::Vector(Array1::from_iter(t_out.iter().map(|&v| Complex::new(v, 0.0))));
    Ok(Value::Tuple(vec![y_val, t_val]))
}

// ─── LQR helpers ──────────────────────────────────────────────────────────────

/// Multiply two complex matrices: C = A * B  (naive O(n³), fine for small n)
fn mat_mul_cx(a: &CMatrix, b: &CMatrix) -> CMatrix {
    let (ra, ca) = (a.nrows(), a.ncols());
    let cb = b.ncols();
    assert_eq!(ca, b.nrows(), "mat_mul_cx: inner dimensions must match");
    let mut c: CMatrix = Array2::zeros((ra, cb));
    for i in 0..ra {
        for j in 0..cb {
            for k in 0..ca {
                c[[i, j]] += a[[i, k]] * b[[k, j]];
            }
        }
    }
    c
}

/// Inverse iteration: given matrix M and approximate eigenvalue λ,
/// return the corresponding eigenvector (2n-dim complex vector).
fn inverse_iteration_cx(m: &CMatrix, eigenvalue: C64, max_iter: usize) -> Result<CVector, ScriptError> {
    let n = m.nrows();
    // Perturb the shift so (M - shift*I) is nonsingular
    let scale = eigenvalue.norm().max(1.0);
    let shift = eigenvalue + Complex::new(scale * 1e-6, scale * 1e-6);

    let mut shifted = m.to_owned();
    for i in 0..n {
        shifted[[i, i]] -= shift;
    }

    let inv = matrix_inv(&shifted).map_err(|e| {
        ScriptError::Type(format!("lqr: inverse iteration failed (singular shift): {}", e))
    })?;

    // Initial vector: unit in first component
    let mut v: CVector = Array1::zeros(n);
    v[0] = Complex::new(1.0, 0.0);

    for _ in 0..max_iter {
        // v = inv * v
        let mut new_v: CVector = Array1::zeros(n);
        for i in 0..n {
            for j in 0..n {
                new_v[i] += inv[[i, j]] * v[j];
            }
        }
        let norm: f64 = new_v.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
        if norm < 1e-15 {
            break;
        }
        for c in new_v.iter_mut() {
            *c /= norm;
        }
        v = new_v;
    }
    Ok(v)
}

// ─── lqr(sys_ss, Q, R) ────────────────────────────────────────────────────────

/// `[K, S, e] = lqr(sys_ss, Q, R)`
///
/// Solve the continuous-time algebraic Riccati equation (CARE):
///   A'P + PA − P·B·R⁻¹·B'·P + Q = 0
/// Returns:
///   K  (m×n) — optimal feedback gain: u = -K*x
///   S  (n×n) — Riccati solution P
///   e  (n×1) — closed-loop eigenvalues of (A − B·K)
///
/// Algorithm: Hamiltonian matrix eigendecomposition.
///   H = [A, -B·R⁻¹·B'; -Q, -A']
/// Select the n stable eigenvectors [V1; V2], then P = V2·inv(V1).
fn builtin_lqr(args: Vec<Value>) -> Result<Value, ScriptError> {
    if args.len() != 3 {
        return Err(ScriptError::Type(
            "lqr: requires 3 arguments: lqr(sys, Q, R)".to_string(),
        ));
    }

    // Extract A, B from state-space system
    let (a_mat, b_mat) = match &args[0] {
        Value::StateSpace { A, B, .. } => (A.clone(), B.clone()),
        other => {
            return Err(ScriptError::Type(format!(
                "lqr: first argument must be a state-space system, got {}",
                other.type_name()
            )))
        }
    };

    // Extract Q
    let q_mat = match &args[1] {
        Value::Matrix(m) => m.clone(),
        other => {
            return Err(ScriptError::Type(format!(
                "lqr: Q must be a matrix, got {}",
                other.type_name()
            )))
        }
    };

    // Extract R (allow scalar or matrix)
    let r_mat: CMatrix = match &args[2] {
        Value::Matrix(m) => m.clone(),
        Value::Scalar(s) => {
            let mut m: CMatrix = Array2::zeros((1, 1));
            m[[0, 0]] = Complex::new(*s, 0.0);
            m
        }
        other => {
            return Err(ScriptError::Type(format!(
                "lqr: R must be a matrix or scalar, got {}",
                other.type_name()
            )))
        }
    };

    let n = a_mat.nrows();
    if n != a_mat.ncols() {
        return Err(ScriptError::Type("lqr: A must be square".to_string()));
    }
    if q_mat.nrows() != n || q_mat.ncols() != n {
        return Err(ScriptError::Type(format!(
            "lqr: Q must be {}×{}, got {}×{}",
            n, n, q_mat.nrows(), q_mat.ncols()
        )));
    }
    let m_in = b_mat.ncols();
    if r_mat.nrows() != m_in || r_mat.ncols() != m_in {
        return Err(ScriptError::Type(format!(
            "lqr: R must be {}×{} (inputs), got {}×{}",
            m_in, m_in, r_mat.nrows(), r_mat.ncols()
        )));
    }

    // R⁻¹
    let r_inv = matrix_inv(&r_mat)
        .map_err(|e| ScriptError::Type(format!("lqr: R is singular: {}", e)))?;

    // G = B · R⁻¹ · B'  (n×n)
    let br = mat_mul_cx(&b_mat, &r_inv);               // n×m
    let bt: CMatrix = b_mat.t().mapv(|c| c.conj()).to_owned(); // m×n
    let g = mat_mul_cx(&br, &bt);                       // n×n

    // Hamiltonian H = [A, -G; -Q, -A']  (2n×2n)
    let two_n = 2 * n;
    let mut ham: CMatrix = Array2::zeros((two_n, two_n));
    for i in 0..n {
        for j in 0..n {
            ham[[i, j]]         =  a_mat[[i, j]];
            ham[[i, n + j]]     = -g[[i, j]];
            ham[[n + i, j]]     = -q_mat[[i, j]];
            ham[[n + i, n + j]] = -a_mat[[j, i]].conj(); // -A'
        }
    }

    // Eigenvalues of H
    let h_hess = hessenberg_reduce(&ham);
    let all_eigs = eig_hessenberg(&h_hess)
        .map_err(|e| ScriptError::Type(format!("lqr: Hamiltonian eigenvalues failed: {}", e)))?;

    // Select the n stable eigenvalues (Re < 0), sort for determinism
    let mut stable: Vec<C64> = all_eigs
        .iter()
        .filter(|e| e.re < -1e-10)
        .cloned()
        .collect();

    if stable.len() < n {
        return Err(ScriptError::Type(format!(
            "lqr: found only {} stable Hamiltonian eigenvalues (need {}); \
             system may not be stabilizable",
            stable.len(),
            n
        )));
    }

    // Sort by real part (most negative first) for numerical consistency
    stable.sort_by(|a, b| a.re.partial_cmp(&b.re).unwrap_or(std::cmp::Ordering::Equal));
    let stable = &stable[..n];

    // Eigenvectors via inverse iteration  →  V is 2n×n
    let mut v_mat: CMatrix = Array2::zeros((two_n, n));
    for (col, &lam) in stable.iter().enumerate() {
        let v = inverse_iteration_cx(&ham, lam, 40)?;
        for i in 0..two_n {
            v_mat[[i, col]] = v[i];
        }
    }

    // V1 = top n rows, V2 = bottom n rows
    let mut v1: CMatrix = Array2::zeros((n, n));
    let mut v2: CMatrix = Array2::zeros((n, n));
    for i in 0..n {
        for j in 0..n {
            v1[[i, j]] = v_mat[[i, j]];
            v2[[i, j]] = v_mat[[n + i, j]];
        }
    }

    // P = V2 · inv(V1)  — should be real symmetric positive semi-definite
    let v1_inv = matrix_inv(&v1)
        .map_err(|e| ScriptError::Type(format!("lqr: eigenvector matrix V1 is singular: {}", e)))?;
    let p_cx = mat_mul_cx(&v2, &v1_inv);

    // Take real part (imaginary residuals ≈ 0 for well-conditioned problems)
    let mut p: CMatrix = Array2::zeros((n, n));
    for i in 0..n {
        for j in 0..n {
            p[[i, j]] = Complex::new(p_cx[[i, j]].re, 0.0);
        }
    }

    // K = R⁻¹ · B' · P  (m×n)
    let bt_p = mat_mul_cx(&bt, &p);   // m×n
    let k_mat = mat_mul_cx(&r_inv, &bt_p); // m×n

    // Closed-loop eigenvalues  e = eig(A − B·K)
    let bk = mat_mul_cx(&b_mat, &k_mat); // n×n
    let mut a_cl: CMatrix = a_mat.clone();
    for i in 0..n {
        for j in 0..n {
            a_cl[[i, j]] -= bk[[i, j]];
        }
    }
    let a_cl_h = hessenberg_reduce(&a_cl);
    let cl_eigs = eig_hessenberg(&a_cl_h)
        .map_err(|e| ScriptError::Type(format!("lqr: closed-loop eig failed: {}", e)))?;

    let e_vec: CVector = Array1::from_vec(cl_eigs);

    Ok(Value::Tuple(vec![
        Value::Matrix(k_mat),
        Value::Matrix(p),
        Value::Vector(e_vec),
    ]))
}

/// Add two real polynomials: result = p1 + k * p2.
/// Both in descending-power order; p2 is right-aligned (zero-padded left) if shorter.
fn poly_add_scaled(p1: &[f64], p2: &[f64], k: f64) -> Vec<f64> {
    let n = p1.len();
    let m = p2.len();
    let mut result = p1.to_vec();
    let offset = n.saturating_sub(m);
    for (i, &c) in p2.iter().enumerate() {
        let idx = offset + i;
        if idx < n {
            result[idx] += k * c;
        }
    }
    result
}

/// Reorder `new_roots` to minimise total displacement from `prev_roots` (greedy nearest-neighbour).
/// This keeps root trajectories continuous across K steps.
fn pair_roots_by_proximity(new_roots: Vec<C64>, prev: &[C64]) -> Vec<C64> {
    let n = prev.len().min(new_roots.len());
    let mut used = vec![false; new_roots.len()];
    let mut result = new_roots.clone();
    for i in 0..n {
        let best_j = new_roots.iter().enumerate()
            .filter(|(j, _)| !used[*j])
            .min_by(|(_, a), (_, b)| {
                let da = (*a - prev[i]).norm();
                let db = (*b - prev[i]).norm();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(j, _)| j)
            .unwrap_or(i);
        result[i] = new_roots[best_j];
        used[best_j] = true;
    }
    result
}

fn builtin_rlocus(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("rlocus", &args, 1)?;
    let (num, den) = match &args[0] {
        Value::TransferFn { num, den } => (num.clone(), den.clone()),
        other => return Err(ScriptError::Type(format!(
            "rlocus: expected tf, got {}", other.type_name()
        ))),
    };

    let n_poles = den.len().saturating_sub(1);
    if n_poles == 0 {
        return Err(ScriptError::Runtime("rlocus: system has no poles".to_string()));
    }
    let n_zeros = num.len().saturating_sub(1);
    if n_zeros >= n_poles {
        return Err(ScriptError::Runtime(format!(
            "rlocus: TF must be proper (deg(num) < deg(den)), got {n_zeros} >= {n_poles}"
        )));
    }

    // Open-loop poles (K=0): roots of den, sorted by Im for stable initial ordering
    let ol_val = builtin_roots(vec![real_poly_to_value(&den)])?;
    let mut ol_poles: Vec<C64> = match ol_val {
        Value::Vector(v) => v.to_vec(),
        _ => return Err(ScriptError::Runtime("rlocus: failed to compute poles".to_string())),
    };
    ol_poles.sort_by(|a, b| a.im.partial_cmp(&b.im).unwrap_or(std::cmp::Ordering::Equal));

    // K sweep: log-spaced from 1e-3 to 1e4, 300 points
    let k_vals = logspace(1e-3, 1e4, 300);

    // trajectories[i] = sequence of (re, im) for root i across K
    let mut trajectories: Vec<Vec<(f64, f64)>> =
        ol_poles.iter().map(|p| vec![(p.re, p.im)]).collect();
    let mut prev_roots: Vec<C64> = ol_poles.clone();

    for &k in &k_vals {
        let combined = poly_add_scaled(&den, &num, k);
        let roots_val = match builtin_roots(vec![real_poly_to_value(&combined)]) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let roots: Vec<C64> = match roots_val {
            Value::Vector(v) => v.to_vec(),
            _ => continue,
        };
        if roots.len() != n_poles { continue; }
        let paired = pair_roots_by_proximity(roots, &prev_roots);
        for (i, r) in paired.iter().enumerate() {
            trajectories[i].push((r.re, r.im));
        }
        prev_roots = paired;
    }

    // Set up figure: full reset, then hold=true so all trajectories accumulate
    FIGURE.with(|fig| {
        let mut f = fig.borrow_mut();
        f.reset();
        let sp = f.current_mut();
        sp.title  = "Root Locus".to_string();
        sp.xlabel = "Real".to_string();
        sp.ylabel = "Imaginary".to_string();
        f.hold = true;
    });

    for (i, traj) in trajectories.iter().enumerate() {
        let x: Vec<f64> = traj.iter().map(|&(re, _)| re).collect();
        let y: Vec<f64> = traj.iter().map(|&(_, im)| im).collect();
        push_xy_line(x, y, &format!("root {}", i + 1), "", Some(SeriesColor::cycle(i)), LineStyle::Solid);
    }

    render_figure_terminal().map_err(|e| ScriptError::Runtime(e.to_string()))?;
    Ok(Value::None)
}

fn builtin_margin(args: Vec<Value>) -> Result<Value, ScriptError> {
    check_args("margin", &args, 1)?;
    let (num, den) = match &args[0] {
        Value::TransferFn { num, den } => (num.clone(), den.clone()),
        other => return Err(ScriptError::Type(format!(
            "margin: expected tf, got {}", other.type_name()
        ))),
    };

    // Dense grid for accurate crossing detection
    let poles = builtin_roots(vec![real_poly_to_value(&den)])?;
    let w_nat = match &poles {
        Value::Vector(v) if !v.is_empty() =>
            v.iter().map(|c| c.norm()).fold(0.0f64, f64::max).max(1.0),
        _ => 1.0,
    };
    let w_vec = logspace((w_nat * 0.001).max(1e-4), w_nat * 1000.0, 1000);
    let (mag_db, phase_deg) = bode_compute(&num, &den, &w_vec);

    // Gain crossover: |H| = 0 dB
    let wcp = find_crossing(&w_vec, &mag_db, 0.0);

    // Phase crossover: phase = -180°
    let wcg = find_crossing(&w_vec, &phase_deg, -180.0);

    // Gain margin = 1 / |H(jWcg)|
    let gm = if let Some(wc) = wcg {
        let jw = Complex::new(0.0, wc);
        let h = poly_eval_c(&num, jw) / poly_eval_c(&den, jw);
        if h.norm() > 1e-30 { 1.0 / h.norm() } else { f64::INFINITY }
    } else {
        f64::INFINITY
    };

    // Phase margin = 180° + ∠H(jWcp)
    let pm = if let Some(wc) = wcp {
        let jw = Complex::new(0.0, wc);
        let h = poly_eval_c(&num, jw) / poly_eval_c(&den, jw);
        180.0 + h.arg().to_degrees()
    } else {
        f64::INFINITY
    };

    Ok(Value::Tuple(vec![
        Value::Scalar(gm),
        Value::Scalar(pm),
        Value::Scalar(wcg.unwrap_or(f64::INFINITY)),
        Value::Scalar(wcp.unwrap_or(f64::INFINITY)),
    ]))
}
