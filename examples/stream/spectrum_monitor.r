# examples/stream/spectrum_monitor.r
#
# Real-time audio spectrum monitor.
#
# Reads raw f32-LE mono PCM from stdin, applies a 1 kHz FIR lowpass,
# and periodically displays two panels:
#
#   Panel 1  —  time domain of the filtered output  (press any key to advance)
#   Panel 2  —  Hann-windowed FFT magnitude in dB   (press any key to resume)
#
# Processing resumes automatically after the second keypress.
# When stdin closes, the pipeline exits cleanly (exit code 0).
#
# Note: stdout is left as a terminal so plots render in the alternate screen.
# To simultaneously pass filtered audio to speakers, tap the stream with tee:
#
#   sox -d -r 44100 -c 1 -b 32 -e float -t raw - \
#     | tee >(rustlab run examples/stream/spectrum_monitor.r) \
#     | rustlab run examples/stream/filter.r \
#     | sox -r 44100 -c 1 -b 32 -e float -t raw - -d
#
# ─── Platform usage ─────────────────────────────────────────────────────────────
#
# macOS (requires sox):
#   sox -d -r 44100 -c 1 -b 32 -e float -t raw - \
#     | rustlab run examples/stream/spectrum_monitor.r
#
# Linux (ALSA):
#   arecord -r 44100 -c 1 -f FLOAT_LE -t raw \
#     | rustlab run examples/stream/spectrum_monitor.r
#
# Hardware-free synthetic test (440 Hz passband + 4 kHz stopband):
#   python3 -c "
#     import struct, math, sys
#     sr, n = 44100, 44032   # exact multiple of frame size 256
#     for i in range(n):
#       t = i / sr
#       s = 0.5 * math.sin(2 * math.pi *  440 * t) \
#         + 0.5 * math.sin(2 * math.pi * 4000 * t)
#       sys.stdout.buffer.write(struct.pack('<f', s))
#   " | rustlab run examples/stream/spectrum_monitor.r

sr             = 44100.0;
FRAME          = 256;
DISPLAY_FRAMES = 32;                        # frames per snapshot (~186 ms at 44100 Hz)
WIN_SAMPLES    = FRAME * DISPLAY_FRAMES;    # 8192 samples → FFT resolution Δf ≈ 5.4 Hz

# ── FIR design: 1 kHz lowpass (Parks-McClellan, 64 taps) ──────────────────────
# Band edges normalised to [0,1] where 1 = Nyquist (22050 Hz).
cutoff = 1000.0 / (sr / 2.0);
h      = firpm(64, [0.0, cutoff * 0.9, cutoff, 1.0], [1.0, 1.0, 0.0, 0.0]);
state  = state_init(length(h) - 1);

# ── Audio input (no audio_out — stdout stays as terminal for plots) ────────────
src = audio_in(sr, FRAME);

# ── Pre-allocate display buffers ──────────────────────────────────────────────
buf  = zeros(WIN_SAMPLES);                 # filtered output accumulator (im = 0)
win  = window("hann", WIN_SAMPLES);       # analysis window
t_ms = linspace(0.0, WIN_SAMPLES / sr * 1000.0, WIN_SAMPLES);  # time axis (ms)
n    = 0;                                  # frames in current snapshot

# ── Streaming loop ─────────────────────────────────────────────────────────────
while true
  frame      = audio_read(src);
  [y, state] = filter_stream(frame, h, state);

  # Accumulate real output samples into display buffer
  base = n * FRAME;
  for k = 1:FRAME
    buf(base + k) = real(y(k));
  end
  n = n + 1;

  if n >= DISPLAY_FRAMES
    # Hann-windowed FFT → DC-centred magnitude spectrum
    X = fft(buf .* win);
    H = spectrum(X, sr);

    # Panel 1: time domain — press any key to reveal spectrum
    figure()
    subplot(2, 1, 1)
      title("Time domain  (1 kHz lowpass, 186 ms window)")
      xlabel("Time (ms)")
      ylabel("Amplitude")
      plot(t_ms, buf, "label", "filtered")

    # Panel 2: FFT magnitude — press any key to resume collection
    subplot(2, 1, 2)
      title("Spectrum  (Hann window, df ~5.4 Hz)")
      plotdb(H, "FFT magnitude (dB)")

    n = 0;   # reset accumulator for next snapshot
  end
end
