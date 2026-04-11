# Rustlab full test script — produces out2_*.csv files matching reference_full.m

# ── Math functions ────────────────────────────────────────────────────────────
x = [-3.0, -1.5, 0.0, 1.5, 3.0]

save("out2_abs.csv",   abs(x))
save("out2_sign.csv",  sign(x))
save("out2_floor.csv", floor(x))
save("out2_ceil.csv",  ceil(x))
save("out2_round.csv", round(x))
save("out2_sqrt.csv",  sqrt([0.0, 1.0, 4.0, 9.0, 16.0]))
save("out2_exp.csv",   exp([-1.0, 0.0, 1.0, 2.0]))
save("out2_log.csv",   log([1.0, exp(1.0), exp(2.0), 10.0]))
save("out2_log10.csv", log10([1.0, 10.0, 100.0, 1000.0]))
save("out2_log2.csv",  log2([1.0, 2.0, 4.0, 8.0]))

# mod: test scalar cases individually
m1 = mod(10.0, 3.0)
m2 = mod(-7.0, 3.0)
m3 = mod(5.0, 3.0)
save("out2_mod.csv", [m1, m2, m3])

# Trig
t = [0.0, 0.5235987755982988, 0.7853981633974483, 1.0471975511965976, 1.5707963267948966, 3.141592653589793]
save("out2_sin.csv",   sin(t))
save("out2_cos.csv",   cos(t))
save("out2_tanh.csv",  tanh([-1.0, 0.0, 0.5, 1.0]))
save("out2_sinh.csv",  sinh([-1.0, 0.0, 0.5, 1.0]))
save("out2_cosh.csv",  cosh([-1.0, 0.0, 0.5, 1.0]))

# Inverse trig
save("out2_asin.csv",  asin([-1.0, -0.5, 0.0, 0.5, 1.0]))
save("out2_acos.csv",  acos([-1.0, -0.5, 0.0, 0.5, 1.0]))
save("out2_atan.csv",  atan([-1.0, 0.0, 1.0]))
y_a2 = [1.0, -1.0, 0.0, 1.0]
x_a2 = [1.0, 1.0, -1.0, 0.0]
save("out2_atan2.csv", atan2(y_a2, x_a2))

# Complex
vc = [1.0+j*2.0, 3.0-j*1.0, -2.0+j*0.0, 0.0+j*4.0]
save("out2_real.csv",      real(vc))
save("out2_imag.csv",      imag(vc))
save("out2_angle.csv",     angle(vc))
save("out2_conj_re.csv",   real(conj(vc)))
save("out2_conj_im.csv",   imag(conj(vc)))
save("out2_abs_complex.csv", abs(vc))

# ── Array / Stats ─────────────────────────────────────────────────────────────
v = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0]

save("out2_sum.csv",    sum(v))
save("out2_prod.csv",   prod(v))
save("out2_cumsum.csv", cumsum(v))
save("out2_mean.csv",   mean(v))
save("out2_median.csv", median(v))
save("out2_std.csv",    std(v))
save("out2_min.csv",    min(v))
save("out2_max.csv",    max(v))
save("out2_sort.csv",   sort(v))
save("out2_argmin.csv", argmin(v))
save("out2_argmax.csv", argmax(v))

# trapz
xq = [0.0, 1.0, 2.0, 3.0, 4.0]
yq = [0.0, 1.0, 4.0, 9.0, 16.0]
save("out2_trapz.csv", trapz(xq, yq))

# logspace
save("out2_logspace.csv", logspace(0.0, 3.0, 7))

# ── Matrix operations ─────────────────────────────────────────────────────────
# eye
E3 = eye(3)
save("out2_eye.csv", reshape(E3, 1, 9))

# diag (create from vector)
d = diag([1.0, 2.0, 3.0])
save("out2_diag_create.csv", reshape(d, 1, 9))

# diag (extract from matrix)
A = [1.0, 2.0, 3.0; 4.0, 5.0, 6.0; 7.0, 8.0, 9.0]
save("out2_diag_extract.csv", diag(A))

# trace
save("out2_trace.csv", trace(A))

# reshape
v4 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
R = reshape(v4, 2, 3)
save("out2_reshape.csv", reshape(R, 1, 6))

# repmat
B = [1.0, 2.0; 3.0, 4.0]
RM = repmat(B, 1, 2)
save("out2_repmat.csv", reshape(RM, 1, 8))

# transpose
T = A'
save("out2_transpose.csv", reshape(T, 1, 9))

# horzcat: A is 3x3, append column [1;4;7]
col1 = [1.0; 4.0; 7.0]
H = horzcat(A, col1)
save("out2_horzcat.csv", reshape(H, 1, 12))

# vertcat
V = vertcat(B, B)
save("out2_vertcat.csv", reshape(V, 1, 8))

# ── Linear algebra ────────────────────────────────────────────────────────────
A2 = [4.0, 2.0; 1.0, 3.0]

# dot product
save("out2_dot.csv", dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]))

# cross product
save("out2_cross.csv", cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]))

# outer product
o1 = [1.0, 2.0, 3.0]
o2 = [4.0, 5.0]
O = outer(o1, o2)
save("out2_outer.csv", reshape(O, 1, 6))

# kron
K = kron(eye(2), [1.0, 2.0; 3.0, 4.0])
save("out2_kron.csv", reshape(K, 1, 16))

# norm (vector L2)
save("out2_norm_vec.csv", norm([3.0, 4.0]))

# norm (matrix Frobenius)
save("out2_norm_mat.csv", norm([1.0, 2.0; 3.0, 4.0]))

# det
save("out2_det.csv", det(A2))

# inv
Ai = inv(A2)
save("out2_inv.csv", reshape(Ai, 1, 4))

# linsolve: Ax=b
Alin = [2.0, 1.0; 1.0, 3.0]
b_lin = [5.0; 10.0]
xsol = linsolve(Alin, b_lin)
save("out2_linsolve.csv", xsol)

# eig (sorted ascending)
ev = eig(A2)
ev_sorted = sort(ev)
save("out2_eig.csv", ev_sorted)

# svd (singular values only)
A_svd = [1.0, 2.0; 3.0, 4.0; 5.0, 6.0]
[U_svd, S_svd, V_svd] = svd(A_svd)
sv = sort(S_svd)
# sort gives ascending, we need descending to match octave svd order
sv_desc = [sv(length(sv)), sv(length(sv)-1)]
save("out2_svd.csv", sv_desc)

# rank
save("out2_rank.csv", rank(A2))

# roots (sorted by real part for comparison)
r = roots([1.0, -3.0, 2.0])
r_sorted = sort(r)
save("out2_roots.csv", r_sorted)

# expm
Aexp = [0.0, -1.0; 1.0, 0.0]
E = expm(Aexp)
save("out2_expm.csv", reshape(E, 1, 4))

# ── DSP ───────────────────────────────────────────────────────────────────────
# filtfilt (FIR)
b_ff = [0.25, 0.5, 0.25]
x_ff = [1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0]
y_ff = filtfilt(b_ff, [1.0], x_ff)
save("out2_filtfilt_fir.csv", y_ff)

# upfirdn: upsample by 2
x_up = [1.0, 2.0, 3.0, 4.0]
h_up = [0.5, 1.0, 0.5]
y_up = upfirdn(x_up, h_up, 2, 1)
save("out2_upfirdn.csv", y_up)

# fftfreq: rustlab takes sample_rate (Fs), not sample spacing
f_freq = fftfreq(8, 8.0)
save("out2_fftfreq.csv", f_freq)

# ── Controls / ODE ────────────────────────────────────────────────────────────
# rk4: dx/dt = -x, x(0)=1, integrate to t=1 with 11 steps
f_decay = @(x, t) -x
t_rk4 = linspace(0.0, 1.0, 11)
x_rk4 = rk4(f_decay, 1.0, t_rk4)
save("out2_rk4_traj.csv", x_rk4)
save("out2_rk4_final.csv", x_rk4(11))
