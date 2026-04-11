% Octave comparison script for full function coverage.
% Run with: octave --no-gui compare_full.m  (from tests/octave directory)

pass_count = 0;
fail_count = 0;

function result = compare_vectors(name, ref, out, tol)
  if length(ref) ~= length(out)
    fprintf('FAIL  %-35s  length mismatch: ref=%d out=%d\n', name, length(ref), length(out));
    result = false;
    return;
  end
  err = max(abs(ref - out));
  if err <= tol
    fprintf('PASS  %-35s  max_err=%.2e  (tol=%.2e)\n', name, err, tol);
    result = true;
  else
    fprintf('FAIL  %-35s  max_err=%.2e  (tol=%.2e)\n', name, err, tol);
    result = false;
  end
end

function [p, f] = check(name, ref_file, out_file, tol)
  try
    ref = csvread(ref_file)(:)';
    out = csvread(out_file)(:)';
    ok  = compare_vectors(name, ref, out, tol);
    if ok; p=1; f=0; else p=0; f=1; end
  catch e
    fprintf('FAIL  %-35s  error: %s\n', name, e.message);
    p=0; f=1;
  end
end

fprintf('%-6s %-35s\n', 'Result', 'Test');
fprintf('%s\n', repmat('-', 1, 70));

T_EXACT  = 1e-9;
T_TRIG   = 1e-9;
T_ITER   = 1e-4;

% ── Math functions ────────────────────────────────────────────────────────────
[p,f]=check('abs',           'ref2_abs.csv',          'out2_abs.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('sign',          'ref2_sign.csv',         'out2_sign.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('floor',         'ref2_floor.csv',        'out2_floor.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('ceil',          'ref2_ceil.csv',         'out2_ceil.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('round',         'ref2_round.csv',        'out2_round.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('sqrt',          'ref2_sqrt.csv',         'out2_sqrt.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('exp',           'ref2_exp.csv',          'out2_exp.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('log',           'ref2_log.csv',          'out2_log.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('log10',         'ref2_log10.csv',        'out2_log10.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('log2',          'ref2_log2.csv',         'out2_log2.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('mod',           'ref2_mod.csv',          'out2_mod.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('sin',           'ref2_sin.csv',          'out2_sin.csv',          T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('cos',           'ref2_cos.csv',          'out2_cos.csv',          T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('tanh',          'ref2_tanh.csv',         'out2_tanh.csv',         T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('sinh',          'ref2_sinh.csv',         'out2_sinh.csv',         T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('cosh',          'ref2_cosh.csv',         'out2_cosh.csv',         T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('asin',          'ref2_asin.csv',         'out2_asin.csv',         T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('acos',          'ref2_acos.csv',         'out2_acos.csv',         T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('atan',          'ref2_atan.csv',         'out2_atan.csv',         T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('atan2',         'ref2_atan2.csv',        'out2_atan2.csv',        T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('real',          'ref2_real.csv',         'out2_real.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('imag',          'ref2_imag.csv',         'out2_imag.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('angle',         'ref2_angle.csv',        'out2_angle.csv',        T_TRIG);  pass_count+=p; fail_count+=f;
[p,f]=check('conj (re)',     'ref2_conj_re.csv',      'out2_conj_re.csv',      T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('conj (im)',     'ref2_conj_im.csv',      'out2_conj_im.csv',      T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('abs (complex)', 'ref2_abs_complex.csv',  'out2_abs_complex.csv',  T_EXACT); pass_count+=p; fail_count+=f;

% ── Array / Stats ─────────────────────────────────────────────────────────────
[p,f]=check('sum',           'ref2_sum.csv',          'out2_sum.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('prod',          'ref2_prod.csv',         'out2_prod.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('cumsum',        'ref2_cumsum.csv',       'out2_cumsum.csv',       T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('mean',          'ref2_mean.csv',         'out2_mean.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('median',        'ref2_median.csv',       'out2_median.csv',       T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('std',           'ref2_std.csv',          'out2_std.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('min',           'ref2_min.csv',          'out2_min.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('max',           'ref2_max.csv',          'out2_max.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('sort',          'ref2_sort.csv',         'out2_sort.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('argmin',        'ref2_argmin.csv',       'out2_argmin.csv',       T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('argmax',        'ref2_argmax.csv',       'out2_argmax.csv',       T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('trapz',         'ref2_trapz.csv',        'out2_trapz.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('logspace',      'ref2_logspace.csv',     'out2_logspace.csv',     T_EXACT); pass_count+=p; fail_count+=f;

% ── Matrix operations ─────────────────────────────────────────────────────────
[p,f]=check('eye',           'ref2_eye.csv',          'out2_eye.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('diag (create)', 'ref2_diag_create.csv',  'out2_diag_create.csv',  T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('diag (extract)','ref2_diag_extract.csv', 'out2_diag_extract.csv', T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('trace',         'ref2_trace.csv',        'out2_trace.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('reshape',       'ref2_reshape.csv',      'out2_reshape.csv',      T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('repmat',        'ref2_repmat.csv',       'out2_repmat.csv',       T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('transpose',     'ref2_transpose.csv',    'out2_transpose.csv',    T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('horzcat',       'ref2_horzcat.csv',      'out2_horzcat.csv',      T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('vertcat',       'ref2_vertcat.csv',      'out2_vertcat.csv',      T_EXACT); pass_count+=p; fail_count+=f;

% ── Linear algebra ────────────────────────────────────────────────────────────
[p,f]=check('dot',           'ref2_dot.csv',          'out2_dot.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('cross',         'ref2_cross.csv',        'out2_cross.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('outer',         'ref2_outer.csv',        'out2_outer.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('kron',          'ref2_kron.csv',         'out2_kron.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('norm (vector)', 'ref2_norm_vec.csv',     'out2_norm_vec.csv',     T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('norm (matrix)', 'ref2_norm_mat.csv',     'out2_norm_mat.csv',     T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('det',           'ref2_det.csv',          'out2_det.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('inv',           'ref2_inv.csv',          'out2_inv.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('linsolve',      'ref2_linsolve.csv',     'out2_linsolve.csv',     T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('eig (sorted)',  'ref2_eig.csv',          'out2_eig.csv',          T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('svd (values)',  'ref2_svd.csv',          'out2_svd.csv',          T_ITER);  pass_count+=p; fail_count+=f;
[p,f]=check('rank',          'ref2_rank.csv',         'out2_rank.csv',         T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('roots',         'ref2_roots.csv',        'out2_roots.csv',        T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('expm',          'ref2_expm.csv',         'out2_expm.csv',         T_ITER);  pass_count+=p; fail_count+=f;

% ── DSP ───────────────────────────────────────────────────────────────────────
[p,f]=check('filtfilt (FIR)','ref2_filtfilt_fir.csv', 'out2_filtfilt_fir.csv', T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('upfirdn',       'ref2_upfirdn.csv',      'out2_upfirdn.csv',      T_EXACT); pass_count+=p; fail_count+=f;
[p,f]=check('fftfreq',       'ref2_fftfreq.csv',      'out2_fftfreq.csv',      T_EXACT); pass_count+=p; fail_count+=f;

% ── Controls / ODE ────────────────────────────────────────────────────────────
[p,f]=check('rk4 (final)',   'ref2_rk4_final.csv',    'out2_rk4_final.csv',    T_ITER);  pass_count+=p; fail_count+=f;
[p,f]=check('rk4 (traj)',    'ref2_rk4_traj.csv',     'out2_rk4_traj.csv',     T_ITER);  pass_count+=p; fail_count+=f;

fprintf('%s\n', repmat('-', 1, 70));
fprintf('Total: %d passed, %d failed\n', pass_count, fail_count);
if fail_count == 0
  fprintf('ALL TESTS PASSED\n');
else
  fprintf('SOME TESTS FAILED\n');
end
