# Vector calculus on uniform 2-D grids
# Demonstrates: gradient(F), divergence(Fx, Fy), curl(Fx, Fy)
#
# Grid convention: F(i, j) ↔ position (x = (j-1)*dx, y = (i-1)*dy).
# Rows index y, columns index x — matches MATLAB / NumPy.
# Trailing `;` suppresses implicit echo for assignments.

# ── Build a uniform grid ─────────────────────────────────────────
dx = 0.1;
dy = 0.1;
xs = -1:dx:1;                          # 21 points across [-1, 1]
ys = -1:dy:1;
[X, Y] = meshgrid(xs, ys);              # both 21×21

# ── 1. gradient of a scalar field ───────────────────────────────
# F(x, y) = x² + y²  →  ∇F = (2x, 2y)
F = X .^ 2 + Y .^ 2;
[Fx, Fy] = gradient(F, dx, dy);

# Centre cell (x = y = 0): both components ≈ 0
print(Fx(11, 11))                      # ≈ 0
print(Fy(11, 11))                      # ≈ 0

# Top-right corner (x = 1, y = 1): boundary one-sided ≈ (2, 2)
print(Fx(21, 21))                      # ≈ 2
print(Fy(21, 21))                      # ≈ 2

# ── 2. divergence of a vector field ─────────────────────────────
# F = (x, y)  →  ∇·F = 2 everywhere
D = divergence(X, Y, dx, dy);
print(D(11, 11))                       # ≈ 2
print(D(1, 1))                         # ≈ 2 (boundary one-sided)

# ── 3. curl of a vector field ───────────────────────────────────
# F = (-y, x)  →  ∇×F · ẑ = 2 (solid-body rotation)
Cz = curl(-Y, X, dx, dy);
print(Cz(11, 11))                      # ≈ 2

# F = (x, y) is irrotational  →  curl = 0
Cz_irr = curl(X, Y, dx, dy);
print(Cz_irr(11, 11))                  # ≈ 0

# ── 4. Combine: ∇²V = ∇·(∇V)  (the Laplacian) ───────────────────
# V(x, y) = x² + y²  →  ∇²V = 4 everywhere
[Vx, Vy] = gradient(F, dx, dy);
laplV = divergence(Vx, Vy, dx, dy);
print(laplV(11, 11))                   # ≈ 4

# ── 5. Complex inputs (frequency-domain field) ──────────────────
# F(x, y) = exp(j*x)  →  ∂F/∂x = j*exp(j*x), ∂F/∂y = 0
Fc = exp(j * X);
[Fxc, Fyc] = gradient(Fc, dx, dy);
print(Fxc(11, 11))                     # ≈ j*exp(0) = 0 + j
print(Fyc(11, 11))                     # ≈ 0

# ── Notes ───────────────────────────────────────────────────────
# • Each axis must have length ≥ 3 (for the 2nd-order one-sided stencil).
# • If you omit dx and dy, both default to 1.0:
#     [Fx, Fy] = gradient(F)
# • Same defaults for divergence(Fx, Fy) and curl(Fx, Fy).
