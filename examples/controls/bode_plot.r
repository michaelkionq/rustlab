# bode() — Bode magnitude and phase frequency response

s = tf("s")
G = 10 / (s^2 + 2*s + 10)

# Return magnitude (dB), phase (deg), and frequency (rad/s) vectors
[mag, phase, w] = bode(G)
savefig(mag,   "bode_magnitude.svg", "Bode Magnitude |G(jw)| (dB)")
savefig(phase, "bode_phase.svg",     "Bode Phase /G(jw) (deg)")
