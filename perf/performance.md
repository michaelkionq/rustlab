# Performance & Binary Size

Measurements taken on the current `main` build (v0.1.1, Apple M-series arm64).

---

## Running the benchmarks

All benchmarks are `.r` scripts in this directory. Run them with the release
binary and `time` to measure wall time:

```sh
# Build release first (always measure release, not debug)
cargo build --release

# Individual workloads
time ./target/release/rustlab run perf/bench_upfirdn.r
time ./target/release/rustlab run perf/bench_fft.r
time ./target/release/rustlab run perf/bench_linalg.r

# All at once
for f in perf/bench_*.r; do
  echo "── $f ──"
  time ./target/release/rustlab run "$f"
done
```

For more detailed profiling on macOS use `Instruments` or `samply`:

```sh
# samply (cargo install samply)
samply record ./target/release/rustlab run perf/bench_upfirdn.r

# hyperfine for statistical wall-time (cargo install hyperfine)
hyperfine --warmup 3 './target/release/rustlab run perf/bench_upfirdn.r'
```

---

## Benchmark scripts

| Script | What it measures |
|---|---|
| `bench_upfirdn.r` | polyphase upfirdn — three signal sizes and rate ratios |
| `bench_fft.r` | FFT/IFFT round-trip — 1 K, 16 K, 128 K points |
| `bench_linalg.r` | matrix multiply, inverse, and eigenvalues at 32–256 size |

---

## Current binary size

| Build | Size |
|---|---|
| `cargo build --release` (baseline) | **4.3 MB** |
| After `strip` | **3.7 MB** |

The 0.6 MB difference is debug symbols embedded in the Mach-O binary.
The `__TEXT` (code) section is **3.44 MB**; `__DATA` is 16 KB.

---

## Dependency audit

### `zip` — biggest avoidable cost

`zip`'s default feature set includes every compression codec it supports:

```
default = [aes-crypto, bzip2, deflate64, deflate, lzma, time, zstd, xz]
```

The codebase uses **only `Stored` (no compression) for writing** and needs
**only deflate for reading** Python-generated `.npz` files (NumPy uses
`Compression.ZIP_DEFLATED`). Everything else is dead weight:

| Pulled in by zip defaults | Used? |
|---|---|
| `aes-crypto` (AES, HMAC, PBKDF2, SHA-1) | No |
| `bzip2` | No |
| `deflate64` | No |
| `lzma` / `xz` | No |
| `zstd` | No |
| `deflate` (flate2) | **Yes** — for reading .npz |
| `time` | No |

**Fix:** pin zip to `default-features = false, features = ["deflate"]`.

### `rayon` — enabled but unused

`ndarray` is declared with `features = ["rayon"]` in the workspace, which
compiles the full `rayon` thread-pool runtime into every crate that depends
on `ndarray`. No code in the project calls `par_iter`, `par_azip`, or any
other parallel ndarray operation.

**Fix:** remove the `rayon` feature from the `ndarray` workspace dependency.

### No release profile

The workspace has no `[profile.release]` section, so it uses Rust defaults:
`opt-level = 3`, `lto = false`, `codegen-units = 16`, no stripping.

Without LTO the linker cannot inline or dead-strip across crate boundaries,
leaving unreachable code from large dependencies in the binary.

---

## Recommended changes

### 1. Trim `zip` features

In `Cargo.toml` (`[workspace.dependencies]`):

```toml
# Before
zip = "2"

# After
zip = { version = "2", default-features = false, features = ["deflate"] }
```

Removes: `aes`, `bzip2`, `deflate64`, `hmac`, `lzma`, `pbkdf2`, `sha1`,
`xz`, `zstd`, `zopfli`, `getrandom`, `zeroize` and their transitive deps.

### 2. Remove unused `rayon` from `ndarray`

In `Cargo.toml` (`[workspace.dependencies]`):

```toml
# Before
ndarray = { version = "0.16", features = ["rayon"] }

# After
ndarray = "0.16"
```

Removes the `rayon` thread-pool from all five crates. If parallel matrix
operations are added in future, re-enable it only in the crate that needs it.

### 3. Add a release profile

In the workspace `Cargo.toml`:

```toml
[profile.release]
opt-level     = 3       # already the default, explicit for clarity
lto           = "thin"  # cross-crate dead-code elimination and inlining
codegen-units = 1       # single CGU lets LLVM see the whole program
strip         = "symbols"  # drop debug symbols from the installed binary
```

`lto = "thin"` gives most of the binary-size and speed benefit of full LTO
with much faster link times. Use `lto = true` (fat LTO) for the smallest
possible binary at the cost of longer release builds.

`panic = "abort"` can also be added to remove the unwinding machinery
(saves ~50–100 KB), but check that any code relying on `catch_unwind` still
works — `rustyline` uses it internally for the REPL, so leave this out unless
you verify it is safe.

### 4. Install-time stripping (`make install`)

The Makefile copies the binary; `strip = "symbols"` in the profile will
handle this automatically. If the profile strip is not added, the Makefile
can do it explicitly:

```makefile
install: release
	cp target/release/rustlab $(INSTALL_DIR)/rustlab
	strip $(INSTALL_DIR)/rustlab
	codesign --sign - --force $(INSTALL_DIR)/rustlab  # macOS only
```

---

## Expected impact

These are estimates based on the dependency tree audit; measure after each
change to confirm.

| Change | Expected size reduction |
|---|---|
| Trim `zip` features | ~300–500 KB |
| Remove `rayon` | ~150–250 KB |
| Add `lto = "thin"` | ~200–400 KB (also improves runtime) |
| `strip = "symbols"` | ~600 KB |
| All four combined | **~1.3–1.7 MB** off the current 4.3 MB |

Final estimate: **~2.6–3.0 MB** installed binary.

---

## Future considerations

- **`clap`** pulls in a formatting and help-generation system; if the CLI
  surface grows, consider `clap` with `default-features = false` and only
  the features you use (`derive`, `help`, `error-context`).
- **`plotters`** is already pared down with `default-features = false` — good.
- **`ratatui` / `crossterm`** are the terminal-rendering stack; they are
  well-scoped and unlikely to be a size issue.
- Once `ndarray-linalg` is enabled (the stubbed `linalg` feature), it will
  pull in BLAS/LAPACK — pin carefully and use the `accelerate` backend on
  macOS to avoid bundling a BLAS implementation.
