# rustlab

A matrix algebra and DSP toolkit written in Rust — scriptable from the command line or embedded in your own applications.

---

## Install (macOS and Linux)

### Option 1 — system-wide to `/opt/rustlab/bin` (recommended)

```sh
make install
```

Builds a release binary and copies it to `/opt/rustlab/bin/rustlab` (requires `sudo`).
On macOS, `codesign` is run automatically. On Linux, it is skipped.

Add to your shell's `PATH` once:

```sh
# bash (~/.bash_profile / ~/.bashrc) or zsh (~/.zshrc)
export PATH="/opt/rustlab/bin:$PATH"
```

Then reload your shell or run `source ~/.zshrc`.

### Option 2 — into Cargo's bin directory

```sh
cargo install --path crates/rustlab-cli
```

The binary lands in `~/.cargo/bin/rustlab`, which is already on `PATH` if you installed Rust via `rustup`.

---

> **No system libraries required on any platform.** All plotting dependencies are
> pure Rust — no `libfreetype`, `libfontconfig`, or other C libraries needed.

After either option, verify with:

```sh
rustlab --version
```

---

## Quick Start

### Run a script

```sh
rustlab run examples/lowpass.r
```

### Interactive REPL

Run `rustlab` with no arguments to enter the interactive REPL. Readline history and editing are supported.

```sh
rustlab
```

```
rustlab 0.1.0 — type 'exit' or press Ctrl+D to quit
Tip: end a line with ; to suppress output

>> a = 1:5
a = [1.000000, 2.000000, 3.000000, 4.000000, 5.000000]
>> b = a .* a;
>> print(b)
[1.000000, 4.000000, 9.000000, 16.000000, 25.000000]
>> exit
bye
```

### Design an FIR filter

```sh
rustlab filter fir --type low --cutoff 1000 --sr 44100 --taps 64 --window hann
```

### Generate and plot a window function

```sh
rustlab window --type kaiser --length 64 --beta 8.6 --plot
```

---

## Scripting Language Reference

rustlab scripts use the `.r` extension. The interpreter is line-oriented: each statement is one line.

### Variables and Assignment

```
x = 3.14
name = "hello"
```

Variable names are lowercase identifiers. Values are dynamically typed (scalar, vector, matrix, or string).

### Suppress Output

End any statement with `;` to suppress its output:

```
h = fir_lowpass(64, 1000.0, 44100.0, "hann");   # silent — no output
plot(h)                                            # no semicolon — runs immediately
```

This is especially useful in scripts when you don't want intermediate results printed.

### Complex Numbers

`j` is predefined as the imaginary unit `Complex(0.0, 1.0)`. Compose complex literals with ordinary arithmetic:

```
a = 1.0 + j*2.0        # 1 + 2j
b = 3.0 * j            # 0 + 3j
c = a * b              # (-6) + 3j
```

Real and imaginary parts are `f64`. Magnitude is `abs(z)`, phase is `angle(z)`.

### Vectors and Matrices

**Literal syntax** — comma-separated elements in square brackets:

```
v = [1.0, 2.0, 3.0]
M = [1, 2; 3, 4]          # 2×2 matrix — semicolon separates rows
```

**Range / colon operator** — `start:stop` and `start:step:stop`:

```
v = 1:5                    # [1, 2, 3, 4, 5]
w = 0:0.5:2                # [0, 0.5, 1.0, 1.5, 2.0]  (start:step:stop)
r = 10:-1:1                # [10, 9, 8, ..., 1]
t = 0:1/44100:1            # time axis at 44.1 kHz
```

**1-based indexing**:

```
v = 1:10
x = v(3)                   # 3  (1-based)
s = v(2:5)                 # [2, 3, 4, 5]
last = v(end)              # 10
tail = v(end-2:end)        # [8, 9, 10]
```

**Concatenation** — place vectors inside `[...]` to join them:

```
a = 1:3                    # [1, 2, 3]
b = 4:6                    # [4, 5, 6]
c = [a, b]                 # [1, 2, 3, 4, 5, 6]
```

**Transpose** with `'`:

```
v = [1, 2, 3]
t = v'                     # conjugate transpose (for real data, same values)
```

### Arithmetic Operators

| Operator | Description |
|----------|-------------|
| `+`      | Addition (scalar, vector, matrix) |
| `-`      | Subtraction |
| `*`      | Multiplication / scalar broadcast |
| `/`      | Division / scalar broadcast |
| `^`      | Power |
| `.*`     | Element-wise multiply |
| `./`     | Element-wise divide |
| `.^`     | Element-wise power |
| `'`      | Conjugate transpose |

All operators broadcast scalars onto vectors and matrices automatically.
`v .^ 2` squares every element; `2 .^ v` raises 2 to each element of v.

### Builtin Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `print` | `print(x)` | Print a value to stdout |
| `abs` | `abs(x)` | Absolute value / magnitude of scalar or vector |
| `angle` | `angle(z)` | Phase angle in radians |
| `real` | `real(v)` | Real part of a complex scalar or vector |
| `imag` | `imag(v)` | Imaginary part of a complex scalar or vector |
| `cos` | `cos(x)` | Cosine (element-wise) |
| `sin` | `sin(x)` | Sine (element-wise) |
| `exp` | `exp(x)` | Natural exponential (element-wise) |
| `log` | `log(x)` | Natural logarithm (element-wise) |
| `log2` | `log2(x)` | Base-2 logarithm (element-wise) |
| `log10` | `log10(x)` | Base-10 logarithm (element-wise) |
| `sqrt` | `sqrt(x)` | Square root (element-wise) |
| `linspace` | `linspace(start, stop, n)` | `n` evenly-spaced values from `start` to `stop` |
| `zeros` | `zeros(n)` | Vector of `n` zeros |
| `ones` | `ones(n)` | Vector of `n` ones |
| `len` | `len(v)` | Length of a vector |
| `length` | `length(v)` | Alias for `len` — number of elements in a vector |
| `numel` | `numel(x)` | Total number of elements (rows × cols for matrices) |
| `size` | `size(x)` | Returns `[rows, cols]` as a vector |
| `plot` | `plot(v)` or `plot(v, "color", "blue", "label", "name")` | Line plot — renders to terminal or buffers into current figure |
| `stem` | `stem(v)` or `stem(v, "color", "red")` | Stem plot |
| `figure` | `figure()` | Reset figure state to blank |
| `hold` | `hold("on")` / `hold("off")` | Enable/disable multi-series overlay |
| `title` | `title("text")` | Set subplot title |
| `xlabel` | `xlabel("text")` | Set x-axis label |
| `ylabel` | `ylabel("text")` | Set y-axis label |
| `xlim` | `xlim([lo, hi])` | Set x-axis limits |
| `ylim` | `ylim([lo, hi])` | Set y-axis limits |
| `grid` | `grid("on")` | Toggle grid lines |
| `legend` | `legend()` or `legend("l1","l2")` | Show series legend (with optional label overrides) |
| `subplot` | `subplot(rows, cols, idx)` | Select subplot panel (1-based) |
| `imagesc` | `imagesc(M)` or `imagesc(M, "jet")` | Render matrix as heatmap in terminal |
| `savefig` | `savefig(path)` or `savefig(v, path)` | Save current figure (or a vector) to PNG/SVG |
| `savestem` | `savestem(v, path)` | Save stem plot to PNG/SVG |
| `savedb` | `savedb(freqz_matrix, path)` | Save dB frequency response plot to PNG/SVG |
| `saveimagesc` | `saveimagesc(M, path)` or `saveimagesc(M, path, title, colormap)` | Save matrix heatmap to PNG/SVG |
| `fir_lowpass` | `fir_lowpass(taps, cutoff_hz, sr, window)` | Windowed-sinc FIR lowpass filter |
| `fir_highpass` | `fir_highpass(taps, cutoff_hz, sr, window)` | Windowed-sinc FIR highpass filter |
| `fir_bandpass` | `fir_bandpass(taps, low_hz, high_hz, sr, window)` | Windowed-sinc FIR bandpass filter |
| `fir_lowpass_kaiser` | `fir_lowpass_kaiser(cutoff_hz, tbw_hz, attn_db, sr)` | Kaiser auto-designed lowpass — computes beta and tap count from spec |
| `fir_highpass_kaiser` | `fir_highpass_kaiser(cutoff_hz, tbw_hz, attn_db, sr)` | Kaiser auto-designed highpass |
| `fir_bandpass_kaiser` | `fir_bandpass_kaiser(low_hz, high_hz, tbw_hz, attn_db, sr)` | Kaiser auto-designed bandpass |
| `fir_notch` | `fir_notch(center_hz, bw_hz, sr, taps, window)` | Notch filter via spectral inversion of a bandpass |
| `freqz` | `freqz(h, n_points, sr)` | Complex frequency response — returns 2×n matrix: row 1 = freq axis (Hz), row 2 = H(f) |
| `butterworth_lowpass` | `butterworth_lowpass(order, cutoff_hz, sr)` | Butterworth IIR lowpass filter |
| `butterworth_highpass` | `butterworth_highpass(order, cutoff_hz, sr)` | Butterworth IIR highpass filter |
| `fft` | `fft(v)` | Forward FFT — zero-pads input to next power of two |
| `ifft` | `ifft(V)` | Inverse FFT (input length must be a power of two) |
| `fftshift` | `fftshift(V)` | Shift DC component to center of spectrum |
| `fftfreq` | `fftfreq(n, sr)` | Frequency axis in Hz for an n-point FFT |
| `convolve` | `convolve(x, h)` | Linearly convolve signal `x` with kernel `h` |
| `window` | `window(type, n)` | Generate a window vector of length `n` |
| `eig` | `eig(M)` | Eigenvalues of square matrix `M` as a complex vector |
| `factor` | `factor(n)` | Prime factors of positive integer `n` as a vector |

Window type strings: `"hann"`, `"hamming"`, `"blackman"`, `"rectangular"`, `"kaiser"`.

#### Kaiser filter design parameters

`fir_lowpass_kaiser`, `fir_highpass_kaiser`, and `fir_bandpass_kaiser` accept:

| Parameter | Description |
|-----------|-------------|
| `cutoff_hz` | Passband edge frequency in Hz |
| `tbw_hz` | One-sided transition bandwidth in Hz (controls tap count) |
| `attn_db` | Desired stopband attenuation in dB (controls window beta) |
| `sr` | Sample rate in Hz |

The required beta and number of taps are computed automatically using the Harris (1978) formulas. You no longer need to specify tap count manually.

#### Frequency response with `freqz`

`freqz` returns a 2×n_points matrix. Row 1 is the frequency axis in Hz, row 2 is the complex frequency response H(f):

```
h  = fir_lowpass_kaiser(1000.0, 200.0, 60.0, 8000.0)
H  = freqz(h, 512, 8000.0)
f  = H(1)      # frequency axis (row 1)
Hf = H(2)      # complex response (row 2)
plot(abs(Hf), "Magnitude response")
```

#### Figure-based plotting (multi-series, subplots, file output)

Use `figure()` + `hold()` to compose multi-series plots and save them:

```
t  = linspace(0, 1, 200);
y1 = sin(2 * 3.14159 * 3 * t);
y2 = cos(2 * 3.14159 * 3 * t);

figure()
hold("on")
plot(y1, "color", "blue", "label", "sin")
plot(y2, "color", "red",  "label", "cos")
title("Sine and cosine")
xlabel("Sample")
ylabel("Amplitude")
legend()
savefig("output.svg")
```

Use `subplot(rows, cols, idx)` for multi-panel figures:

```
figure()
subplot(2, 1, 1)
title("Signal")
plot(y1)
subplot(2, 1, 2)
title("Spectrum")
plot(abs(fft(y1)))
savefig("two_panel.svg")
```

Colormaps for `saveimagesc`: `"viridis"` (default), `"jet"`, `"hot"`, `"gray"`.

```
M = rand(32, 32)
saveimagesc(M, "heatmap.svg", "Random matrix", "jet")
```

---

## CLI Reference

### `rustlab run <script>`

Execute a `.r` script file.

```sh
rustlab run examples/bandpass.r
```

### `rustlab filter fir [OPTIONS]`

Design an FIR filter and print coefficients to stdout (one per line).

| Option | Default | Description |
|--------|---------|-------------|
| `--taps N` | 32 | Number of filter taps |
| `--cutoff HZ` | required | Cutoff frequency (low cutoff for bandpass) |
| `--cutoff-high HZ` | — | High cutoff frequency (bandpass only) |
| `--sr HZ` | 44100 | Sample rate in Hz |
| `--type TYPE` | `low` | Filter type: `low`, `high`, `band` |
| `--window WIN` | `hann` | Window function: `hann`, `hamming`, `blackman`, `rectangular`, `kaiser` |
| `--beta B` | — | Kaiser beta parameter |

Examples:

```sh
# 64-tap lowpass at 2 kHz
rustlab filter fir --taps 64 --cutoff 2000 --sr 44100

# Bandpass 500–2000 Hz with Hamming window
rustlab filter fir --type band --cutoff 500 --cutoff-high 2000 --sr 44100 --window hamming

# Highpass with Kaiser window
rustlab filter fir --type high --cutoff 4000 --window kaiser --beta 8.6
```

Output format (one coefficient per line):
```
h[  0] = +0.00012345 + +0.00000000j
h[  1] = +0.00234567 + +0.00000000j
...
```

### `rustlab filter iir [OPTIONS]`

Design an IIR Butterworth filter and print `b` (numerator) and `a` (denominator) coefficients.

| Option | Default | Description |
|--------|---------|-------------|
| `--order N` | 2 | Filter order |
| `--cutoff HZ` | required | Cutoff frequency in Hz |
| `--sr HZ` | 44100 | Sample rate in Hz |
| `--type TYPE` | `low` | Filter type: `low`, `high` |

```sh
rustlab filter iir --order 4 --cutoff 1000 --sr 44100
rustlab filter iir --type high --order 2 --cutoff 8000 --sr 44100
```

### `rustlab convolve [OPTIONS]`

Convolve two signals from CSV files. Prints the result to stdout (one `re,im` per line).

| Option | Default | Description |
|--------|---------|-------------|
| `--signal FILE` | required | Input signal CSV |
| `--kernel FILE` | required | Kernel CSV |
| `--method METHOD` | `direct` | `direct` or `overlap-add` |

Each CSV line is either `re` (real-only) or `re,im`.

```sh
rustlab convolve --signal input.csv --kernel filter.csv
rustlab convolve --signal input.csv --kernel filter.csv --method overlap-add
```

### `rustlab window [OPTIONS]`

Generate a window function and print values to stdout.

| Option | Default | Description |
|--------|---------|-------------|
| `--type TYPE` | required | Window type: `hann`, `hamming`, `blackman`, `rectangular`, `kaiser` |
| `--length N` | required | Number of samples |
| `--beta B` | — | Kaiser beta parameter |
| `--plot` | false | Show an ASCII stem plot |

```sh
rustlab window --type hann --length 32
rustlab window --type kaiser --length 64 --beta 8.6 --plot
```

### `rustlab plot [OPTIONS]`

Plot a CSV signal file (ASCII art rendered to the terminal).

| Option | Default | Description |
|--------|---------|-------------|
| `--input FILE` | required | CSV file (one `re` or `re,im` per line) |
| `--title TITLE` | `Signal` | Plot title |
| `--type TYPE` | `line` | `line` or `stem` |

```sh
rustlab plot --input output.csv --title "Filtered signal"
rustlab plot --input coeffs.csv --type stem --title "Filter coefficients"
```

### `rustlab info`

Print version and usage hints.

```sh
rustlab info
```

---

## Examples

The `examples/` directory contains annotated scripts demonstrating common workflows:

| File | Description |
|------|-------------|
| `examples/complex_basics.r` | Complex number arithmetic, magnitude, phase, complex vectors |
| `examples/vectors.r` | Range operator, indexing, concatenation, element-wise ops, transpose |
| `examples/lowpass.r` | Design and inspect a 32-tap Hann-windowed FIR lowpass filter |
| `examples/bandpass.r` | Bandpass filter design and application to a synthetic dual-tone signal |
| `examples/fft.r` | Compute and plot the spectrum of a two-tone signal; round-trip FFT/IFFT |
| `examples/kaiser_fir.r` | Auto-designed Kaiser FIR lowpass, highpass, bandpass, and notch filters |

Run any example with:

```sh
rustlab run examples/<name>.r
```

See `docs/examples.md` for annotated walkthroughs.
