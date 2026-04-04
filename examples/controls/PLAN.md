# Control Systems Feature Plan

Gap analysis for `classical_control.r`. Items are grouped by implementation
phase, ordered so each phase depends only on what precedes it.

---

## Phase 1 — Language Foundations  *(no new types)*

These are self-contained language/builtin additions with no dependencies on
the new control-systems types.

### 1a. `%` comment syntax
`classical_control.r` uses `%` line comments.  Currently only `#` is
recognised.

**Change:** `lexer.rs` — treat `%` identically to `#` (skip to end of line).

---

### 1b. `if / else / end` control flow
Required for the stability check:
```
if all(real(p) < 0)
    disp("stable")
else
    disp("unstable")
end
```

**Changes:**
- `lexer.rs` — add `Token::If`, `Token::Else`
- `ast.rs` — add `Stmt::If { cond, then_body, else_body }`
- `parser.rs` — parse `if expr \n body [else \n body] end`
- `eval/mod.rs` — eval condition (must be `Value::Bool`), run appropriate branch

---

### 1c. Multiple-assignment from function calls
Required for `[Gm, Pm, Wcg, Wcp] = margin(G)` and `[K, S, e] = lqr(...)`.

**Changes:**
- `ast.rs` — add `Stmt::MultiAssign { names: Vec<String>, expr: Expr, suppress: bool }`
- `parser.rs` — recognise `[a, b, ...] = call(...)` in `parse_program`
- `ast.rs` / `value.rs` — add `Value::Tuple(Vec<Value>)` as the return type
  for builtins that produce multiple outputs
- `eval/mod.rs` — unpack `Value::Tuple` into named variables

---

### 1d. `disp()` and `fprintf()` builtins
```
disp("text")           % like print but always newline
fprintf("%.2f\n", x)  % C-style formatted output
```

**Changes:** `eval/builtins.rs`
- `disp(x)` — thin wrapper around `print`, always appends newline
- `fprintf(fmt, arg1, arg2, ...)` — `%d %f %s %g %%` format tokens using
  Rust's own formatting; `\n`, `\t` escape sequences

---

### 1e. `all()` and `any()` builtins
```
all(real(p) < 0)    % returns Bool
any(v)
```

Currently `real(p) < 0` on a vector isn't implemented either — extend binop
comparison operators to produce a `Value::BoolVec` (element-wise bool vector),
or simply require scalar context for now (document the limitation).

Simpler interim: `all(v)` checks that every real part of every element
satisfies `re != 0.0`.

**Changes:** `eval/builtins.rs` — `all(v)`, `any(v)`.

---

### 1f. `rank()` builtin
```
rank(Co)   % integer rank of a matrix, via SVD threshold
```

**Changes:** `eval/builtins.rs` — compute SVD (ndarray-linalg), count singular
values above `eps * max_singular * max(nrows, ncols)`.

---

### 1g. `roots()` builtin
Polynomial roots are needed internally for `pole()` and `zero()`, and are also
generally useful.
```
roots([1, 2, 10])   % → roots of s² + 2s + 10
```

**Changes:** `eval/builtins.rs` — build the companion matrix, return eigenvalues
(reuse existing `eig()` logic).

---

## Phase 2 — Transfer Function Type

### 2a. `Value::TransferFn`
Add a new value variant:
```rust
TransferFn { num: Vec<f64>, den: Vec<f64> }
```
Polynomials stored in descending-power order (`[1.0, 2.0, 10.0]` = s²+2s+10).

**Changes:**
- `value.rs` — add variant, `type_name()`, `Display` (shows as `10 / (s² + 2s + 10)`)
- `value.rs` `binop()` — implement TF arithmetic:
  - `TF * scalar`, `scalar * TF`, `TF / scalar` — scale numerator
  - `scalar / TF` — scalar becomes scalar*[1]/den
  - `TF + TF`, `TF - TF` — common denominator, polynomial add/sub
  - `TF * TF` — polynomial multiply num and den
  - `TF ^ usize` — repeated multiplication
  - `TF ^ scalar` — only for integer exponent, error otherwise

---

### 2b. `tf()` builtin
```
s = tf("s")               % Laplace variable: num=[1,0], den=[1]
G = tf([10], [1, 2, 10])  % explicit numerator/denominator
```

**Changes:** `eval/builtins.rs` — two call signatures.

---

### 2c. `pole()` and `zero()` builtins
```
p = pole(G)   % roots of denominator polynomial
z = zero(G)   % roots of numerator polynomial
```

**Changes:** `eval/builtins.rs` — call `roots()` on `G.den` / `G.num`.

---

## Phase 3 — State-Space Type

### 3a. `Value::StateSpace`
```rust
StateSpace { A: CMatrix, B: CMatrix, C: CMatrix, D: CMatrix }
```
Field access via dot notation (`sys.A`, `sys.B`, etc.) works automatically
because `Value::StateSpace` fields map to the existing `Expr::Field` handler —
the evaluator just needs to special-case `StateSpace` alongside `Struct`.

**Changes:**
- `value.rs` — add variant, `Display`, `type_name()`
- `eval/mod.rs` `Expr::Field` — add match arm for `Value::StateSpace`

---

### 3b. `ss()` builtin
Convert a `TransferFn` to `StateSpace` using observable canonical form:

For G(s) = b₀/(sⁿ + a₁sⁿ⁻¹ + … + aₙ):
```
A = companion matrix (observable form)
B = [0; 0; …; 1]
C = [b₀, 0, …, 0]
D = [0]
```

**Changes:** `eval/builtins.rs`.

---

### 3c. `ctrb()` and `obsv()` builtins
```
Co = ctrb(A, B)   % [B, AB, A²B, …]  (n×n²)
Ob = obsv(A, C)   % [C; CA; CA²; …]  (n²×n)
```

**Changes:** `eval/builtins.rs` — matrix multiply loop.

---

## Phase 4 — Frequency and Time-Domain Analysis

### 4a. `bode()` builtin
Evaluate H(jω) over a log-spaced frequency grid, plot:
- magnitude in dB (upper panel)
- phase in degrees (lower panel)

```
bode(G)                    % auto frequency range
bode(G, w)                 % user-supplied frequency vector
[mag, phase, w] = bode(G)  % return data without plotting
```

**Changes:**
- `eval/builtins.rs` — evaluate `G(jω)` by substituting `s = jω` into ratio
  of polynomials, then call plot builtins for the two-panel figure.

---

### 4b. `step()` builtin
Simulate the unit-step response.  For a stable TF, use partial-fraction
expansion + analytical exponentials, or a simple RK4 ODE on the state-space
form.

```
step(G)                  % plot only
[y, t] = step(G)         % return time vector and output
[y, t] = step(G, t_end)  % user-supplied final time
```

**Changes:** `eval/builtins.rs` — convert to SS, RK4 integrate with u=1 input.

---

### 4c. `margin()` builtin
Compute gain margin (GM) and phase margin (PM) from the Bode data.

```
margin(G)                        % plot with margins annotated
[Gm, Pm, Wcg, Wcp] = margin(G)  % return values only
```

Algorithm:
1. Compute Bode data over a dense frequency grid.
2. Phase crossover frequency Wcg: find ω where ∠H(jω) = −180°.
3. Gain crossover frequency Wcp: find ω where |H(jω)| = 1 (0 dB).
4. GM = 1/|H(jWcg)|,  PM = 180° + ∠H(jWcp).

Returns `Value::Tuple([Gm, Pm, Wcg, Wcp])` for multi-assignment.

**Changes:** `eval/builtins.rs`.

---

## Phase 5 — Optimal Control

### 5a. `lqr()` builtin
Solve the continuous-time algebraic Riccati equation (CARE):

  A'P + PA − PBR⁻¹B'P + Q = 0

Return optimal gain K = R⁻¹B'P, the solution P (called S here), and the
closed-loop eigenvalues e.

```
[K, S, e] = lqr(sys_ss, Q, R)
```

Algorithm: iterative doubling or Hamiltonian eigendecomposition.
Crate `control` (crates.io) has a CARE solver; alternatively implement
via Hamiltonian matrix eigendecomposition using the existing `eig()` code.

Returns `Value::Tuple([K, S, e])`.

**Changes:** `eval/builtins.rs`, possibly new dependency.

---

## Phase 6 — Root Locus *(optional / advanced)*

### 6a. `rlocus()` builtin
Plot the paths of closed-loop poles as the scalar gain K varies from 0 to ∞.

```
rlocus(G)
```

Algorithm: for K in a log-spaced vector, compute roots of `den + K*num = 0`
using the companion-matrix eigenvalue method, plot each set of roots as a
point on the complex plane.

**Changes:** `eval/builtins.rs`, plotting (scatter/dot style on complex plane).

---

## Implementation Order Summary

| Phase | Items | Key dependency |
|-------|-------|---------------|
| 1 | `%` comments, `if/else`, multi-assign, `disp`, `fprintf`, `all`, `rank`, `roots` | none |
| 2 | `Value::TransferFn`, TF arithmetic, `tf`, `pole`, `zero` | Phase 1 (`roots`) |
| 3 | `Value::StateSpace`, `ss`, `ctrb`, `obsv` | Phase 2 |
| 4 | `bode`, `step`, `margin` | Phases 2–3 |
| 5 | `lqr` | Phase 3 |
| 6 | `rlocus` | Phase 2 |

Phase 1 items are independent of each other and can be done in any order or
in parallel.  Phases 2–6 must follow sequentially because each builds on the
types defined before it.
