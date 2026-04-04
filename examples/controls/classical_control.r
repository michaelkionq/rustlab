% Classical Control Systems — Transfer Function Analysis
% Demonstrates TF creation, step response, Bode plot, stability analysis,
% state-space conversion, controllability/observability, and LQR design.
%
% NOTE: Several features used here require implementation (see PLAN.md).
%       This file is the target syntax once those features land.

% ── Part 1: Transfer Function Definition ─────────────────────────────────────

% Create the Laplace variable 's'
s = tf("s");

% Build G(s) = 10 / (s² + 2s + 10)  using TF arithmetic
G = 10 / (s^2 + 2*s + 10);
disp("Transfer Function G(s):");
disp(G)

% ── Part 2: Time-Domain Step Response ────────────────────────────────────────

figure("Name", "Classical Control Basics");
subplot(2, 1, 1);
step(G);
title("Step Response of G(s)");
grid on;

% ── Part 3: Frequency-Domain Bode Plot ───────────────────────────────────────

subplot(2, 1, 2);
bode(G);
title("Bode Plot of G(s)");
grid on;

% ── Part 4: Poles, Zeros, and Stability ──────────────────────────────────────

p = pole(G);
z = zero(G);
disp("System Poles:"); disp(p);

if all(real(p) < 0)
    disp("System is strictly stable (all poles in the left half-plane).");
else
    disp("System is unstable or marginally stable.");
end

% ── Part 5: Root Locus ───────────────────────────────────────────────────────

figure("Name", "Stability Analysis");
rlocus(G);
title("Root Locus of G(s)");
grid on;

% ── Part 6: Gain and Phase Margins ───────────────────────────────────────────

figure("Name", "Stability Margins");
margin(G);
[Gm, Pm, Wcg, Wcp] = margin(G);
fprintf("Gain Margin:  %.2f dB at %.2f rad/s\n", 20*log10(Gm), Wcg);
fprintf("Phase Margin: %.2f deg at %.2f rad/s\n", Pm, Wcp);

% ── Part 7: State-Space Conversion ───────────────────────────────────────────

sys_ss = ss(G);
A = sys_ss.A;
B = sys_ss.B;
C = sys_ss.C;
D = sys_ss.D;
disp("State-Space A Matrix:"); disp(A);

% ── Part 8: Controllability and Observability ─────────────────────────────────

Co   = ctrb(A, B);
unco = length(A) - rank(Co);
if unco == 0
    disp("System is fully controllable.");
else
    fprintf("System has %d uncontrollable states.\n", unco);
end

Ob   = obsv(A, C);
unob = length(A) - rank(Ob);
if unob == 0
    disp("System is fully observable.");
else
    fprintf("System has %d unobservable states.\n", unob);
end

% ── Part 9: LQR Design ───────────────────────────────────────────────────────

Q = diag([10, 1]);   % state weighting
R = 1;               % control-effort weighting
[K, S, e] = lqr(sys_ss, Q, R);
disp("Optimal LQR Feedback Gain K:"); disp(K);
disp("Closed-loop eigenvalues with LQR:"); disp(e);
