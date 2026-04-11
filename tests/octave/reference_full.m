% Octave reference: generates ref2_*.csv files for full function coverage.
% Run with: octave --no-gui reference_full.m  (from the tests/octave directory)
pkg load signal;

fprintf('Writing Octave reference files...\n');

% ── Math functions ────────────────────────────────────────────────────────────
x = [-3.0, -1.5, 0.0, 1.5, 3.0];

csvwrite('ref2_abs.csv',   abs(x));
csvwrite('ref2_sign.csv',  sign(x));
csvwrite('ref2_floor.csv', floor(x));
csvwrite('ref2_ceil.csv',  ceil(x));
csvwrite('ref2_round.csv', round(x));
csvwrite('ref2_sqrt.csv',  sqrt([0.0, 1.0, 4.0, 9.0, 16.0]));
csvwrite('ref2_exp.csv',   exp([-1.0, 0.0, 1.0, 2.0]));
csvwrite('ref2_log.csv',   log([1.0, exp(1), exp(2), 10.0]));
csvwrite('ref2_log10.csv', log10([1.0, 10.0, 100.0, 1000.0]));
csvwrite('ref2_log2.csv',  log2([1.0, 2.0, 4.0, 8.0]));
csvwrite('ref2_mod.csv',   mod([10.0, -7.0, 5.0], [3.0, 3.0, 3.0]));

% Trig
t = [0.0, pi/6, pi/4, pi/3, pi/2, pi];
csvwrite('ref2_sin.csv',   sin(t));
csvwrite('ref2_cos.csv',   cos(t));
csvwrite('ref2_tanh.csv',  tanh([-1.0, 0.0, 0.5, 1.0]));
csvwrite('ref2_sinh.csv',  sinh([-1.0, 0.0, 0.5, 1.0]));
csvwrite('ref2_cosh.csv',  cosh([-1.0, 0.0, 0.5, 1.0]));

% Inverse trig
csvwrite('ref2_asin.csv',  asin([-1.0, -0.5, 0.0, 0.5, 1.0]));
csvwrite('ref2_acos.csv',  acos([-1.0, -0.5, 0.0, 0.5, 1.0]));
csvwrite('ref2_atan.csv',  atan([-1.0, 0.0, 1.0]));
csvwrite('ref2_atan2.csv', atan2([1.0, -1.0, 0.0, 1.0], [1.0, 1.0, -1.0, 0.0]));

% Complex
vc = [1+2j, 3-1j, -2+0j, 0+4j];
csvwrite('ref2_real.csv',  real(vc));
csvwrite('ref2_imag.csv',  imag(vc));
csvwrite('ref2_angle.csv', angle(vc));
csvwrite('ref2_conj_re.csv', real(conj(vc)));
csvwrite('ref2_conj_im.csv', imag(conj(vc)));
csvwrite('ref2_abs_complex.csv', abs(vc));

% ── Array / Stats ─────────────────────────────────────────────────────────────
v = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];

csvwrite('ref2_sum.csv',    sum(v));
csvwrite('ref2_prod.csv',   prod(v));
csvwrite('ref2_cumsum.csv', cumsum(v));
csvwrite('ref2_mean.csv',   mean(v));
csvwrite('ref2_median.csv', median(v));
csvwrite('ref2_std.csv',    std(v));   % N-1 denominator (Bessel-corrected)
csvwrite('ref2_min.csv',    min(v));
csvwrite('ref2_max.csv',    max(v));
csvwrite('ref2_sort.csv',   sort(v));
csvwrite('ref2_argmin.csv', find(v == min(v), 1));   % 1-based index
csvwrite('ref2_argmax.csv', find(v == max(v), 1));   % 1-based index

% trapz
xq = [0.0, 1.0, 2.0, 3.0, 4.0];
yq = [0.0, 1.0, 4.0, 9.0, 16.0];
csvwrite('ref2_trapz.csv', trapz(xq, yq));

% logspace
csvwrite('ref2_logspace.csv', logspace(0, 3, 7));

% ── Matrix operations ─────────────────────────────────────────────────────────
% eye
csvwrite('ref2_eye.csv', reshape(eye(3), 1, 9));

% diag (create from vector)
d = diag([1.0, 2.0, 3.0]);
csvwrite('ref2_diag_create.csv', reshape(d, 1, 9));

% diag (extract from matrix)
A = [1.0, 2.0, 3.0; 4.0, 5.0, 6.0; 7.0, 8.0, 9.0];
csvwrite('ref2_diag_extract.csv', diag(A)');

% trace
csvwrite('ref2_trace.csv', trace(A));

% reshape: column-major in Octave matches Rust column-major
v4 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
R = reshape(v4, 2, 3);
csvwrite('ref2_reshape.csv', reshape(R, 1, 6));

% repmat
B = [1.0, 2.0; 3.0, 4.0];
RM = repmat(B, 1, 2);
csvwrite('ref2_repmat.csv', reshape(RM, 1, 8));

% transpose
T = A';
csvwrite('ref2_transpose.csv', reshape(T, 1, 9));

% horzcat
H = [A, A(:,1)];   % 3x4
csvwrite('ref2_horzcat.csv', reshape(H, 1, 12));

% vertcat
V = [B; B];   % 4x2
csvwrite('ref2_vertcat.csv', reshape(V, 1, 8));

% ── Linear algebra ────────────────────────────────────────────────────────────
A2 = [4.0, 2.0; 1.0, 3.0];

% dot product
csvwrite('ref2_dot.csv', dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]));

% cross product
csvwrite('ref2_cross.csv', cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]));

% outer product
O = [1.0; 2.0; 3.0] * [4.0, 5.0];
csvwrite('ref2_outer.csv', reshape(O, 1, 6));

% kron
K = kron(eye(2), [1.0, 2.0; 3.0, 4.0]);
csvwrite('ref2_kron.csv', reshape(K, 1, 16));

% norm (vector L2)
csvwrite('ref2_norm_vec.csv', norm([3.0, 4.0]));

% norm (matrix Frobenius)
csvwrite('ref2_norm_mat.csv', norm([1.0, 2.0; 3.0, 4.0], 'fro'));

% det
csvwrite('ref2_det.csv', det(A2));

% inv
Ai = inv(A2);
csvwrite('ref2_inv.csv', reshape(Ai, 1, 4));

% linsolve: Ax=b
Alin = [2.0, 1.0; 1.0, 3.0];
b_lin = [5.0; 10.0];
xsol = Alin \ b_lin;
csvwrite('ref2_linsolve.csv', xsol');

% eig (sorted ascending for comparison)
ev = sort(eig(A2));
csvwrite('ref2_eig.csv', ev');

% svd (singular values only, sorted descending)
sv = svd([1.0, 2.0; 3.0, 4.0; 5.0, 6.0]);
csvwrite('ref2_svd.csv', sv');

% rank
csvwrite('ref2_rank.csv', rank(A2));

% roots (sorted by real part then imag for comparison)
r = roots([1.0, -3.0, 2.0]);
r_sorted = sort(real(r));
csvwrite('ref2_roots.csv', r_sorted');

% expm (matrix exponential of rotation matrix)
Aexp = [0.0, -1.0; 1.0, 0.0];
E = expm(Aexp);
csvwrite('ref2_expm.csv', reshape(E, 1, 4));

% ── DSP ───────────────────────────────────────────────────────────────────────
% filtfilt (FIR)
b_ff = [0.25, 0.5, 0.25];
x_ff = [1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0];
y_ff = filtfilt(b_ff, [1.0], x_ff);
csvwrite('ref2_filtfilt_fir.csv', y_ff);

% filtfilt (IIR - Butterworth 2nd order, LP at 0.3 normalized)
[b2, a2] = butter(2, 0.3);
x_long = sin(2*pi*0.1*(0:19)) + 0.5*sin(2*pi*0.4*(0:19));
y_iir = filtfilt(b2, a2, x_long);
csvwrite('ref2_filtfilt_iir.csv', y_iir);
csvwrite('ref2_butter_b.csv', b2);
csvwrite('ref2_butter_a.csv', a2);

% upfirdn: upsample by 2 with interpolation filter
x_up = [1.0, 2.0, 3.0, 4.0];
h_up = [0.5, 1.0, 0.5];
y_up = upfirdn(x_up, h_up, 2, 1);
csvwrite('ref2_upfirdn.csv', y_up);

% fftfreq: rustlab fftfreq(n, Fs) = k * Fs / n with wrap-around for negative freqs
% This matches numpy's fftfreq(n, d=1/Fs) = k * Fs / n
N_fft = 8;
Fs = 8.0;
freqs = zeros(1, N_fft);
half = floor(N_fft/2);
for k = 0:N_fft-1
  if k <= half - 1 + mod(N_fft,2)
    freqs(k+1) = k * Fs / N_fft;
  else
    freqs(k+1) = (k - N_fft) * Fs / N_fft;
  end
end
csvwrite('ref2_fftfreq.csv', freqs);

% ── Controls / ODE ────────────────────────────────────────────────────────────
% rk4: dx/dt = -x, x(0)=1, integrate to t=1 with 100 steps
% Compare final value: x(1) = exp(-1) ≈ 0.367879441
rk4_exact = exp(-1.0);
csvwrite('ref2_rk4_final.csv', rk4_exact);

% Also save rk4 trajectory at 11 points for comparison
t_rk4 = linspace(0, 1, 11);
x_rk4 = exp(-t_rk4);
csvwrite('ref2_rk4_traj.csv', x_rk4);

fprintf('All reference files written.\n');
