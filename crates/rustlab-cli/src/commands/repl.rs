use anyhow::Result;
use rustyline::{error::ReadlineError, DefaultEditor};
use rustlab_script::{lexer, parser, Evaluator};

// ─── Help text ────────────────────────────────────────────────────────────────

struct HelpEntry {
    name:    &'static str,
    brief:   &'static str,
    detail:  &'static str,
}

const HELP: &[HelpEntry] = &[
    // Math
    HelpEntry { name: "abs",    brief: "Absolute value / magnitude",
        detail: "abs(x)  — scalar, complex, or vector\n  Returns magnitude for complex values." },
    HelpEntry { name: "angle",  brief: "Phase angle in radians",
        detail: "angle(x)  — scalar, complex, or vector\n  Returns the argument of a complex number." },
    HelpEntry { name: "real",   brief: "Real part",
        detail: "real(x)  — scalar, complex, or vector" },
    HelpEntry { name: "imag",   brief: "Imaginary part",
        detail: "imag(x)  — scalar, complex, or vector" },
    HelpEntry { name: "cos",    brief: "Cosine",        detail: "cos(x)  — element-wise, accepts complex" },
    HelpEntry { name: "sin",    brief: "Sine",          detail: "sin(x)  — element-wise, accepts complex" },
    HelpEntry { name: "sqrt",   brief: "Square root",   detail: "sqrt(x)  — element-wise, accepts complex" },
    HelpEntry { name: "exp",    brief: "Exponential",   detail: "exp(x)  — element-wise, accepts complex" },
    HelpEntry { name: "log",    brief: "Natural log",   detail: "log(x)  — element-wise (natural log), accepts complex" },
    HelpEntry { name: "log10",  brief: "Base-10 log",   detail: "log10(x)  — element-wise base-10 logarithm, accepts complex" },
    HelpEntry { name: "log2",   brief: "Base-2 log",    detail: "log2(x)  — element-wise base-2 logarithm, accepts complex" },
    // Array / stats
    HelpEntry { name: "zeros",    brief: "Vector of zeros",
        detail: "zeros(n)  — returns a length-n complex zero vector" },
    HelpEntry { name: "ones",     brief: "Vector of ones",
        detail: "ones(n)  — returns a length-n complex one vector" },
    HelpEntry { name: "linspace", brief: "Linearly spaced vector",
        detail: "linspace(start, stop, n)  — n evenly spaced real values from start to stop" },
    HelpEntry { name: "rand",  brief: "Uniform random vector  [0, 1)",
        detail: "rand(n)  — n samples drawn uniformly from [0, 1)" },
    HelpEntry { name: "randn", brief: "Normal random vector  (mean 0, std 1)",
        detail: "randn(n)  — n samples from a standard normal distribution (μ=0, σ=1)" },
    HelpEntry { name: "randi", brief: "Random integer(s) in a range",
        detail: "randi(imax)        — single integer in [1, imax]\nrandi(imax, n)     — n integers in [1, imax]\nrandi([lo,hi], n)  — n integers in [lo, hi] (inclusive)" },
    HelpEntry { name: "min",  brief: "Minimum value of a vector",
        detail: "min(v)  — smallest real value in the vector" },
    HelpEntry { name: "max",  brief: "Maximum value of a vector",
        detail: "max(v)  — largest real value in the vector" },
    HelpEntry { name: "mean", brief: "Mean (average) of a vector",
        detail: "mean(v)  — arithmetic mean; returns a complex scalar for complex vectors" },
    HelpEntry { name: "std",  brief: "Standard deviation of a vector  (N-1 denominator)",
        detail: "std(v)  — sample standard deviation (Bessel-corrected, N-1 denominator)" },
    HelpEntry { name: "histogram", brief: "Histogram — plot and return bin counts",
        detail: "histogram(v)        — 10 bins (default)\nhistogram(v, n)     — n bins\nReturns 2×n matrix: row 1 = bin centers, row 2 = counts" },
    HelpEntry { name: "savehist", brief: "Save histogram to PNG or SVG file",
        detail: "savehist(v, \"file.svg\")             — 10 bins\nsavehist(v, \"file.svg\", \"title\")    — 10 bins with title\nsavehist(v, n, \"file.svg\")          — n bins\nsavehist(v, n, \"file.svg\", \"title\") — n bins with title" },
    HelpEntry { name: "len",      brief: "Length of vector/string  (alias: length)",
        detail: "len(x)  — number of elements in a vector, rows in a matrix, or chars in a string" },
    HelpEntry { name: "length",   brief: "Alias for len",
        detail: "length(x)  — see len" },
    HelpEntry { name: "numel",    brief: "Total number of elements",
        detail: "numel(x)  — total elements (rows*cols for matrices, 1 for scalars)" },
    HelpEntry { name: "size",     brief: "Dimensions as a 2-element vector",
        detail: "size(x)  — returns [rows, cols]; vectors return [1, n]" },
    // Matrix
    HelpEntry { name: "eye",       brief: "Identity matrix",
        detail: "eye(n)  — returns an n×n identity matrix" },
    HelpEntry { name: "transpose", brief: "Non-conjugate transpose  (also: A.')",
        detail: "transpose(A)  — transposes rows and cols without conjugating\n  Equivalent to the postfix operator A.'" },
    HelpEntry { name: "diag",      brief: "Create diagonal matrix or extract diagonal",
        detail: "diag(v)  — creates an n×n diagonal matrix from vector v\ndiag(M)  — extracts the main diagonal of matrix M as a vector" },
    HelpEntry { name: "trace",     brief: "Sum of the main diagonal",
        detail: "trace(M)  — returns the sum of diagonal elements" },
    HelpEntry { name: "reshape",   brief: "Reshape a vector or matrix",
        detail: "reshape(A, m, n)  — returns an m×n matrix filled column-major from A\n  Total elements must be preserved." },
    HelpEntry { name: "repmat",    brief: "Tile a matrix",
        detail: "repmat(A, m, n)  — tiles matrix A m times vertically, n times horizontally" },
    HelpEntry { name: "horzcat",   brief: "Horizontal concatenation  (also: [A B])",
        detail: "horzcat(A, B, ...)  — concatenates matrices side by side (same row count required)" },
    HelpEntry { name: "vertcat",   brief: "Vertical concatenation  (also: [A; B])",
        detail: "vertcat(A, B, ...)  — stacks matrices vertically (same column count required)" },
    // Linear algebra
    HelpEntry { name: "dot",      brief: "Inner (dot) product of two vectors",
        detail: "dot(u, v)  — sum of element-wise products; conjugates u for complex vectors" },
    HelpEntry { name: "cross",    brief: "3-element cross product",
        detail: "cross(u, v)  — both vectors must have exactly 3 elements" },
    HelpEntry { name: "norm",     brief: "Euclidean norm of a vector or Frobenius norm of a matrix",
        detail: "norm(v)  — L2 norm of a vector\nnorm(M)  — Frobenius norm of a matrix" },
    HelpEntry { name: "det",      brief: "Determinant of a square matrix",
        detail: "det(M)  — computed via LU decomposition with partial pivoting" },
    HelpEntry { name: "inv",      brief: "Inverse of a square matrix",
        detail: "inv(M)  — computed via Gauss-Jordan elimination; errors on singular matrices" },
    HelpEntry { name: "linsolve", brief: "Solve the linear system  A*x = b",
        detail: "linsolve(A, b)  — A is n×n, b is a length-n vector\n  Returns x as a vector." },
    HelpEntry { name: "eig",      brief: "Eigenvalues of a square matrix",
        detail: "eig(M)  — returns a complex vector of eigenvalues\n  Uses QR iteration via Hessenberg reduction." },
    HelpEntry { name: "factor",   brief: "Prime factorization of a positive integer",
        detail: "factor(n)  — returns a real vector of prime factors in ascending order\n  factor(12) → [2, 2, 3]\n  factor(17) → [17]" },
    // DSP
    HelpEntry { name: "fir_lowpass",  brief: "FIR low-pass filter coefficients",
        detail: "fir_lowpass(taps, cutoff_hz, sample_rate, window)\n  window: \"hann\", \"hamming\", \"blackman\", \"rectangular\", \"kaiser\"" },
    HelpEntry { name: "fir_highpass", brief: "FIR high-pass filter coefficients",
        detail: "fir_highpass(taps, cutoff_hz, sample_rate, window)" },
    HelpEntry { name: "fir_bandpass", brief: "FIR band-pass filter coefficients",
        detail: "fir_bandpass(taps, low_hz, high_hz, sample_rate, window)" },
    HelpEntry { name: "butterworth_lowpass",  brief: "Butterworth IIR low-pass (returns b coefficients)",
        detail: "butterworth_lowpass(order, cutoff_hz, sample_rate)" },
    HelpEntry { name: "butterworth_highpass", brief: "Butterworth IIR high-pass (returns b coefficients)",
        detail: "butterworth_highpass(order, cutoff_hz, sample_rate)" },
    HelpEntry { name: "convolve", brief: "Linear convolution of two vectors",
        detail: "convolve(x, h)  — returns x convolved with h" },
    HelpEntry { name: "window",   brief: "Generate a window function vector",
        detail: "window(name, n)  — name: \"hann\", \"hamming\", \"blackman\", \"rectangular\", \"kaiser\"" },
    // FFT
    HelpEntry { name: "fft",      brief: "Forward FFT (zero-pads to next power of two)",
        detail: "fft(v)  — returns complex spectrum; length is next power of two >= len(v)" },
    HelpEntry { name: "ifft",     brief: "Inverse FFT",
        detail: "ifft(V)  — input length must be a power of two (as returned by fft)" },
    HelpEntry { name: "fftshift", brief: "Shift zero-frequency component to center",
        detail: "fftshift(V)  — rearranges FFT output so DC is at the center" },
    HelpEntry { name: "fftfreq",  brief: "FFT frequency axis",
        detail: "fftfreq(n, sample_rate)  — frequency bin values for an n-point FFT" },
    HelpEntry { name: "spectrum", brief: "DC-centered spectrum matrix ready for plotdb/savedb",
        detail: "spectrum(X, sample_rate)  — applies fftshift and pairs with Hz frequency axis\n  Returns 2×n matrix: row 1 = Hz (DC centered), row 2 = complex spectrum\n  Pass directly to plotdb() or savedb()" },
    // Kaiser FIR
    HelpEntry { name: "fir_lowpass_kaiser",  brief: "Auto-designed Kaiser lowpass FIR",
        detail: "fir_lowpass_kaiser(cutoff_hz, trans_bw_hz, stopband_attn_db, sample_rate)\n  Beta and tap count computed automatically from attenuation and transition bandwidth." },
    HelpEntry { name: "fir_highpass_kaiser", brief: "Auto-designed Kaiser highpass FIR",
        detail: "fir_highpass_kaiser(cutoff_hz, trans_bw_hz, stopband_attn_db, sample_rate)" },
    HelpEntry { name: "fir_bandpass_kaiser", brief: "Auto-designed Kaiser bandpass FIR",
        detail: "fir_bandpass_kaiser(low_hz, high_hz, trans_bw_hz, stopband_attn_db, sample_rate)" },
    HelpEntry { name: "fir_notch", brief: "FIR notch filter (spectral inversion of bandpass)",
        detail: "fir_notch(center_hz, bandwidth_hz, sample_rate, num_taps, window)\n  Rejects a narrow band around center_hz." },
    // Fixed-point quantization
    HelpEntry { name: "qfmt", brief: "Create a Q-format spec (word bits, frac bits, rounding, overflow)",
        detail: "qfmt(word_bits, frac_bits)\nqfmt(word_bits, frac_bits, round_mode)\nqfmt(word_bits, frac_bits, round_mode, overflow_mode)\n\n  round_mode:    floor (default/hardware), ceil, zero, round, round_even\n  overflow_mode: saturate (default), wrap\n\nExample:\n  fmt = qfmt(16, 15, \"round_even\", \"saturate\")  # Q0.15, 16-bit" },
    HelpEntry { name: "quantize", brief: "Quantize a scalar / vector / matrix to a Q-format grid",
        detail: "quantize(x, fmt)\n  x   — scalar, complex, vector, or matrix\n  fmt — QFmt spec from qfmt()\n\nReal and imaginary parts are quantized independently.\nReturns same type as input — compatible with all math and plot functions.\n\nExample:\n  fmt = qfmt(16, 15)\n  xq  = quantize(x, fmt)" },
    HelpEntry { name: "qadd", brief: "Fixed-point element-wise add, quantized to fmt",
        detail: "qadd(a, b, fmt)\n  a, b — real scalars or vectors (same length)\n  fmt  — output QFmt spec\n\nComputes a+b at full precision, then quantizes to fmt.\n\nExample:\n  y = qadd(xq, offset, fmt)" },
    HelpEntry { name: "qmul", brief: "Fixed-point element-wise multiply, quantized to fmt",
        detail: "qmul(a, b, fmt)\n  a, b — real scalars or vectors (same length)\n  fmt  — output QFmt spec\n\nFull Q-product computed internally; result rounded to fmt.\n\nExample:\n  y = qmul(xq, gain, fmt)" },
    HelpEntry { name: "qconv", brief: "Fixed-point FIR convolution, output quantized to fmt",
        detail: "qconv(x, h, fmt)\n  x   — input signal (real vector)\n  h   — filter coefficients (real vector)\n  fmt — output QFmt spec\n\nAccumulates products at full precision, then quantizes each output.\nOutput length = len(x) + len(h) - 1.\n\nExample:\n  y = qconv(xq, hq, fmt)" },
    HelpEntry { name: "snr", brief: "Signal-to-noise ratio in dB between reference and quantized signal",
        detail: "snr(x_ref, x_quantized)\n  Both must be real vectors of equal length.\n  Returns 10*log10(signal_power / noise_power) in dB.\n  Returns Inf when signals are identical.\n\nExample:\n  db = snr(y_ref, y_q)\n  print(db)" },
    HelpEntry { name: "firpm",    brief: "Parks-McClellan optimal equiripple FIR filter",
        detail: "firpm(n_taps, bands, desired)\nfirpm(n_taps, bands, desired, weights)\n  bands   — normalized frequency edges [0,1], 1 = Nyquist; pairs define each band\n  desired — target amplitude at each band edge (piecewise-linear)\n  weights — optional, one value per band pair (default: all 1.0)\n  Example (lowpass): firpm(63, [0,0.2,0.3,1], [1,1,0,0])" },
    HelpEntry { name: "freqz",    brief: "Complex frequency response of a filter",
        detail: "freqz(h, n_points, sample_rate)  — returns 2×n matrix: row 1 = freq axis, row 2 = H(f)" },
    // Plotting
    HelpEntry { name: "print", brief: "Print values to stdout",
        detail: "print(a, b, ...)  — prints space-separated values followed by newline" },
    HelpEntry { name: "plot",  brief: "Plot a vector in the terminal",
        detail: "plot(v)  or  plot(v, \"title\")  — opens a ratatui terminal chart; press any key to close\n  plot(v, \"title\", \"color\")  — color: r, g, b, c, m, y, k, w\n  plot(v, \"title\", \"color\", \"dashed\")" },
    HelpEntry { name: "stem",  brief: "Stem plot of a vector",
        detail: "stem(v)  or  stem(v, \"title\")  — discrete-sample stem chart" },
    HelpEntry { name: "plotdb",   brief: "Terminal dB frequency response plot",
        detail: "plotdb(Hz)  or  plotdb(Hz, \"title\")\n  Hz is the 2×n matrix returned by freqz()\n  x-axis: Hz, y-axis: dB magnitude" },
    HelpEntry { name: "savefig",  brief: "Save a line plot to PNG or SVG",
        detail: "savefig(v, \"file.svg\")  or  savefig(v, \"file.png\", \"title\")\n  Extension determines format: .svg or .png" },
    HelpEntry { name: "savestem", brief: "Save a stem plot to PNG or SVG",
        detail: "savestem(v, \"file.svg\")  or  savestem(v, \"file.png\", \"title\")" },
    HelpEntry { name: "savedb",   brief: "Save a dB frequency response plot to PNG or SVG",
        detail: "savedb(Hz, \"file.svg\")  or  savedb(Hz, \"file.png\", \"title\")\n  Hz is the 2×n matrix from freqz()" },
    HelpEntry { name: "imagesc",  brief: "Display matrix as a colour heatmap in the terminal",
        detail: "imagesc(M)\nimagesc(M, colormap)\n  colormap: \"viridis\" (default), \"jet\", \"hot\", \"gray\"\n  Press any key to close." },
    HelpEntry { name: "saveimagesc", brief: "Save matrix heatmap to PNG or SVG",
        detail: "saveimagesc(M, \"file.png\")\nsaveimagesc(M, \"file.png\", \"title\")\nsaveimagesc(M, \"file.png\", \"title\", colormap)" },
    // Figure controls
    HelpEntry { name: "figure",   brief: "Reset figure state (clear all subplots)",
        detail: "figure()  — clears all series, titles, and subplot layout; starts a fresh figure" },
    HelpEntry { name: "hold",     brief: "Keep existing series when adding new ones",
        detail: "hold(\"on\")   — subsequent plot() calls overlay on the current subplot\nhold(\"off\")  — each plot() replaces the previous series (default)\nhold(1) / hold(0) also accepted" },
    HelpEntry { name: "grid",     brief: "Show or hide grid lines",
        detail: "grid(\"on\")   — enable grid lines (default)\ngrid(\"off\")  — disable grid lines\ngrid(1) / grid(0) also accepted" },
    HelpEntry { name: "xlabel",   brief: "Set x-axis label",
        detail: "xlabel(\"text\")  — sets the x-axis label on the current subplot" },
    HelpEntry { name: "ylabel",   brief: "Set y-axis label",
        detail: "ylabel(\"text\")  — sets the y-axis label on the current subplot" },
    HelpEntry { name: "title",    brief: "Set subplot title",
        detail: "title(\"text\")  — sets the title of the current subplot" },
    HelpEntry { name: "xlim",     brief: "Set x-axis limits",
        detail: "xlim([lo, hi])  — fixes the x-axis range on the current subplot" },
    HelpEntry { name: "ylim",     brief: "Set y-axis limits",
        detail: "ylim([lo, hi])  — fixes the y-axis range on the current subplot" },
    HelpEntry { name: "subplot",  brief: "Switch to a subplot panel",
        detail: "subplot(rows, cols, idx)  — divides the figure into rows×cols panels\n  idx is 1-based, counts left-to-right then top-to-bottom\n  Example: subplot(2, 1, 1)  — top panel of a 2-row layout" },
    HelpEntry { name: "legend",   brief: "Label series in the current subplot",
        detail: "legend(\"s1\", \"s2\", ...)  — assigns labels to series in the order they were added\n  Labels appear in the chart legend." },
    // I/O
    HelpEntry { name: "save", brief: "Save a variable to NPY, NPZ, or CSV",
        detail: "save(\"file.npy\", x)                          — single array, NumPy format\nsave(\"file.csv\", x)                          — single array, CSV text\nsave(\"file.npz\", \"a\", a, \"b\", b, ...)        — multiple named arrays\n\nNPY/NPZ files are compatible with numpy.load() in Python." },
    HelpEntry { name: "load", brief: "Load variables from NPY, NPZ, or CSV",
        detail: "load(\"file.npz\")              — loads ALL variables into the workspace (bare call only)\nload(\"file.npz\", \"varname\")   — returns one named array from the archive\nload(\"file.npy\")              — returns the array as a value\nload(\"file.csv\")              — returns scalar / vector / matrix" },
    HelpEntry { name: "whos", brief: "List workspace variables or inspect an NPZ file",
        detail: "whos                          — list all workspace variables\nwhos(\"file.npz\")              — list arrays stored in an NPZ file\n  Shows name, type (real/complex), and size for each array." },
    // Language
    HelpEntry { name: "i / j", brief: "Imaginary unit constant  (0 + 1i)",
        detail: "i and j are both pre-defined constants equal to sqrt(-1)\n  Example: z = 3 + j*4   or   z = 3 + i*4" },
    HelpEntry { name: "pi",   brief: "π  (3.14159…)",  detail: "pi  — pre-defined constant" },
    HelpEntry { name: "e",    brief: "Euler's number (2.71828…)", detail: "e  — pre-defined constant" },
    HelpEntry { name: "range", brief: "Range syntax: start:stop  or  start:step:stop",
        detail: "1:5       → [1, 2, 3, 4, 5]\n0:0.5:2   → [0, 0.5, 1.0, 1.5, 2.0]\nUse v(end) for last element." },
    HelpEntry { name: "index", brief: "1-based indexing: v(i)  or  v(1:3)",
        detail: "v(1)      — first element\nv(end)    — last element\nv(2:4)    — elements 2 through 4" },
    HelpEntry { name: "clear", brief: "Remove all variables from the session",
        detail: "clear  — deletes every user-defined variable; built-in constants (j, pi, e) are kept" },
    // Structs
    HelpEntry { name: "struct", brief: "Create a struct from field-value pairs",
        detail: "struct(\"x\", 1, \"y\", 2)  — creates a struct with fields x=1, y=2\n  Access: s.x\n  Assign: s.z = 3  (auto-creates struct if s is undefined)" },
    HelpEntry { name: "isstruct", brief: "Test if a value is a struct",
        detail: "isstruct(x)  — returns true if x is a struct, false otherwise" },
    HelpEntry { name: "fieldnames", brief: "List field names of a struct",
        detail: "fieldnames(s)  — prints all field names of struct s" },
    HelpEntry { name: "isfield", brief: "Test if a struct has a given field",
        detail: "isfield(s, \"x\")  — returns true if struct s has field 'x'" },
    HelpEntry { name: "rmfield", brief: "Remove a field from a struct (returns new struct)",
        detail: "s2 = rmfield(s, \"x\")  — returns a copy of s with field 'x' removed" },
    // Output
    HelpEntry { name: "disp", brief: "Display a value (always prints newline)",
        detail: "disp(x)  — prints x followed by a newline\n  Equivalent to print(x) but guaranteed to end with \\n." },
    HelpEntry { name: "fprintf", brief: "Formatted print (C-style)",
        detail: "fprintf(fmt, arg1, arg2, ...)\n  Specifiers: %d %i %f %g %e %s %%\n  Escapes:    \\n \\t \\\\\n  Width/precision: %8.2f  %-10s\n  Example: fprintf(\"x = %.3f\\n\", 3.14159)" },
    // Aggregates
    HelpEntry { name: "all", brief: "True if all elements are nonzero",
        detail: "all(v)  — true if every element of v is nonzero\n  Works on scalars, bools, and vectors." },
    HelpEntry { name: "any", brief: "True if any element is nonzero",
        detail: "any(v)  — true if at least one element of v is nonzero" },
    // Matrix analysis
    HelpEntry { name: "rank", brief: "Matrix rank (SVD threshold)",
        detail: "rank(M)  — number of linearly independent rows/columns\n  Uses SVD-based threshold: eps * max(size) * max_sv" },
    HelpEntry { name: "roots", brief: "Roots of a polynomial",
        detail: "roots(p)  — roots of polynomial with coefficients p (descending power)\n  roots([1, -3, 2])  →  [2, 1]  (roots of x²-3x+2)\n  roots([1, 2, 10])  →  [-1+3j, -1-3j]" },
    // Control Systems
    HelpEntry { name: "tf", brief: "Create a transfer function",
        detail: "tf(\"s\")           — Laplace variable s\ntf(num, den)      — TF from numerator/denominator coefficient vectors (descending power)\n\nExample:\n  s = tf(\"s\")\n  G = 10 / (s^2 + 2*s + 10)\n  G = tf([10], [1, 2, 10])   % equivalent" },
    HelpEntry { name: "pole", brief: "Poles of a transfer function",
        detail: "pole(G)  — complex vector of closed-loop poles (roots of denominator)\n\nExample:\n  G = tf([10], [1, 2, 10])\n  p = pole(G)  % ≈ [-1+3j, -1-3j]" },
    HelpEntry { name: "zero", brief: "Zeros of a transfer function",
        detail: "zero(G)  — complex vector of transmission zeros (roots of numerator)\n\nExample:\n  G = tf([1, 1], [1, 2, 10])\n  z = zero(G)  % ≈ -1" },
    HelpEntry { name: "ss", brief: "Convert transfer function to state-space",
        detail: "ss(G)  — observable canonical form state-space {A, B, C, D}\n\nAccess fields: sys.A, sys.B, sys.C, sys.D\n\nExample:\n  G   = tf([10], [1, 2, 10])\n  sys = ss(G)" },
    HelpEntry { name: "ctrb", brief: "Controllability matrix",
        detail: "ctrb(A, B)  — [B, AB, A²B, …]  (n × n·p matrix)\n\nFull column rank ↔ system is controllable.\n\nExample:\n  sys = ss(G)\n  Wc  = ctrb(sys.A, sys.B)\n  rank(Wc)   % should equal n for controllable system" },
    HelpEntry { name: "obsv", brief: "Observability matrix",
        detail: "obsv(A, C)  — [C; CA; CA²; …]  (n·q × n matrix)\n\nFull row rank ↔ system is observable.\n\nExample:\n  sys = ss(G)\n  Wo  = obsv(sys.A, sys.C)\n  rank(Wo)" },
    HelpEntry { name: "bode", brief: "Bode magnitude and phase plot",
        detail: "bode(G)         — plot magnitude (dB) and phase (deg) vs log10(ω)\nbode(G, w)      — use supplied frequency vector w (rad/s)\n[mag, ph, w] = bode(G)  — return data without plotting\n\nExample:\n  G = tf([10], [1, 2, 10])\n  bode(G)\n  [m, p, w] = bode(G)" },
    HelpEntry { name: "step", brief: "Step response plot",
        detail: "step(G)              — plot unit step response\n[y, t] = step(G)     — return output and time vectors\n[y, t] = step(G, tf) — specify final time\n\nExample:\n  G = tf([10], [1, 2, 10])\n  step(G)\n  [y, t] = step(G, 5)" },
    HelpEntry { name: "margin", brief: "Gain and phase margins",
        detail: "[Gm, Pm, Wcg, Wcp] = margin(G)\n  Gm  — gain margin (linear ratio)\n  Pm  — phase margin (degrees)\n  Wcg — gain crossover frequency (rad/s)\n  Wcp — phase crossover frequency (rad/s)\n\nExample:\n  G = tf([10], [1, 2, 10])\n  [Gm, Pm, Wcg, Wcp] = margin(G)" },
    HelpEntry { name: "lqr", brief: "Linear-Quadratic Regulator design",
        detail: "[K, S, e] = lqr(sys, Q, R)\n  sys — state-space system (from ss())\n  Q   — state weighting matrix (n×n, positive semi-definite)\n  R   — input weighting matrix (m×m, positive definite)\n  K   — optimal gain matrix\n  S   — Riccati solution (cost matrix)\n  e   — closed-loop eigenvalues\n\nSolves the continuous-time algebraic Riccati equation (CARE).\n\nExample:\n  sys = ss(tf([1], [1, 0, 0]))   % double integrator\n  [K, S, e] = lqr(sys, eye(2), 1)" },
    HelpEntry { name: "rlocus", brief: "Root locus plot",
        detail: "rlocus(G)  — plot closed-loop pole trajectories as loop gain K sweeps 0 → ∞\n\nEach coloured path shows where one pole moves as K increases.\nOpen-loop poles are the starting points (K=0).\n\nExample:\n  s = tf(\"s\")\n  G = 1 / (s * (s + 1))\n  rlocus(G)" },
    // Control flow
    HelpEntry { name: "if", brief: "Conditional branching",
        detail: "if cond\n  body\nend\n\nif cond\n  then_body\nelse\n  else_body\nend\n\nCondition may be a Bool or scalar (0 = false, nonzero = true)." },
    // User-defined functions
    HelpEntry { name: "function", brief: "Define a named function",
        detail: "function y = foo(x)\n  y = x * 2\nend\n\nfunction bar(a, b)\n  print(a + b)\nend\n\nSyntax:\n  function retvar = name(param1, param2, ...)\n    body\n  end\n  function name(param, ...)   % no return value\n    body\n  end\n\nuse 'return' to exit early." },
    // Filesystem / script loading
    HelpEntry { name: "run", brief: "Run a .r script file in the current session",
        detail: "run <file>  — execute a script file; its variables remain available afterward\n  Example: run examples/fft.r" },
    HelpEntry { name: "ls",  brief: "List directory contents",
        detail: "ls          — list current directory\nls <path>   — list the given directory" },
    HelpEntry { name: "cd",  brief: "Change working directory",
        detail: "cd          — change to home directory\ncd <path>   — change to the given path" },
    HelpEntry { name: "pwd", brief: "Print working directory",
        detail: "pwd  — show the current working directory" },
];

fn whos_type(v: &rustlab_script::Value) -> &'static str {
    use rustlab_script::Value;
    match v {
        Value::Scalar(_)  => "scalar",
        Value::Complex(_) => "complex",
        Value::Vector(_)  => "vector",
        Value::Matrix(_)  => "matrix",
        Value::Bool(_)    => "bool",
        Value::Str(_)     => "string",
        Value::QFmt(_)    => "qfmt",
        Value::Struct(_)  => "struct",
        Value::Tuple(_)   => "tuple",
        Value::All        => "all-index",
        Value::None       => "none",
        Value::TransferFn { .. } => "tf",
        Value::StateSpace { .. } => "ss",
    }
}

fn whos_size(v: &rustlab_script::Value) -> String {
    use rustlab_script::Value;
    match v {
        Value::Vector(v)  => format!("1×{}", v.len()),
        Value::Matrix(m)  => format!("{}×{}", m.nrows(), m.ncols()),
        Value::Str(s)     => format!("1×{}", s.len()),
        Value::Struct(f)      => format!("1×1 ({} fields)", f.len()),
        Value::Tuple(v)       => format!("1×{}", v.len()),
        Value::StateSpace { A, .. } => format!("{}×{}", A.nrows(), A.ncols()),
        Value::All            => "—".to_string(),
        _                     => "1×1".to_string(),
    }
}

fn whos_preview(v: &rustlab_script::Value) -> String {
    use rustlab_script::Value;
    match v {
        Value::Scalar(n)  => format!("{n}"),
        Value::Complex(c) => {
            if c.im >= 0.0 {
                format!("{}+{}j", c.re, c.im)
            } else {
                format!("{}{}j", c.re, c.im)
            }
        }
        Value::Bool(b)    => format!("{b}"),
        Value::Str(s)     => {
            if s.len() <= 40 { format!("\"{s}\"") }
            else             { format!("\"{}…\"", &s[..37]) }
        }
        Value::Vector(v) => {
            let preview: Vec<String> = v.iter().take(3)
                .map(|c| if c.im == 0.0 { format!("{:.4}", c.re) }
                         else            { format!("{:.4}+{:.4}j", c.re, c.im) })
                .collect();
            let suffix = if v.len() > 3 { ", …" } else { "" };
            format!("[{}{}]", preview.join(", "), suffix)
        }
        Value::Matrix(m)  => format!("[{}×{} matrix]", m.nrows(), m.ncols()),
        Value::Struct(f)  => {
            let mut names: Vec<&str> = f.keys().map(|s| s.as_str()).collect();
            names.sort();
            let preview = names.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
            let suffix = if names.len() > 3 { ", …" } else { "" };
            format!("{{{}{}}}",  preview, suffix)
        }
        Value::QFmt(spec) => format!("{}", rustlab_script::Value::QFmt(spec.clone())),
        Value::Tuple(v)   => format!("({} values)", v.len()),
        Value::All        => ":".to_string(),
        Value::None       => "none".to_string(),
        Value::TransferFn { num, den } => format!(
            "{} / ({} terms)",
            num.len(), den.len()
        ),
        Value::StateSpace { A, B, C, .. } => format!(
            "{}-state, {} input, {} output",
            A.nrows(), B.ncols(), C.nrows()
        ),
    }
}

fn print_whos(ev: &rustlab_script::Evaluator) {
    let vars = ev.vars();
    if vars.is_empty() {
        println!("  (no variables defined)");
        return;
    }
    println!();
    println!("  {:<16}  {:<10}  {:<8}  {}", "Name", "Type", "Size", "Value");
    println!("  {}", "─".repeat(70));
    for (name, val) in &vars {
        println!("  {:<16}  {:<10}  {:<8}  {}",
            name,
            whos_type(val),
            whos_size(val),
            whos_preview(val),
        );
    }
    println!();
}

fn cmd_pwd() {
    match std::env::current_dir() {
        Ok(p)  => println!("{}", p.display()),
        Err(e) => eprintln!("pwd: {e}"),
    }
}

fn cmd_cd(path: &str) {
    let target = if path.is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    } else {
        path.to_string()
    };
    if let Err(e) = std::env::set_current_dir(&target) {
        eprintln!("cd: {target}: {e}");
    }
}

fn cmd_ls(path: &str) {
    let target = if path.is_empty() { "." } else { path };
    let dir = std::path::Path::new(target);

    let mut entries = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>(),
        Err(e) => { eprintln!("ls: {target}: {e}"); return; }
    };
    entries.sort_by_key(|e| e.file_name());

    let mut dirs:  Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            dirs.push(format!("{name}/"));
        } else {
            files.push(name);
        }
    }

    // Print directories first, then files, in columns of 4
    let all: Vec<String> = dirs.into_iter().chain(files).collect();
    if all.is_empty() { return; }

    let col_w = all.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
    let cols  = (80 / col_w).max(1);

    println!();
    for (i, name) in all.iter().enumerate() {
        if i > 0 && i % cols == 0 { println!(); }
        print!("  {:<width$}", name, width = col_w);
    }
    println!("\n");
}

fn print_help_list() {
    println!();
    println!("  {:<26}  {}", "Command / Topic", "Description");
    println!("  {}", "-".repeat(60));

    let categories = [
        ("Math",             &["abs","angle","real","imag","cos","sin","sqrt","exp","log","log10","log2"][..]),
        ("Array / Stats",    &["zeros","ones","linspace","rand","randn","randi",
                               "min","max","mean","std","histogram","savehist",
                               "len","length","numel","size"]),
        ("Matrix",           &["eye","transpose","diag","trace","reshape","repmat",
                               "horzcat","vertcat","rank"]),
        ("Linear Algebra",   &["dot","cross","norm","det","inv","linsolve","eig","factor","roots"]),
        ("DSP",              &["fir_lowpass","fir_highpass","fir_bandpass",
                               "fir_lowpass_kaiser","fir_highpass_kaiser","fir_bandpass_kaiser",
                               "fir_notch","firpm","freqz",
                               "butterworth_lowpass","butterworth_highpass","convolve","window",
                               "fft","ifft","fftshift","fftfreq","spectrum"]),
        ("Fixed-point",      &["qfmt","quantize","qadd","qmul","qconv","snr"]),
        ("Plotting",         &["plot","stem","plotdb","imagesc",
                               "savefig","savestem","savedb","saveimagesc","histogram","savehist"]),
        ("Figure Controls",  &["figure","hold","grid","xlabel","ylabel","title",
                               "xlim","ylim","subplot","legend"]),
        ("Controls",         &["tf","pole","zero","ss","ctrb","obsv",
                               "bode","step","margin","lqr","rlocus"]),
        ("Structs",          &["struct","isstruct","fieldnames","isfield","rmfield"]),
        ("Functions",        &["function","if"]),
        ("Output",           &["disp","fprintf","print"]),
        ("I/O",              &["print","save","load","whos"]),
        ("Language / REPL",  &["i / j","pi","e","range","index","clear","whos"]),
        ("Filesystem",       &["run","ls","cd","pwd"]),
    ];

    for (cat, names) in &categories {
        println!("\n  {}:", cat);
        for &n in *names {
            if let Some(e) = HELP.iter().find(|e| e.name == n) {
                println!("    {:<24}  {}", e.name, e.brief);
            }
        }
    }
    println!();
    println!("  Type  help <command>  or  ? <command>  for details.");
    println!();
}

fn print_help_detail(topic: &str) {
    match HELP.iter().find(|e| e.name == topic) {
        Some(e) => {
            println!();
            println!("  {}  —  {}", e.name, e.brief);
            println!();
            for line in e.detail.lines() {
                println!("  {}", line);
            }
            println!();
        }
        None => println!("No help found for '{}'.  Type 'help' for a full list.", topic),
    }
}

// ─── REPL ─────────────────────────────────────────────────────────────────────

pub fn execute() -> Result<()> {
    println!("rustlab {} — type 'help' or '?' for help, 'exit' or Ctrl+D to quit", env!("CARGO_PKG_VERSION"));
    println!("Tip: end a line with ; to suppress output\n");

    let mut rl = DefaultEditor::new()?;
    let mut ev = Evaluator::new();

    let hist_path = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".rustlab_history"))
        .unwrap_or_else(|| std::path::PathBuf::from(".rustlab_history"));
    let _ = rl.load_history(&hist_path);

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                rl.add_history_entry(trimmed).ok();

                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }

                // help / ?
                if trimmed == "help" || trimmed == "?" {
                    print_help_list();
                    continue;
                }
                if let Some(topic) = trimmed.strip_prefix("help ").or_else(|| trimmed.strip_prefix("? ")) {
                    print_help_detail(topic.trim());
                    continue;
                }

                // whos
                if trimmed == "whos" {
                    print_whos(&ev);
                    continue;
                }

                // clear
                if trimmed == "clear" {
                    ev.clear_vars();
                    continue;
                }

                // run <file>
                if let Some(path) = trimmed.strip_prefix("run ") {
                    let path = path.trim();
                    match std::fs::read_to_string(path) {
                        Err(e) => eprintln!("run: {path}: {e}"),
                        Ok(src) => {
                            match lexer::tokenize(&src).and_then(|t| parser::parse(t)) {
                                Err(e) => eprintln!("error: {e}"),
                                Ok(stmts) => {
                                    for stmt in &stmts {
                                        if let Err(e) = ev.exec_stmt(stmt) {
                                            eprintln!("error: {e}");
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    continue;
                }

                // directory commands
                if trimmed == "pwd" {
                    cmd_pwd();
                    continue;
                }
                if trimmed == "cd" {
                    cmd_cd("");
                    continue;
                }
                if let Some(path) = trimmed.strip_prefix("cd ") {
                    cmd_cd(path.trim());
                    continue;
                }
                if trimmed == "ls" {
                    cmd_ls("");
                    continue;
                }
                if let Some(path) = trimmed.strip_prefix("ls ") {
                    cmd_ls(path.trim());
                    continue;
                }

                // Multi-line input for function definitions
                let source = if trimmed.starts_with("function ") || trimmed == "function" {
                    let mut buf = format!("{}\n", trimmed);
                    let mut depth: i32 = 1;
                    loop {
                        match rl.readline(".. ") {
                            Ok(cont) => {
                                let ct = cont.trim();
                                rl.add_history_entry(ct).ok();
                                // Track nesting for nested function defs
                                if ct.starts_with("function ") || ct == "function" { depth += 1; }
                                buf.push_str(ct);
                                buf.push('\n');
                                if ct == "end" || ct == "end;" {
                                    depth -= 1;
                                    if depth <= 0 { break; }
                                }
                            }
                            Err(ReadlineError::Interrupted) => { println!("(interrupted)"); break; }
                            Err(_) => break,
                        }
                    }
                    buf
                } else {
                    format!("{}\n", trimmed)
                };

                match lexer::tokenize(&source)
                    .and_then(|tokens| parser::parse(tokens))
                {
                    Ok(stmts) => {
                        for stmt in &stmts {
                            if let Err(e) = ev.exec_stmt(stmt) {
                                eprintln!("error: {}", e);
                            }
                        }
                    }
                    Err(e) => eprintln!("error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C — clear current input, continue
                println!("(interrupted)");
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D — exit
                break;
            }
            Err(e) => {
                eprintln!("readline error: {}", e);
                break;
            }
        }
    }

    let _ = rl.save_history(&hist_path);
    println!("bye");
    Ok(())
}
