# Random number generation examples
# rand, randn, randi — and histograms to visualise their distributions

# ── Uniform noise ─────────────────────────────────────────────────────────────
u = rand(2000)
histogram(u, 20)
savehist(u, 20, "rand_hist.svg", "Uniform Distribution [0, 1)")

# ── Gaussian (normal) noise ───────────────────────────────────────────────────
n = randn(2000)
histogram(n, 30)
savehist(n, 30, "randn_hist.svg", "Normal Distribution (μ=0, σ=1)")

# ── Random integers ───────────────────────────────────────────────────────────
# Single integer in [1, 6] — like rolling a die
roll = randi(6)
print(roll)

# 5000 die rolls — should produce roughly equal counts in each bin
rolls = randi(6, 5000)
histogram(rolls, 6)
savehist(rolls, 6, "randi_hist.svg", "Die Rolls: randi(6, 5000)")

# Range with explicit lo/hi: integers in [-5, 5]
signed = randi([-5, 5], 1000)
histogram(signed, 11)
savehist(signed, 11, "randi_signed_hist.svg", "Random Integers [-5, 5]")

# ── Noisy sinusoid ────────────────────────────────────────────────────────────
t     = linspace(0.0, 1.0, 1000)
clean = cos(t * 2.0 * pi * 5.0)
noise = randn(1000) * 0.3
noisy = clean + noise

plot(real(noisy), "Noisy 5 Hz Sinusoid")
savefig(real(noisy), "noisy_sinusoid.svg", "Noisy 5 Hz Sinusoid (SNR ≈ 10 dB)")

# Spectrum of the noisy signal
X = fft(real(noisy))
savefig(abs(fftshift(X)), "noisy_spectrum.svg", "Noisy Sinusoid Spectrum")
