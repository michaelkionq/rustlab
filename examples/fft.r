# FFT example
# Compute the spectrum of a two-tone signal at 8 kHz sample rate

sr = 8000.0
n  = 256

# Build a time vector and a signal with tones at 500 Hz and 1500 Hz
t = linspace(0.0, (n - 1) / sr, n)
x = cos(t * 2.0 * pi * 500.0) + cos(t * 2.0 * pi * 1500.0)

# Save the input signal
savefig(real(x), "fft_input.svg", "Input Signal (500 Hz + 1500 Hz)")

# Forward FFT
X = fft(x)

# spectrum() applies fftshift and pairs with the Hz frequency axis,
# returning a 2×n matrix that plugs straight into plotdb / savedb.
H = spectrum(X, sr)

# Interactive: dB magnitude spectrum with Hz x-axis
plotdb(H, "Magnitude Spectrum")

# Save to file
savedb(H, "fft_spectrum.svg", "Magnitude Spectrum")

# Round-trip: reconstruct original signal from spectrum
x_rec = real(ifft(X))
savefig(x_rec, "fft_reconstructed.svg", "Reconstructed Signal")
