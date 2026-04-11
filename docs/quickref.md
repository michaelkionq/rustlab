# rustlab Quick Reference

Concise cheat sheet for the rustlab scripting language. Full signatures and examples: `docs/functions.md`.

Run a script: `rustlab run script.r` — Interactive REPL: `rustlab`

> **For AI agents:** This file is the canonical capability index. Check it to know what functions exist before generating code. It is kept in sync with the actual builtins; if a function is not listed here, it is not implemented.

---

## Language

| Syntax | Description |
|---|---|
| `j`, `i` | Imaginary unit; complex literal: `z = 3.0 + j*4.0` |
| `pi`, `e` | Built-in constants |
| `true`, `false` | Boolean constants — usable in `if` and `while` conditions |
| `v(1)`, `v(end)`, `v(2:4)` | 1-based indexing; `end` = last element; slice returns Vector |
| `v(i) = val`, `M(r,c) = val` | Indexed assignment; vectors auto-grow as needed |
| `f(args)(i)` | Chain call and index without a temporary variable |
| `[a; b; c]` | Column vector literal |
| `[a, b; c, d]` | Matrix literal — `,` same row, `;` new row |
| `[A, B]` / `[A; B]` | Horizontal / vertical concatenation |
| `[X, Y] = f(...)` | Destructuring assignment |
| `1:5`, `0:0.5:2`, `10:-1:1` | Range: `start:stop` or `start:step:stop` |
| `.*`, `./`, `.^` | Element-wise multiply, divide, power |
| `*` | Matrix multiply |
| `'` | Conjugate transpose |
| `.'` | Non-conjugate transpose |
| `;` | Suppress output on a statement |
| `#` | Comment |
| `for i = 1:n` … `end` | For loop; also iterates over a vector |
| `while cond` … `end` | While loop; condition is Bool, Scalar (nonzero), or Complex |
| `if expr` … `elseif expr` … `else` … `end` | Conditional; `elseif` and `else` are optional; nesting supported |
| `function [out] = name(args)` … `end` | User-defined function |
| `return` | Early return from a function |
| `@(x, y) expr` | Anonymous function (lambda); captures current env by snapshot |
| `@name` | Function handle — reference to a builtin or user function |
| `arrayfun(f, v)` | Apply callable to each element; scalar results → Vector, vector results → Matrix |
| `feval("name", args...)` | Call function by string name |
| `profile(fn1, fn2)` | Enable call profiling for named functions; `profile()` tracks all |
| `profile_report()` | Print profiling table to stderr immediately |
| `logspace(a, b, n)` | n log-spaced points from 10^a to 10^b |
| `rk4(f, x0, t)` | Fixed-step 4th-order Runge-Kutta; f(x,t)→x_dot |
| `lyap(A, Q)` | Solve Lyapunov equation A*X + X*A' + Q = 0 |
| `gram(A, B, "c"/"o")` | Controllability or observability Gramian |
| `care(A, B, Q, R)` | Continuous Algebraic Riccati Equation → P |
| `dare(A, B, Q, R)` | Discrete Algebraic Riccati Equation → P |
| `place(A, B, poles)` | Ackermann pole placement (SISO) → K |
| `freqresp(A, B, C, D, w)` | H(jω) from state-space at each frequency ω |
| `svd(A)` | Jacobi SVD → Tuple [U, sigma_vector, V] |
| `s.field` | Struct field access |
| `s.field = val` | Struct field assignment (auto-creates struct) |

---

## Math (all element-wise)

| Function | Description |
|---|---|
| `exp(v)` | $e^v$ |
| `sqrt(v)` | Square root |
| `abs(v)` | Absolute value / modulus |
| `log(v)` | Natural logarithm |
| `log10(v)`, `log2(v)` | Base-10 and base-2 logarithms |
| `sin(v)`, `cos(v)` | Trig (radians) |
| `asin(v)`, `acos(v)`, `atan(v)` | Inverse trig |
| `atan2(y, x)` | Four-quadrant arctangent |
| `tanh(v)`, `sinh(v)`, `cosh(v)` | Hyperbolic trig |
| `floor(v)`, `ceil(v)`, `round(v)` | Rounding (applied to real and imaginary parts independently) |
| `sign(v)` | −1/0/+1 for real; `z/\|z\|` for complex |
| `mod(v, m)` | Modulo: `v − m·floor(v/m)` (m must be a real scalar) |
| `real(v)`, `imag(v)` | Real and imaginary parts |
| `conj(v)` | Complex conjugate — negates imaginary part |
| `angle(v)` | Phase = atan2(Im, Re), element-wise |

---

## Array Construction & Inspection

| Function | Description |
|---|---|
| `linspace(a, b, n)` | n evenly-spaced points from a to b |
| `zeros(n)` / `zeros(n, m)` | Length-n zero vector, or n×m zero matrix |
| `ones(n)` / `ones(n, m)` | Length-n ones vector, or n×m ones matrix |
| `eye(n)` | n×n identity matrix |
| `rand(n)` | n floats uniform [0, 1) |
| `randn(n)` / `randn(m, n)` | n floats (or m×n matrix) from N(0,1) |
| `randi(imax)` / `randi(imax, n)` / `randi([lo,hi], n)` | Random integers |
| `len(v)` / `length(v)` | Number of elements |
| `size(v)` | `[rows, cols]` as a Vector |
| `numel(v)` | Total element count |
| `diag(v)` | Diagonal matrix from vector; or extract diagonal |
| `reshape(M, r, c)` | Reshape to r×c |
| `repmat(M, r, c)` | Tile M r×c times |
| `transpose(M)` | Non-conjugate transpose |
| `horzcat(A, B, ...)` | Horizontal concatenation (also `[A, B]`) |
| `vertcat(A, B, ...)` | Vertical concatenation (also `[A; B]`) |
| `meshgrid(x, y)` | Returns `[X, Y]` matrices for 2D grids |

---

## Statistics

| Function | Description |
|---|---|
| `sum(v)` | Sum all elements |
| `prod(v)` | Product of all elements |
| `cumsum(v)` | Cumulative sum |
| `min(v)`, `max(v)` | Min / max value |
| `argmin(v)`, `argmax(v)` | 1-based index of min / max |
| `mean(v)` | Arithmetic mean |
| `median(v)` | Median (real parts; average of two middles for even length) |
| `std(v)` | Standard deviation (N-1 denominator) |
| `sort(v)` | Sort ascending by real part |
| `trapz(v)` / `trapz(x, v)` | Trapezoidal integration (unit or explicit spacing) |
| `histogram(v)` / `histogram(v, n)` | Histogram; returns 2×n matrix (bin centers, counts) |
| `savehist(v, file)` | Save histogram to PNG or SVG |
| `all(v)` | True if all elements nonzero |
| `any(v)` | True if any element nonzero |

---

## Linear Algebra

| Function | Description |
|---|---|
| `dot(u, v)` | Inner (dot) product |
| `cross(u, v)` | 3-element cross product |
| `outer(u, v)` | Outer product → N×M matrix |
| `kron(A, B)` | Kronecker tensor product |
| `norm(v)` | L2 norm of vector; Frobenius norm of matrix |
| `inv(M)` | Matrix inverse |
| `det(M)` | Determinant |
| `trace(M)` | Trace |
| `rank(M)` | Numerical rank |
| `eig(M)` | Eigenvalues (column vector) |
| `expm(M)` | Matrix exponential $e^M$ (Padé approximant) |
| `linsolve(A, b)` | Solve A·x = b; returns x |
| `roots(p)` | Roots of polynomial with coefficients p |

---

## Special Functions

| Function | Description |
|---|---|
| `laguerre(n, alpha, x)` | Associated Laguerre polynomial $L_n^\alpha(x)$, element-wise |
| `legendre(l, m, x)` | Associated Legendre polynomial $P_l^m(x)$, element-wise |
| `convolve(x, h)` | Linear convolution (output length = len(x)+len(h)-1) |
| `filtfilt(b, a, x)` | Zero-phase forward-backward IIR filter; use `a=[1]` for FIR |
| `factor(n)` | Prime factorization |

---

## Fourier Transforms

| Function | Description |
|---|---|
| `fft(v)` | Discrete Fourier transform (zero-pads to next power of 2) |
| `ifft(V)` | Inverse DFT |
| `fftshift(V)` | Shift zero-frequency to center |
| `fftfreq(n, sr)` | Frequency axis for n-point DFT at sample rate sr |
| `spectrum(v, sr)` | Returns 2×n matrix: row 1 = Hz (DC-centered), row 2 = complex spectrum |

---

## DSP — Filters

| Function | Description |
|---|---|
| `fir_lowpass(taps, cutoff_hz, sr, window)` | FIR lowpass coefficients |
| `fir_highpass(taps, cutoff_hz, sr, window)` | FIR highpass coefficients |
| `fir_bandpass(taps, low_hz, high_hz, sr, window)` | FIR bandpass coefficients |
| `butterworth_lowpass(order, cutoff_hz, sr)` | Butterworth IIR lowpass (b coefficients) |
| `butterworth_highpass(order, cutoff_hz, sr)` | Butterworth IIR highpass (b coefficients) |
| `fir_lowpass_kaiser(cutoff_hz, trans_bw_hz, atten_db, sr)` | Auto-designed Kaiser lowpass |
| `fir_highpass_kaiser(cutoff_hz, trans_bw_hz, atten_db, sr)` | Auto-designed Kaiser highpass |
| `fir_bandpass_kaiser(lo_hz, hi_hz, trans_bw_hz, atten_db, sr)` | Auto-designed Kaiser bandpass |
| `fir_notch(center_hz, bw_hz, sr, taps, window)` | FIR notch filter |
| `firpm(n_taps, bands, desired)` | Parks-McClellan optimal equiripple FIR |
| `firpm(n_taps, bands, desired, weights)` | Parks-McClellan with per-band weights |
| `firpmq(n_taps, bands, desired [, weights [, bits [, n_iter]]])` | Integer-coefficient Parks-McClellan (default bits=16, n_iter=8); returns integer taps. For unit-gain passband use `freqz(h / sum(h), ...)` to normalize. |
| `freqz(h, n_points, sr)` | Complex frequency response → 2×n matrix |
| `upfirdn(x, h, p, q)` | Upsample·filter·downsample via polyphase decomposition |
| `window(name, n)` | Window vector; names: `"hann"` `"hamming"` `"blackman"` `"rectangular"` `"kaiser"` |

---

## Control Systems

| Function | Description |
|---|---|
| `tf(num, den)` | Create transfer function from numerator/denominator coefficient vectors |
| `pole(sys)` | Poles of a transfer function |
| `zero(sys)` | Zeros of a transfer function |
| `ss(A, B, C, D)` | Create state-space system |
| `ctrb(A, B)` | Controllability matrix |
| `obsv(A, C)` | Observability matrix |
| `bode(sys)` | Bode plot in terminal |
| `step(sys)` | Step response plot in terminal |
| `margin(sys)` | Gain and phase margins |
| `lqr(A, B, Q, R)` | LQR optimal gain matrix K |
| `rlocus(sys)` | Root locus plot in terminal |

---

## Fixed-Point Quantization

| Function | Description |
|---|---|
| `qfmt(word_bits, frac_bits)` | Create Q-format spec (default: floor rounding, saturate overflow) |
| `qfmt(w, f, round_mode, overflow_mode)` | Full spec; round: `"floor"` `"ceil"` `"zero"` `"round"` `"round_even"`; overflow: `"saturate"` `"wrap"` |
| `quantize(x, fmt)` | Quantize scalar/vector/matrix to Q-format grid |
| `qadd(a, b, fmt)` | Fixed-point element-wise add, result quantized to fmt |
| `qmul(a, b, fmt)` | Fixed-point element-wise multiply, result quantized to fmt |
| `qconv(x, h, fmt)` | Fixed-point FIR convolution, output quantized to fmt |
| `snr(x_ref, x_q)` | Signal-to-noise ratio in dB between reference and quantized signal |

---

## ML / Activation Functions

| Function | Description |
|---|---|
| `softmax(v)` | Softmax probability distribution (numerically stable) |
| `relu(v)` | Rectified linear unit: max(0, x), element-wise |
| `gelu(v)` | Gaussian error linear unit, element-wise |
| `layernorm(v)` / `layernorm(v, eps)` | Layer normalization: (v − mean) / std |

---

## Structs

| Syntax / Function | Description |
|---|---|
| `s = struct("x", 1, "y", 2)` | Create struct from field-value pairs |
| `s.field` | Access a field |
| `s.field = val` | Set a field (auto-creates struct if s is undefined) |
| `isstruct(x)` | True if x is a struct |
| `fieldnames(s)` | Print all field names |
| `isfield(s, "name")` | True if struct has the named field |
| `rmfield(s, "name")` | Return new struct with named field removed |

---

## Output & I/O

| Function | Description |
|---|---|
| `print(x, ...)` | Print to stdout, space-separated |
| `disp(x)` | Display a value (always appends newline) |
| `fprintf(fmt, args...)` | Formatted print; specifiers: `%d %f %g %e %s %%`; escapes: `\n \t` |
| `save("file.npy", x)` | Save array to NumPy .npy format |
| `save("file.npz", "a", a, "b", b, ...)` | Save multiple named arrays to .npz |
| `save("file.csv", x)` | Save array to CSV |
| `load("file.npy")` | Load .npy → value |
| `load("file.npz")` | Load all arrays from .npz into workspace |
| `load("file.npz", "name")` | Load one named array from .npz |
| `load("file.csv")` | Load CSV → scalar / vector / matrix |
| `whos` | List workspace variables with type and size |
| `whos("file.npz")` | Inspect arrays stored in an NPZ file |

---

## Plotting — Terminal (interactive, blocks until keypress)

| Function | Description |
|---|---|
| `plot(v)` / `plot(v, "title")` | Line plot |
| `plot(v, "title", "color")` | Colors: `r g b c m y k w` |
| `plot(v, "title", "color", "dashed")` | Dashed line |
| `stem(v)` / `stem(v, "title")` | Stem plot |
| `bar(y)` / `bar(x, y)` / `bar(y, "title")` | Bar chart |
| `scatter(x, y)` / `scatter(x, y, "title")` | Scatter plot |
| `plotdb(Hz)` / `plotdb(Hz, "title")` | dB frequency response (Hz from `freqz` or `spectrum`) |
| `imagesc(M)` / `imagesc(M, cmap)` | Matrix heatmap; colormaps: `"viridis"` `"jet"` `"hot"` `"gray"` |

---

## Plotting — File Output (PNG or SVG by extension)

| Function | Description |
|---|---|
| `savefig(v, file)` / `savefig(v, file, "title")` | Line plot → file |
| `savestem(v, file)` / `savestem(v, file, "title")` | Stem plot → file |
| `savebar(y, file)` / `savebar(x, y, file, "title")` | Bar chart → file |
| `savescatter(x, y, file)` / `savescatter(x, y, file, "title")` | Scatter plot → file |
| `savedb(Hz, file)` / `savedb(Hz, file, "title")` | dB response → file |
| `savehist(v, file)` / `savehist(v, n, file, "title")` | Histogram → file |
| `saveimagesc(M, file)` / `saveimagesc(M, file, "title", cmap)` | Heatmap → file |

---

## Figure Controls (apply to the next `plot`/`stem`/… call)

| Function | Description |
|---|---|
| `figure()` | Reset figure state; clears all subplots and series |
| `hold("on")` / `hold("off")` | Overlay series on current subplot |
| `grid("on")` / `grid("off")` | Show / hide grid lines |
| `title("text")` | Set subplot title |
| `xlabel("text")` | Set x-axis label |
| `ylabel("text")` | Set y-axis label |
| `xlim([lo, hi])` | Fix x-axis range |
| `ylim([lo, hi])` | Fix y-axis range |
| `subplot(rows, cols, idx)` | Switch to panel idx (1-based, left-to-right then top-to-bottom) |
| `legend("s1", "s2", ...)` | Label series in order added |

---

## Streaming DSP

| Function | Description |
|---|---|
| `state_init(n)` | Allocate overlap-save history buffer of length n (use `length(h)-1`) |
| `filter_stream(frame, h, state)` | Filter frame through FIR h; returns Tuple `[y, state]` |

## Audio I/O

Raw f32 LE stdin/stdout PCM. Use bridge programs (sox, arecord/aplay) to connect hardware.

| Function | Description |
|---|---|
| `audio_in(sr, frame)` | Create AudioIn descriptor (sample_rate, frame_size) |
| `audio_out(sr, frame)` | Create AudioOut descriptor (sample_rate, frame_size) |
| `audio_read(src)` | Read one frame from stdin; exits cleanly on EOF |
| `audio_write(dst, y)` | Write one frame (real parts) to stdout; flushes after each frame |

---

## Common Patterns

**2D grid:**
```r
x = linspace(-10.0, 10.0, N)
[X, Z] = meshgrid(x, x)
r_mat = sqrt(X .^ 2 + Z .^ 2)
```

**Build a vector in a loop:**
```r
for i = 1:n
  v(i) = some_fn(i)
end
```

**Trapezoidal integral with custom spacing:**
```r
norm = trapz(x, prob)
```

**FIR filter (windowed sinc):**
```r
h = fir_lowpass(63, 1000.0, 44100.0, "hann")
y = convolve(x, h)
```

**Auto-designed Kaiser lowpass:**
```r
h = fir_lowpass_kaiser(1000.0, 200.0, 60.0, 44100.0)
```

**Parks-McClellan equiripple lowpass:**
```r
h = firpm(63, [0, 0.2, 0.3, 1.0], [1, 1, 0, 0])
```

**Frequency response plot:**
```r
H = freqz(h, 512, 44100.0)
plotdb(H, "Lowpass response")
```

**Fixed-point quantization:**
```r
fmt = qfmt(16, 15, "round_even", "saturate")
xq  = quantize(x, fmt)
hq  = quantize(h, fmt)
yq  = qconv(xq, hq, fmt)
db  = snr(y_ref, yq)
```

**Transfer function and step response:**
```r
sys = tf([1], [1, 2, 1])
step(sys)
```

**State-space LQR design:**
```r
A = [0, 1; -1, -0.5]
B = [0; 1]
Q = eye(2)
R = [1]
K = lqr(A, B, Q, R)
```

**Multi-panel figure:**
```r
figure()
subplot(2, 1, 1)
  title("Signal")
  plot(x, "Signal")
subplot(2, 1, 2)
  title("Spectrum")
  plotdb(freqz(h, 512, sr), "Response")
```

**Save and reload workspace:**
```r
save("data.npz", "x", x, "y", y)
load("data.npz")
```

**Real-time FIR streaming (stdin → stdout):**
```r
sr    = 44100.0
FRAME = 256
h     = firpm(64, [0.0, 0.2, 0.3, 1.0], [1.0, 1.0, 0.0, 0.0])
state = state_init(length(h) - 1)
src   = audio_in(sr, FRAME)
dst   = audio_out(sr, FRAME)
while true
  frame = audio_read(src)
  [y, state] = filter_stream(frame, h, state)
  audio_write(dst, y)
end
```
Run as: `sox -d ... | rustlab run filter.r | sox ... -d` (see `examples/stream/`)
