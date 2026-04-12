# Spectral Estimation

Comparing two approaches to estimating the power spectrum of a noisy
multi-tone signal.

## Test Signal

Two sinusoids at f1 = 0.15 and f2 = 0.18 cycles/sample, plus noise.
These are intentionally close together to test frequency resolution.

```rustlab
N = 1024;
n = 0:N-1;
x = sin(2*pi*0.15*n) + 0.8*sin(2*pi*0.18*n) + 0.3*randn(N);
plot(x(1:200))
title("Test Signal (first 200 samples)")
xlabel("Sample")
grid on
```

## Direct FFT (Periodogram)

The simplest spectral estimate — magnitude-squared of the FFT:

```rustlab
X = fft(x);
Pxx = abs(X(1:N/2)).^2 / N;
f = linspace(0, 0.5, N/2);
plot(f, 10*log10(Pxx))
title("Periodogram")
xlabel("Frequency (cycles/sample)")
ylabel("Power (dB)")
grid on
```

Both tones are visible, but the periodogram is noisy.

## Windowed FFT (Hann)

Applying a Hann window reduces leakage at the cost of slightly wider peaks:

```rustlab
w = window("hann", N);
xw = x .* w;
Xw = fft(xw);
Pww = abs(Xw(1:N/2)).^2 / sum(w.^2);
plot(f, 10*log10(Pww))
title("Hann-Windowed Periodogram")
xlabel("Frequency (cycles/sample)")
ylabel("Power (dB)")
grid on
```

The noise floor drops and the two tones are clearly resolved.

## Comparison

```rustlab
subplot(2,1,1)
plot(f, 10*log10(Pxx))
title("Rectangular Window"); ylabel("dB"); grid on
subplot(2,1,2)
plot(f, 10*log10(Pww))
title("Hann Window"); ylabel("dB"); xlabel("Frequency"); grid on
```

| Window      | Main Lobe Width | First Sidelobe |
|-------------|-----------------|----------------|
| Rectangular | 2/N             | -13 dB         |
| Hann        | 4/N             | -31 dB         |
