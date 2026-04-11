# Real-time audio spectrum monitor
#
# Panel 1 (top):    time-domain waveform (input)
# Panel 2 (bottom): magnitude spectrum in dB (DC to Nyquist)
#
# Run with sox (macOS):
#   sox -d -t raw -r 44100 -e float -b 32 -c 1 - \
#     | rustlab run examples/audio/spectrum_monitor.r
#
# Run with arecord (Linux):
#   arecord -f FLOAT_LE -r 44100 -c 1 \
#     | rustlab run examples/audio/spectrum_monitor.r
#
# Test without hardware (5 seconds of 440 Hz + 2 kHz sine):
#   python3 -c "
#     import struct, math, sys
#     sr = 44100; n = sr * 5
#     for i in range(n):
#         s = 0.5*math.sin(2*math.pi*440*i/sr) + 0.5*math.sin(2*math.pi*2000*i/sr)
#         sys.stdout.buffer.write(struct.pack('f', s))
#   " | rustlab run examples/audio/spectrum_monitor.r

sr       = 44100.0;
fft_size = 1024;
half     = fft_size / 2;

% Hann window for spectral leakage reduction
h = window(fft_size, "hann");

% Time axis for waveform panel (milliseconds)
t_ms = linspace(0.0, fft_size / sr * 1000.0, fft_size);

% Frequency axis — keep DC-to-Nyquist half
freqs = fftfreq(fft_size, sr);
f_hz  = freqs(1:half);

adc = audio_in(sr, fft_size);
fig = figure_live(2, 1);

while true
    frame = audio_read(adc);

    % Windowed FFT — magnitude spectrum in dB
    X  = fft(frame .* h);
    Xd = mag2db(X(1:half));

    % Panel 1: waveform (time in ms)
    plot_update(fig, 1, t_ms, frame);

    % Panel 2: spectrum (frequency in Hz, magnitude in dB)
    plot_update(fig, 2, f_hz, Xd);

    figure_draw(fig);
end
