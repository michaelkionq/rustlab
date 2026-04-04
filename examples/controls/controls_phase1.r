% Phase 1 Language Features Example
% Demonstrates: % comments, if/else, disp, fprintf, all/any, rank, roots

% ── Comments ─────────────────────────────────────────────────────────────────
% Both % and # work as line comments.
x = 10  % trailing comment ignored
y = 20  # also ignored

% ── if / else / end ──────────────────────────────────────────────────────────
if x < y
  disp("x is less than y")
else
  disp("x is not less than y")
end

% Scalar condition: zero is false, nonzero is true
flag = 1
if flag
  disp("flag is truthy")
end

% User function with if inside
function label = sign_label(n)
  if n > 0
    label = "positive"
  else
    label = "non-positive"
  end
end

disp(sign_label(7))
disp(sign_label(-3))

% Nested if
function grade = classify(score)
  if score >= 90
    grade = "A"
  else
    if score >= 70
      grade = "B"
    else
      grade = "C"
    end
  end
end

fprintf("score 95 → %s\n", classify(95))
fprintf("score 75 → %s\n", classify(75))
fprintf("score 55 → %s\n", classify(55))

% ── disp and fprintf ─────────────────────────────────────────────────────────
disp("--- disp always appends a newline ---")
disp(3.14159)

fprintf("%-12s  %8.4f\n", "pi",    3.14159)
fprintf("%-12s  %8.4f\n", "e",     2.71828)
fprintf("%-12s  %8.4e\n", "small", 0.0000123)
fprintf("%-12s  %8d\n",   "count", 42)
fprintf("100%% complete\n")

% ── all() and any() ──────────────────────────────────────────────────────────
v = [4, 7, 2, 9, 1];

if all(v > 0)
  disp("all elements are positive")
end

if any(v > 8)
  disp("at least one element exceeds 8")
end

% Element-wise comparison returns a 0/1 vector
above5 = v > 5
all_positive = all(v > 0)
none_negative = all(v >= 0)

% ── rank() ───────────────────────────────────────────────────────────────────
A = [1, 2, 3; 4, 5, 6; 7, 8, 9];  % rank-2: row3 = row2 + (row2 - row1)
B = eye(4);
C = [1, 2; 2, 4];                  % rank-1: row2 is 2x row1

fprintf("rank of 3x3 arithmetic-progression matrix: %d\n", rank(A))
fprintf("rank of 4x4 identity: %d\n", rank(B))
fprintf("rank of [1,2;2,4] (singular 2x2): %d\n", rank(C))

% ── roots() ──────────────────────────────────────────────────────────────────
% Polynomial coefficients in descending power order
% x^2 - 5x + 6  =  (x-2)(x-3)
r1 = roots([1, -5, 6])
fprintf("roots of x^2-5x+6: %.0f and %.0f\n", real(r1(1)), real(r1(2)))

% s^2 + 2s + 10  →  complex conjugate pair
r2 = roots([1, 2, 10])
fprintf("roots of s^2+2s+10: %.2f ± %.2fj\n", real(r2(1)), abs(imag(r2(1))))

% ── Stability check (controls pattern) ───────────────────────────────────────
function check_stability(coeffs)
  p = roots(coeffs)
  if all(real(p) < 0)
    fprintf("  stable   — all %d poles in left half-plane\n", len(p))
  else
    fprintf("  UNSTABLE — pole(s) in right half-plane\n")
  end
end

fprintf("\nStability analysis:\n")
fprintf("  s^2+2s+10:  ")
check_stability([1, 2, 10])
fprintf("  s^2-2s+10:  ")
check_stability([1, -2, 10])
fprintf("  s^2+s+1:    ")
check_stability([1, 1, 1])
fprintf("  s^2-1:      ")
check_stability([1, 0, -1])
