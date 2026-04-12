# HTML Report generation demo
# Run: cargo run --bin rustlab --features viewer -- run examples/report_demo.r
#
# Demonstrates the report feature:
#   report start "title"   — begin a report session
#   report save            — save to report/index.html
#   report save "path"     — save to custom path
#
# Figures are captured automatically on clf, figure(), and report save.
# The generated HTML has a sidebar navigation and interactive Plotly charts.

report start "DSP Analysis Report"

% --- Time-domain signals ---
t = linspace(0, 0.01, 1000)
f1 = 440
f2 = 1200
x = sin(2*pi*f1*t) + 0.5*sin(2*pi*f2*t)
plot(t*1000, x)
title("Two-Tone Signal (440 Hz + 1200 Hz)")
xlabel("Time (ms)")
ylabel("Amplitude")
grid on

% --- Spectrum (clf captures the previous figure) ---
clf
N = length(x)
Fs = 100000
X = fft(x)
freqs = fftfreq(N, Fs)
Nhalf = N/2
mag = abs(X(1:Nhalf))
plot(freqs(1:Nhalf), mag)
title("Magnitude Spectrum")
xlabel("Frequency (Hz)")
ylabel("|X(f)|")
grid on

% --- FIR lowpass filter design ---
clf
h = fir_lowpass(63, 800, Fs, "hamming")
Hmat = freqz(h, 512, Fs)
w = Hmat(1,:)
H = Hmat(2,:)
subplot(2, 1, 1)
stem(h)
title("Lowpass FIR Coefficients (63 taps, fc=800 Hz)")
xlabel("Tap Index")
ylabel("h[n]")
subplot(2, 1, 2)
plot(w, 20*log10(abs(H)))
title("Frequency Response")
xlabel("Frequency (Hz)")
ylabel("Magnitude (dB)")
ylim([-80, 5])
grid on

% --- Filtered output ---
clf
y = convolve(x, h)
y = y(1:length(x))
subplot(2, 1, 1)
plot(t*1000, x)
title("Input Signal")
xlabel("Time (ms)")
ylabel("Amplitude")
subplot(2, 1, 2)
plot(t*1000, y)
title("Filtered Output (1200 Hz removed)")
xlabel("Time (ms)")
ylabel("Amplitude")

% --- Histogram of noise ---
clf
n = randn(10000)
hist(n, 50)
title("Gaussian Noise Distribution (10k samples)")
xlabel("Value")
ylabel("Count")

% --- Save (auto-captures the last figure) ---
report save "report_demo.html"
fprintf("Report saved to report_demo.html\n")
