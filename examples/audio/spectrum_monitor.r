# Real-time audio spectrum monitor
#
# Displays a live single-panel ratatui plot of the Hann-windowed FFT
# magnitude spectrum in dB (DC to Nyquist), updated roughly once per second.
#
# Run with sox (macOS):
#   sox -d -t raw -r 44100 -e float -b 32 -c 1 - \
#     | rustlab run examples/audio/spectrum_monitor.r
#
# Run with arecord (Linux):
#   arecord -f FLOAT_LE -r 44100 -c 1 -t raw \
#     | rustlab run examples/audio/spectrum_monitor.r
#
# Or use the wrapper:
#   ./examples/audio/spectrum_monitor.sh
#   ./examples/audio/spectrum_monitor.sh --test

sr       = 44100.0;
frame    = 256;
fft_size = 4096;
half     = fft_size / 2;
frames_per_update = floor(sr / frame);   # ~172 frames ≈ 1 s

# Hann window and frequency axis
win  = window("hann", fft_size);
freqs = fftfreq(fft_size, sr);
f_hz  = freqs(1:half);

# Accumulation buffer
buf = zeros(fft_size);
pos = 0;

adc = audio_in(sr, frame);
fig = figure_live(1, 1);

while true
    samples = audio_read(adc);

    # Append frame into circular buffer
    for k = 1:frame
        idx = mod(pos, fft_size) + 1;
        buf(idx) = real(samples(k));
        pos = pos + 1;
    end

    if mod(pos, frames_per_update * frame) < frame
        X  = fft(buf .* win);
        Xd = mag2db(X(1:half));
        plot_update(fig, 1, f_hz, Xd);
        figure_draw(fig);
    end
end
