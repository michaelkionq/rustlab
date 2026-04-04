# FIR lowpass filter design
# 32-tap Hann-windowed sinc at 1 kHz, 44.1 kHz sample rate

h = fir_lowpass(32, 1000.0, 44100.0, "hann")
print(h)

# Interactive: impulse response stem plot
stem(real(h), "Lowpass Impulse Response")

# Save impulse response to file
savestem(real(h), "lowpass_impulse.svg", "Lowpass Impulse Response")

# Frequency response
Hz = freqz(h, 512, 44100.0)

# Interactive: dB magnitude response with Hz x-axis
plotdb(Hz, "Lowpass Frequency Response")

# Save frequency response to file
savedb(Hz, "lowpass_response.svg", "Lowpass Frequency Response")
