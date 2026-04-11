# Real-time FIR lowpass filter via stdin/stdout PCM
#
# Usage (macOS):
#   sox -d -t raw -r 44100 -e float -b 32 -c 1 - \
#     | rustlab run examples/audio/realtime_fir.r \
#     | sox -t raw -r 44100 -e float -b 32 -c 1 - -d
#
# Usage (Linux):
#   arecord -f FLOAT_LE -r 44100 -c 1 \
#     | rustlab run examples/audio/realtime_fir.r \
#     | aplay -f FLOAT_LE -r 44100 -c 1
#
# Test without hardware (generate 5 s of 440 Hz sine, filter, save):
#   python3 -c "
#     import struct, math, sys
#     for i in range(44100*5):
#         sys.stdout.buffer.write(struct.pack('f', math.sin(2*math.pi*440*i/44100)))
#   " | rustlab run examples/audio/realtime_fir.r > filtered.raw

sr     = 44100.0;
n_taps = 64;
cutoff = 1000.0 / (sr / 2.0);   # normalise to [0, 1]  (Nyquist = 1)

h  = firpm(n_taps, [0, cutoff * 0.9, cutoff, 1.0], [1, 1, 0, 0]);
st = state_init(length(h) - 1);

adc = audio_in(sr, 256);
dac = audio_out(sr, 256);

while true
    frame     = audio_read(adc);
    [out, st] = filter_stream(frame, h, st);
    audio_write(dac, out);
end
