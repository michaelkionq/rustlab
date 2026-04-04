# Function Reference

Complete reference for all built-in functions and constants available in the rustlab scripting language.

---

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `i`  | `0 + 1i` | Imaginary unit. Use in expressions: `z = 3 + i*4` |
| `j`  | `0 + 1i` | Alias for `i`. Both are always available: `z = 3 + j*4` |
| `pi` | 3.14159… | π |
| `e`  | 2.71828… | Euler's number |

---

## Math

### `abs(x)`
Absolute value or magnitude.
- Scalar: `abs(-3.0)` → `3.0`
- Complex: `abs(3 + j*4)` → `5.0` (L2 norm)
- Vector: element-wise magnitude, returns real vector

### `angle(x)`
Phase angle in radians (`atan2(im, re)`).
- Complex: `angle(1 + j*1)` → `0.7854` (π/4)
- Vector: element-wise

### `real(x)`
Real part of a scalar, complex number, or vector.

### `imag(x)`
Imaginary part of a scalar, complex number, or vector.

### `cos(x)`
Cosine, element-wise. Accepts real or complex values.

### `sin(x)`
Sine, element-wise. Accepts real or complex values.

### `sqrt(x)`
Square root, element-wise. Accepts real or complex values.

### `exp(x)`
Natural exponential `eˣ`, element-wise. Accepts real or complex values.
```
exp(j * pi)   # → -1 + 0i  (Euler's identity)
```

### `log(x)`
Natural logarithm (base e), element-wise. Accepts real or complex values.

### `log10(x)`
Base-10 logarithm, element-wise. Accepts real or complex values.
```
log10(1000.0)   # → 3.0
```
Commonly used for dB calculations:
```
db = 20.0 * log10(abs(X) + 1e-12)
```

### `log2(x)`
Base-2 logarithm, element-wise. Accepts real or complex values.
```
log2(8.0)    # → 3.0
log2(1024)   # → 10.0
```
Useful for computing bit depths and octave-spaced frequency grids.

---

## Statistics

### `min(v)`
Smallest value in a vector (real part used for complex vectors).
```
min([3.0, 1.0, 4.0, 1.5])   # → 1.0
```

### `max(v)`
Largest value in a vector (real part used for complex vectors).
```
max([3.0, 1.0, 4.0, 1.5])   # → 4.0
```

### `mean(v)`
Arithmetic mean. Returns a complex scalar for complex vectors.
```
mean([1.0, 2.0, 3.0])   # → 2.0
mean(randn(1000))        # → ≈ 0.0
```

### `std(v)`
Sample standard deviation (Bessel-corrected, N−1 denominator).
```
std(randn(10000))   # → ≈ 1.0
```

---

## Array Construction

### `zeros(n)`
Returns a length-n complex zero vector.
```
zeros(4)   # → [0+0j, 0+0j, 0+0j, 0+0j]
```

### `ones(n)`
Returns a length-n complex one vector.
```
ones(3)   # → [1+0j, 1+0j, 1+0j]
```

### `linspace(start, stop, n)`
`n` evenly spaced real values from `start` to `stop` (inclusive).
```
linspace(0.0, 1.0, 5)   # → [0.0, 0.25, 0.5, 0.75, 1.0]
```

### `len(v)` / `length(v)`
Number of elements in a vector, rows in a matrix, or characters in a string.

### `numel(x)`
Total number of elements: `rows × cols` for matrices, `1` for scalars.

### `size(x)`
Returns a 2-element vector `[rows, cols]`. Vectors return `[1, n]`.

---

## Random Numbers

### `rand(n)`
`n` samples drawn uniformly from `[0, 1)`.
```
noise = rand(512)
```

### `randn(n)`
`n` samples from the standard normal distribution (μ=0, σ=1).
```
noise = randn(1024) * 0.1   # low-level Gaussian noise
```

### `randi(imax)` / `randi(imax, n)` / `randi([lo, hi], n)`
Random integers.
```
randi(6)          # single integer in [1, 6]  — one die roll
randi(6, 100)     # 100 integers in [1, 6]
randi([0, 1], 8)  # 8 random bits
randi([-5, 5], 50)  # 50 integers in [-5, 5]
```

---

## FFT

### `fft(v)`
Forward FFT using the Cooley-Tukey radix-2 algorithm. Input is zero-padded to the next power of two if necessary.
```
X = fft(x)          # len(X) is next power of two >= len(x)
```

### `ifft(X)`
Inverse FFT. Input length must be a power of two (as returned by `fft`).
```
x_rec = real(ifft(X))   # round-trip reconstruction
```

### `fftshift(X)`
Rearranges FFT output so the DC component (bin 0) moves to the center. Negative frequencies appear on the left.
```
Xs = fftshift(X)   # [A B] → [B A]
```

### `fftfreq(n, sample_rate)`
Frequency bin values in Hz for an n-point FFT.
- Bins `0..n/2` → positive frequencies `0` to `sr/2 − sr/n`
- Bins `n/2..n` → negative frequencies `−sr/2` to `−sr/n`
```
freqs = fftfreq(256, 8000.0)   # 256-point FFT at 8 kHz
```

### `spectrum(X, sample_rate)`
The recommended way to display FFT results with a correct Hz axis.

Applies `fftshift` to the spectrum and pairs it with the DC-centered frequency axis, returning a **2×n matrix** that plugs directly into `plotdb` and `savedb`:
- Row 1: frequency axis in Hz (DC = 0, negative on left, positive on right)
- Row 2: complex spectrum (DC centered)

```
X = fft(x)
H = spectrum(X, sr)
plotdb(H, "Magnitude Spectrum")
savedb(H, "spectrum.svg", "Magnitude Spectrum")
```

This is the standard workflow for viewing FFT output with a proper frequency axis. Internally it is equivalent to:
```
# What spectrum() does for you:
Xs    = fftshift(X)
freqs = fftshift(fftfreq(len(X), sr))
# (pairs them into a matrix for plotdb/savedb)
```

---

## DSP — FIR Filters (manual tap count)

All FIR design functions return a complex coefficient vector.

### `fir_lowpass(taps, cutoff_hz, sample_rate, window)`
Windowed-sinc lowpass filter.
```
h = fir_lowpass(64, 1000.0, 44100.0, "hann")
```

### `fir_highpass(taps, cutoff_hz, sample_rate, window)`
Windowed-sinc highpass filter (spectral inversion of lowpass).
```
h = fir_highpass(64, 3000.0, 44100.0, "hamming")
```

### `fir_bandpass(taps, low_hz, high_hz, sample_rate, window)`
Windowed-sinc bandpass filter (difference of two lowpass filters).
```
h = fir_bandpass(128, 500.0, 2000.0, 44100.0, "blackman")
```

**Window names:** `"rectangular"`, `"hann"`, `"hamming"`, `"blackman"`, `"kaiser"`

Approximate stopband attenuation by window:

| Window | Stopband attenuation |
|--------|----------------------|
| Rectangular | ~21 dB |
| Hann | ~44 dB |
| Hamming | ~41 dB |
| Blackman | ~74 dB |
| Kaiser (auto β) | user-specified |

### `convolve(x, h)`
Linear convolution. Output length = `len(x) + len(h) − 1`.
```
y = convolve(signal, h)
```

### `window(name, n)`
Generate a standalone window function vector of length `n`.
```
w = window("hann", 64)
```

---

## DSP — Kaiser FIR (automatic tap count)

Kaiser filters automatically compute the window shape parameter β and the required tap count from the desired stopband attenuation and transition bandwidth — no manual tap count needed.

### `fir_lowpass_kaiser(cutoff_hz, trans_bw_hz, stopband_attn_db, sample_rate)`
```
h = fir_lowpass_kaiser(1000.0, 200.0, 60.0, 8000.0)
```
For 60 dB attenuation and 200 Hz transition width at 8 kHz: β ≈ 5.65, ~185 taps.

### `fir_highpass_kaiser(cutoff_hz, trans_bw_hz, stopband_attn_db, sample_rate)`
```
h = fir_highpass_kaiser(3000.0, 200.0, 60.0, 8000.0)
```

### `fir_bandpass_kaiser(low_hz, high_hz, trans_bw_hz, stopband_attn_db, sample_rate)`
```
h = fir_bandpass_kaiser(1000.0, 2500.0, 200.0, 60.0, 8000.0)
```

### `fir_notch(center_hz, bandwidth_hz, sample_rate, num_taps, window)`
Notch filter via spectral inversion of a bandpass. Rejects a narrow band around `center_hz`.
```
h = fir_notch(1000.0, 200.0, 8000.0, 65, "hann")
```

**Kaiser design guidelines:**

| Attenuation | β | Typical use |
|-------------|---|-------------|
| 40 dB | 3.40 | General audio |
| 60 dB | 5.65 | Most signal processing |
| 80 dB | 7.86 | High-fidelity |
| 100 dB | 10.06 | Demanding applications |

### `freqz(h, n_points, sample_rate)`
Complex frequency response of a filter at `n_points` frequencies from 0 to Nyquist.
Returns a **2×n matrix**:
- Row 1: frequency axis in Hz
- Row 2: complex H(f)

```
Hz = freqz(h, 512, 44100.0)
plotdb(Hz, "Frequency Response")
savedb(Hz, "response.svg", "Frequency Response")
```

---

## Fixed-Point Quantization

Fixed-point simulation for FPGA/ASIC bitwidth studies. Operations compute at full float precision internally, then quantize the output to the specified Q format — matching real hardware behaviour exactly.

### `qfmt(word_bits, frac_bits [, round_mode [, overflow_mode]])`

Creates a Q-format specification. All quantization and arithmetic functions accept a `qfmt` spec as their format argument.

| Parameter | Values | Default |
|-----------|--------|---------|
| `word_bits` | 2–32 | required |
| `frac_bits` | 0 to word_bits−1 | required |
| `round_mode` | `"floor"` `"ceil"` `"zero"` `"round"` `"round_even"` | `"floor"` |
| `overflow_mode` | `"saturate"` `"wrap"` | `"saturate"` |

`"floor"` (truncate toward −∞) is the hardware default — it is free in RTL (just drop the LSBs). `"round_even"` (convergent/banker's) minimises bias in long filter chains.

```
fmt = qfmt(16, 15)                            # Q0.15, floor, saturate
fmt = qfmt(16, 15, "round_even", "saturate")  # same with convergent rounding
fmt = qfmt(8,  7,  "floor",      "wrap")      # 8-bit, wrap on overflow
```

In the REPL, a `qfmt` value displays its full spec:
```
QFmt<16-bit Q0.15, round=round_even, overflow=saturate>
```

### `quantize(x, fmt)`

Snap every element to the nearest representable value in `fmt`. Works on scalars, complex, vectors, and matrices. Real and imaginary parts are quantized independently. Returns the same type as the input — compatible with all existing math, FFT, plot, and save functions.

```
fmt = qfmt(16, 15, "round_even", "saturate")
xq  = quantize(x, fmt)
hq  = quantize(h, fmt)
noise = x - real(xq)    # quantization noise vector
```

### `qadd(a, b, fmt)`

Element-wise add, result quantized to `fmt`. Both inputs must be real scalars or real vectors of equal length.

```
y = qadd(xq, dc_offset, fmt)
```

### `qmul(a, b, fmt)`

Element-wise multiply, result quantized to `fmt`. The full Q-product is computed internally (no intermediate truncation).

```
scaled = qmul(xq, gain, fmt)
```

### `qconv(x, h, fmt)`

Fixed-point FIR convolution. Accumulates products at full precision (equivalent to a wide hardware accumulator), then quantizes each output sample to `fmt`. Output length = `len(x) + len(h) − 1`.

```
y = qconv(xq, hq, fmt_out)
```

### `snr(x_ref, x_quantized)`

Signal-to-noise ratio in dB between a float reference and a quantized signal. Both must be real vectors of equal length.

```
SNR = 10 · log₁₀(signal_power / noise_power)
```

Returns `+Inf` when signals are identical, `-Inf` when the reference is all-zeros.

```
db = snr(y_ref, y_quantized)
```

### Bitwidth study example

```
h = firpm(63, [0.0, 0.20, 0.30, 1.0], [1.0, 1.0, 0.0, 0.0])
# Scale randn to stay inside the Q1.14 range (±2); unscaled N(0,1) saturates
# ~5 % of samples, which swamps the coefficient-quantization noise floor.
x = randn(1024) * 0.3
y_ref = real(convolve(x, real(h)))

fmt_data = qfmt(16, 14, "round_even", "saturate")
xq = quantize(x, fmt_data)

fmt8  = qfmt(8,  7,  "round_even", "saturate")
fmt16 = qfmt(16, 15, "round_even", "saturate")

y8  = qconv(xq, real(quantize(h, fmt8)),  fmt_data)
y16 = qconv(xq, real(quantize(h, fmt16)), fmt_data)

print(snr(y_ref, y8))   # ~30 dB  (8-bit coeff)
print(snr(y_ref, y16))  # ~74 dB  (16-bit coeff)
```

---

## DSP — Parks-McClellan optimal FIR

`firpm` designs optimal equiripple FIR filters using the Remez exchange algorithm (). It minimises the maximum weighted error across all specified bands simultaneously, producing the minimum-ripple design for a given tap count.

### `firpm(n_taps, bands, desired)`
### `firpm(n_taps, bands, desired, weights)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `n_taps` | integer | Number of filter taps (forced odd — Type I symmetric) |
| `bands` | vector | Frequency band edges, normalized to [0, 1] where 1 = Nyquist |
| `desired` | vector | Target amplitude at each band edge (piecewise-linear, same length as `bands`) |
| `weights` | vector | Optional — one weight per band pair (default: all 1.0) |

Band edges come in pairs: `[f_low1, f_high1, f_low2, f_high2, ...]`. The gaps between pairs are transition bands (don't-care regions).

**Low-pass (0 to 0.20 Nyquist pass, 0.30 Nyquist+ stop):**
```
h = firpm(63, [0.0, 0.20, 0.30, 1.0], [1.0, 1.0, 0.0, 0.0])
```

**Band-pass (pass 0.30 to 0.50 Nyquist):**
```
h = firpm(79, [0.0, 0.25, 0.30, 0.50, 0.55, 1.0],
              [0.0, 0.0,  1.0,  1.0,  0.0,  0.0])
```

**Weighted — enforce 10x tighter stopband than passband:**
```
h = firpm(51, [0.0, 0.25, 0.35, 1.0],
              [1.0, 1.0,  0.0,  0.0],
              [1.0, 10.0])
```

**Compared to Kaiser:**
- Kaiser automatically determines tap count from attenuation and transition width.
- `firpm` gives the optimal (fewest-ripple) filter for a fixed tap count, often requiring fewer taps than Kaiser for the same spec.

---

## DSP — IIR Filters

### `butterworth_lowpass(order, cutoff_hz, sample_rate)`
Butterworth IIR lowpass filter. Higher order gives a steeper rolloff.
```
h = butterworth_lowpass(4, 1000.0, 44100.0)
y = convolve(x, h)
```

### `butterworth_highpass(order, cutoff_hz, sample_rate)`
Butterworth IIR highpass filter.
```
h = butterworth_highpass(4, 3000.0, 44100.0)
```

---

## Linear Algebra

### `eig(M)`
Eigenvalues of a square matrix `M`. Returns a complex vector of length `n`.
Uses Hessenberg reduction followed by single-shift QR iteration with Wilkinson shifts.
```
M = [2,1;1,2]
v = eig(M)       # → complex vector with eigenvalues ~[3+0i, 1+0i]
```
- Input must be a square matrix (or scalar, which returns a 1-element vector).
- Eigenvalues are returned in convergence order, not sorted.
- The sum of eigenvalues equals `trace(M)`; the product equals `det(M)`.

### `factor(n)`
Prime factorization of a positive integer `n`. Returns a real vector of prime factors
in ascending order, with repetition.
```
factor(12)    # → [2, 2, 3]
factor(17)    # → [17]
factor(1)     # → [] (empty vector)
factor(360)   # → [2, 2, 2, 3, 3, 5]
```
- `n` must be a positive integer scalar.
- `factor(0)` and `factor(-3)` produce a type error.

---

## Plotting

All plot functions accumulate series into a shared **figure state** and render immediately. Use `figure()`, `hold()`, `subplot()` etc. to control layout before calling plot functions.

### Figure State

#### `figure()`
Reset the figure to a blank state (clears all subplots and series).
```
figure()
```

#### `hold("on")` / `hold("off")`
When hold is on, new `plot()`/`stem()` calls add series to the current subplot instead of replacing them. Accepts `"on"`, `"off"`, `1`, or `0`.
```
hold("on")
plot(signal1, "label", "first")
plot(signal2, "label", "second")
hold("off")
```

#### `subplot(rows, cols, idx)`
Switch to subplot panel. `rows` and `cols` define the grid; `idx` is 1-based (row-major order).
```
subplot(2, 1, 1)
plot(x)
subplot(2, 1, 2)
stem(h)
```

#### `grid("on")` / `grid("off")`
Enable or disable grid lines on the current subplot.
```
grid("on")
```

#### `xlabel("text")`
Set the x-axis label on the current subplot.
```
xlabel("Time (s)")
```

#### `ylabel("text")`
Set the y-axis label on the current subplot.
```
ylabel("Amplitude")
```

#### `title("text")`
Set the title on the current subplot.
```
title("Frequency Response")
```

#### `xlim([lo, hi])`
Set x-axis bounds on the current subplot.
```
xlim([0.0, 1000.0])
```

#### `ylim([lo, hi])`
Set y-axis bounds on the current subplot.
```
ylim([-1.0, 1.0])
```

#### `legend("s1", "s2", ...)`
Retroactively set labels on series in the current subplot (in order).
```
hold("on")
plot(a)
plot(b)
legend("signal a", "signal b")
```

---

## Visualization — Interactive (terminal)

These functions open a full-screen terminal chart and wait for a keypress before returning.

### `plot(v)`
Line chart of a real or complex vector (sample index on x). For complex vectors, shows magnitude (blue) and real part (green) overlaid.
```
plot(signal, "440 Hz Sinusoid")
```

### `plot(x, v)`
Line chart with explicit x-axis vector.
```
t = linspace(0.0, 1.0, 1000)
plot(t, signal, "label", "sine wave")
```

### `plot(v, "color", c, "label", lbl, "style", s)`
Plot with options. Options are trailing key-value string pairs:
- `"color"` — color name: `"red"`, `"green"`, `"blue"`, `"cyan"`, `"magenta"`, `"yellow"`, `"black"`, `"white"`, or single-letter shortcuts (`"r"`, `"g"`, `"b"`, ...)
- `"label"` — legend label string
- `"style"` — `"solid"` (default) or `"dashed"`
```
plot(signal, "color", "red", "label", "filtered")
plot(t, noise, "color", "g", "style", "dashed", "label", "noise")
```

### `plot(M)` / `plot(x, M)`
Plot a matrix: one line series per column.
```
plot(M)           # sample index x, each column a series
plot(t, M)        # explicit x axis
```

### `stem(v)` / `stem(x, v)`
Stem (lollipop) chart — one vertical bar per sample. Supports the same color/label/style options as `plot()`.
```
stem(real(h), "Impulse Response")
stem(n, h, "color", "red", "label", "h[n]")
```

### `plotdb(Hz [, title])`
Frequency response in dB. `Hz` is the 2×n matrix returned by `freqz()` or `spectrum()`.
- x-axis: frequency in Hz
- y-axis: 20·log₁₀|H(f)|
```
plotdb(freqz(h, 512, sr), "Lowpass Response")
plotdb(spectrum(fft(x), sr), "Signal Spectrum")
```

### `histogram(v [, n_bins])`
Bar chart histogram of `v`. Default bin count is 10. Displays interactively and returns a **2×n matrix**:
- Row 1: bin centers
- Row 2: counts
```
histogram(randn(2000), 30)
H = histogram(data, 20)   # capture bin data
```

### `imagesc(M)` / `imagesc(M, colormap)`
Display a matrix as a false-color heatmap in the terminal. Each cell is colored according to its magnitude using the specified colormap. Supported colormaps: `"viridis"` (default), `"jet"`, `"hot"`, `"gray"`.
```
imagesc(spectrogram_matrix)
imagesc(M, "jet")
```

---

## Visualization — File Output (PNG / SVG)

File format is detected from the extension (`.svg` or `.png`).

### `savefig(v, filename [, title])`
Save a line chart to file. Renders the current figure state — call `plot()` first or use figure state functions to set up multi-series plots.
```
savefig(real(signal), "signal.svg", "440 Hz Sinusoid")
savefig(mag, "magnitude.png")
```

### `savestem(v, filename [, title])`
Save a stem chart to file.
```
savestem(real(h), "impulse.svg", "Filter Impulse Response")
```

### `savedb(Hz, filename [, title])`
Save a dB frequency response chart to file. `Hz` is the 2×n matrix from `freqz()` or `spectrum()`.
```
savedb(freqz(h, 512, sr), "response.svg", "Lowpass Response")
savedb(spectrum(fft(x), sr), "spectrum.svg", "Signal Spectrum")
```

### `savehist(v, filename [, title])` / `savehist(v, n_bins, filename [, title])`
Save a histogram to file.
```
savehist(randn(2000), "noise_hist.svg", "Noise Distribution")
savehist(data, 30, "data_hist.png", "Data Histogram")
```

### `saveimagesc(M, filename)` / `saveimagesc(M, filename, title)` / `saveimagesc(M, filename, title, colormap)`
Save a matrix heatmap to file. Supported colormaps: `"viridis"` (default), `"jet"`, `"hot"`, `"gray"`.
```
saveimagesc(spectrogram, "spec.png")
saveimagesc(M, "heatmap.svg", "Correlation Matrix", "jet")
```

---

## Import / Export

### `save(filename, x)`
Save a single variable to a file. Format is determined by the file extension.

| Extension | Format | Notes |
|-----------|--------|-------|
| `.npy` | NumPy binary | Real arrays stored as `float64`, complex as `complex128`. Compatible with `numpy.load()` in Python. |
| `.csv` | CSV text | Complex values written as `a+bi`. Real arrays produce plain numbers. |

```
save("signal.npy", x)
save("coeffs.csv", h)
```

### `save(filename, "name1", x1, "name2", x2, ...)`
Save multiple named variables into a single `.npz` archive (a zip file containing one `.npy` entry per variable). The `.npz` extension is required.

```
save("session.npz", "signal", x, "filter", h, "freqs", f)
```

The resulting file is directly readable by `numpy.load("session.npz")` in Python.

### `load(filename)`
Load a single array from a `.npy` or `.csv` file. Returns a scalar, vector, or matrix depending on the stored shape.

```
x = load("signal.npy")
h = load("coeffs.csv")
```

### `load(filename, varname)`
Load one named array from a `.npz` archive.

```
x = load("session.npz", "signal")
h = load("session.npz", "filter")
```

### `whos(filename)`
List the contents of a `.npz` archive — name, type (`real` or `complex`), and size of each stored array. Returns `None`; output is printed.

```
whos("session.npz")
```

Example output:
```
  Name                 Type       Size
  ────────────────────────────────────────────
  signal               complex    1024
  filter               real       65
  freqs                real       512
```

---

## Controls Toolbox

Classical control systems — transfer functions, state-space, frequency analysis, and optimal control.

### `tf(arg)` / `tf(num, den)`

Create a transfer function.

```
s = tf("s")              % Laplace variable: num=[1,0], den=[1]
G = tf([10], [1, 2, 10]) % 10 / (s² + 2s + 10)
```

Build TFs from `s` using arithmetic — the preferred idiom:

```
s   = tf("s")
G   = 10 / (s^2 + 2*s + 10)
C   = 5 * (s + 2) / s       % PI controller
T   = G * C / (1 + G * C)   % closed-loop
```

Supported arithmetic: `+`, `-`, `*`, `/`, `^` (integer exponent), and scalar operands.

### `pole(G)`

Roots of the denominator (open-loop poles).

```
G = tf([10], [1, 2, 10])
p = pole(G)   % ≈ [-1+3j, -1-3j]
```

### `zero(G)`

Roots of the numerator (transmission zeros).

```
G = tf([1, 1], [1, 2, 10])
z = zero(G)   % ≈ -1
```

### `ss(G)`

Convert a transfer function to state-space (observable canonical form).

```
sys = ss(G)
A = sys.A   B = sys.B   C = sys.C   D = sys.D
```

Each field is a `CMatrix`. Eigenvalues of `A` match `pole(G)`.

### `ctrb(A, B)`

Controllability matrix `[B, AB, A²B, …]` — size n × (n·m).

Full column rank ↔ system is controllable.

```
sys = ss(G)
Wc  = ctrb(sys.A, sys.B)
rank(Wc)   % should equal n for controllable system
```

### `obsv(A, C)`

Observability matrix `[C; CA; CA²; …]` — size (n·p) × n.

Full row rank ↔ system is observable.

```
Wo = obsv(sys.A, sys.C)
rank(Wo)
```

### `bode(G)` / `bode(G, w)` / `[mag, phase, w] = bode(G)`

Bode magnitude and phase plot (log10(ω) x-axis). Always plots; returns data as a tuple.

- `mag` — magnitude in dB
- `phase` — phase in degrees (unwrapped)
- `w` — frequency vector in rad/s

```
G = tf([10], [1, 2, 10])
bode(G)                      % interactive plot
[m, p, w] = bode(G)          % capture data
[m, p, w] = bode(G, w_vec)  % user-supplied frequencies
```

### `step(G)` / `step(G, t_end)` / `[y, t] = step(G)`

Unit step response. Always plots; returns data as a tuple.

```
G = tf([10], [1, 2, 10])
step(G)
[y, t] = step(G)       % capture
[y, t] = step(G, 5)    % specify final time (seconds)
```

Auto `t_end = 10 / min(|Re(poles)|)` capped at 100 s.

### `margin(G)` / `[Gm, Pm, Wcg, Wcp] = margin(G)`

Stability margins from the Bode plot.

- `Gm` — gain margin (linear ratio; `Inf` if no phase crossover)
- `Pm` — phase margin (degrees; `Inf` if no gain crossover)
- `Wcg` — phase crossover frequency, rad/s
- `Wcp` — gain crossover frequency, rad/s

```
G = tf([1], [1, 0.5, 1, 0])
[Gm, Pm, Wcg, Wcp] = margin(G)
fprintf("GM=%.1f dB  PM=%.1f deg\n", 20*log10(Gm), Pm)
```

### `[K, S, e] = lqr(sys, Q, R)`

Linear-Quadratic Regulator — solves the continuous-time algebraic Riccati equation (CARE).

- `sys` — StateSpace value from `ss()`
- `Q` — n×n state weighting matrix (positive semi-definite)
- `R` — m×m input weighting matrix (positive definite)
- `K` — m×n optimal gain matrix: u = −K·x
- `S` — n×n Riccati solution
- `e` — closed-loop eigenvalues of (A − B·K)

```
sys = ss(tf([1], [1, 0, 0]))   % double integrator
[K, S, e] = lqr(sys, eye(2), 1)
% all Re(e) < 0 → closed-loop stable
```

Algorithm: Hamiltonian matrix eigendecomposition.

### `rlocus(G)`

Root locus — plot closed-loop pole trajectories as loop gain K sweeps 0 → ∞.

Each coloured path shows where one open-loop pole migrates as K increases. Trajectories start at the open-loop poles (K = 0) and end at the finite zeros or at infinity (K → ∞).

```
s = tf("s")
G = 1 / (s * (s + 1))
rlocus(G)
```

The plot x-axis is the real part of the poles, y-axis is the imaginary part.

---

## Language

### `print(a [, b, ...])`
Print one or more values to stdout, space-separated.
```
print(x)
print("mean:", mean(v), "std:", std(v))
```

### Range operator: `start:stop` / `start:step:stop`
```
1:5          # [1, 2, 3, 4, 5]
0:0.5:2      # [0.0, 0.5, 1.0, 1.5, 2.0]
10:-1:1      # [10, 9, 8, ..., 1]
```

### Indexing (1-based): `v(i)` / `v(start:stop)`
```
v(1)       # first element
v(end)     # last element
v(2:4)     # elements 2, 3, 4
```

### Element-wise operators: `.* ./ .^`
```
a .* b     # element-wise multiply
a ./ b     # element-wise divide
a .^ 2     # element-wise square
```

### Concatenation: `[a, b, c]`
```
c = [1:4, 5:8]   # [1, 2, 3, 4, 5, 6, 7, 8]
```

### Conjugate transpose: `v'`
```
col = row'
```

### Comments: `#`
```
# This is a comment
x = 1.0   # inline comment
```

### Suppress output: `;`
```
h = fir_lowpass(64, 1000.0, 44100.0, "hann");   # no output printed
```

---

## REPL Commands

These are interactive commands available in the `rustlab` REPL only (not in script files).

| Command | Description |
|---------|-------------|
| `whos` | List all variables with type, size, and value preview |
| `clear` | Remove all user-defined variables (keeps `j`, `pi`, `e`) |
| `run <file>` | Execute a `.r` script; its variables persist in the session |
| `ls [path]` | List directory contents |
| `cd [path]` | Change working directory |
| `pwd` | Print current working directory |
| `help` or `?` | Show help. `? <name>` for detail on a specific function |
