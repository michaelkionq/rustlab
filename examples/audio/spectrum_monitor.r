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
update_every = 172;   # ~1 s at 44100/256

# Hann window and frequency axis
win  = window("hann", fft_size);
freqs = fftfreq(fft_size, sr);
f_hz  = freqs(1:half);

adc = audio_in(sr, frame);
fig = figure_live(1, 1);

buf   = zeros(fft_size);
count = 0;

while true
    samples = audio_read(adc);

    # Write into circular buffer
    base = mod(count, fft_size / frame) * frame;
    for k = 1:frame
        buf(base + k) = real(samples(k));
    end
    count = count + 1;

    # Update plot every ~1 second, skip first cycle (buffer not yet full)
    if count >= (fft_size / frame)
        if mod(count, update_every) == 0
            X  = fft(buf .* win);
            Xd = mag2db(X(1:half));
            plot_update(fig, 1, f_hz, Xd);
            figure_draw(fig);
        end
    end
end
