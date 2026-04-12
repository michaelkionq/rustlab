# Lowpass Filter Analysis

## Design

We design a 64-tap FIR lowpass filter with a cutoff at 3 kHz (sample rate
16 kHz) using `fir_lowpass`, then verify its frequency response with
`freqz`.

```rustlab
h = fir_lowpass(64, 3000, 16000, "hamming");
Hw = freqz(h, 512, 16000);
w = Hw(1,:);
H = Hw(2,:);
plot(w, 20*log10(abs(H)))
title("Magnitude Response (dB)")
xlabel("Frequency (Hz)")
ylabel("Magnitude (dB)")
grid on
```

The passband is flat and the stopband rejection is strong.

## Impulse Response

```rustlab
stem(h)
title("Filter Impulse Response")
xlabel("Sample")
ylabel("Amplitude")
grid on
```

The symmetric shape confirms this is a linear-phase Type I FIR filter.

## Filtering a Noisy Signal

Now we test the filter on a noisy sinusoid using convolution.

```rustlab
n = 0:255;
x = sin(2*pi*0.1*n) + 0.5*randn(256);
y = convolve(h, x);
subplot(2,1,1); plot(x); title("Input (noisy)"); grid on
subplot(2,1,2); plot(y(1:256)); title("Filtered Output"); grid on
```

The filter cleanly recovers the underlying sinusoid.
