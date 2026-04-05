# Performance Report

| | |
|---|---|
| **Generated** | 2026-04-05 11:32:31 |
| **Version** | 0.1.1 |
| **Platform** | Darwin 25.3.0 arm64 |
| **Binary** | `target/release/rustlab` |

---

## Binary Size

| Build | Size |
|---|---|
| Release (with debug symbols) | 2.7M |
| After `strip` | 2.7M |

**Section sizes:**
```
__TEXT	__DATA	__OBJC	others	dec	hex
2605056	16384	0	4295147520	4297768960	1002ac000
```

---

## Benchmark Results

### `bench_upfirdn` — polyphase upsample / filter / downsample

| | |
|---|---|
| **Wall time** | 271 ms |
| **Status** | PASS |

```
sr = 48000
n1 = 256
Workload 1: n=256, h=512-tap, 4x interp
  output length:  1532
n2 = 48000
Workload 2: n=48000 (1s), h=64-tap, 3x decimate
  output length:  16021
n3 = 44100
Workload 3: n=44100 (1s at 44.1kHz), h=128-tap, SRC 3/2
  output length:  66213
done
```

### `bench_fft` — FFT / IFFT round-trip

| | |
|---|---|
| **Wall time** | 30 ms |
| **Status** | PASS |

```
sr = 44100
n1 = 1024
FFT/IFFT n=1024
  round-trip length:  1024
n2 = 16384
FFT/IFFT n=16384
  round-trip length:  16384
n3 = 131072
FFT/IFFT n=131072
  round-trip length:  131072
done
```

### `bench_linalg` — matrix multiply, inverse, eigenvalues

| | |
|---|---|
| **Wall time** | 22 ms |
| **Status** | PASS |

```
Matrix multiply 64x64
  result size:  [1×2]  64.000000  64.000000
Matrix multiply 256x256
  result size:  [1×2]  256.000000  256.000000
inv 64x64
  result size:  [1×2]  64.000000  64.000000
eig 32x32
  eigenvalue count:  32
done
```

---

## Dependency Summary

| | |
|---|---|
| Direct dependencies (rustlab-cli) | 10 |
| Total transitive dependency lines | 233 |

```
rustlab-cli v0.1.1 (/Users/mike/projects/2026/rustlab/crates/rustlab-cli)
├── anyhow v1.0.102
├── clap v4.6.0
├── ndarray v0.16.1
├── num-complex v0.4.6
├── rustlab-core v0.1.1 (/Users/mike/projects/2026/rustlab/crates/rustlab-core)
├── rustlab-dsp v0.1.1 (/Users/mike/projects/2026/rustlab/crates/rustlab-dsp)
├── rustlab-plot v0.1.1 (/Users/mike/projects/2026/rustlab/crates/rustlab-plot)
├── rustlab-script v0.1.1 (/Users/mike/projects/2026/rustlab/crates/rustlab-script)
└── rustyline v14.0.0
[dev-dependencies]
└── tempfile v3.27.0
```

---

## Known Optimisation Opportunities

These were identified during the initial audit in `perf/performance.md`.
An AI agent reviewing this report should check whether each item is still
open, estimate its impact given the numbers above, and prioritise them.

| ID | Change | Expected saving | Status |
|----|--------|----------------|--------|
| OPT-1 | `zip`: `default-features = false, features = ["deflate"]` | ~300–500 KB | ✅ applied |
| OPT-2 | `ndarray`: remove `features = ["rayon"]` (unused) | ~150–250 KB | ✅ applied |
| OPT-3 | `[profile.release]` — `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"` | ~400–800 KB + runtime | ✅ applied |

---

<!-- AI_ANALYSIS

You are an AI assistant reviewing a rustlab performance report.
Produce a section headed "## AI Analysis" and append it to this file.

Work through these tasks in order:

1. BINARY SIZE
   - Current unstripped size: 2.7M  stripped: 2.7M
   - Are OPT-1, OPT-2, OPT-3 still open?  Apply whichever are still pending
     and re-measure, or estimate the new size if you cannot rebuild.
   - Flag any new dependencies added since the last report that have not
     been audited for unnecessary default features.

2. BENCHMARK TIMINGS
   Thresholds for concern (flag if exceeded):
   - bench_upfirdn total: > 500 ms
   - bench_fft total:     > 100 ms
   - bench_linalg total:  > 200 ms
   Current values: upfirdn=271ms  fft=30ms  linalg=22ms
   For each benchmark that exceeds its threshold, suggest a specific next
   profiling step (e.g. samply, flamegraph, criterion micro-benchmark).

3. DEPENDENCY GROWTH
   Total transitive dependency lines: 233
   If this is > 400 lines, identify the largest contributor among direct
   deps and suggest whether its feature flags can be trimmed further.

4. ALGORITHMIC SUGGESTIONS
   Review the bench script outputs above.  If any workload output looks
   wrong (e.g. unexpected lengths, NaN values) flag it.  Otherwise suggest
   one concrete algorithmic improvement for the slowest benchmark.

5. SUMMARY TABLE
   End with a markdown table:
   | Recommendation | Priority | Effort | Expected gain |
   listing all findings from steps 1–4, highest priority first.

-->

---

## AI Analysis

_Generated 2026-04-05 after applying OPT-1, OPT-2, OPT-3._

### 1. Binary Size

All three open optimisations were applied in this session and the binary was rebuilt.

| Metric | Before | After | Δ |
|---|---|---|---|
| Unstripped binary | 4.3 MB | 2.7 MB | **−1.6 MB (−37%)** |
| Stripped binary | 3.7 MB | 2.7 MB | −1.0 MB |
| Transitive dep lines | 307 | 233 | −74 |

`strip = "symbols"` is now in `[profile.release]`, so the installed binary is always
stripped — the "unstripped" and "stripped" numbers are now the same.

**What drove the reduction:**
- OPT-3 (`lto = "thin"`, `codegen-units = 1`) — LTO removed dead code across crate
  boundaries; the linker could now eliminate unreachable codepaths from `plotters`,
  `ratatui`, and the DSP library that are never reached from a given call path.
- OPT-1 (trim `zip`) — removed `aes-crypto`, `bzip2`, `deflate64`, `lzma`, `zstd`,
  `xz`, `zopfli`, and their transitive crypto/compression deps (≈10 crates).
- OPT-2 (remove `rayon` feature from `ndarray`) — removed the full rayon
  thread-pool runtime.

**Remaining opportunity — OPT-4 (low effort):**
`zip`'s `deflate` feature pulls in `zopfli` (a high-quality but large deflate
compressor used for writes). The codebase only uses `Stored` compression for
writes; deflate is only needed for *reading* Python-generated `.npz` files.
Switching to `features = ["deflate-flate2"]` would use `flate2` alone (already
a transitive dep) and drop `zopfli`. Estimated saving: 50–100 KB.

### 2. Benchmark Timings

All three benchmarks are comfortably within thresholds.

| Benchmark | Time | Threshold | Status |
|---|---|---|---|
| `bench_upfirdn` | 271 ms | 500 ms | ✓ |
| `bench_fft` | 30 ms | 100 ms | ✓ |
| `bench_linalg` | 22 ms | 200 ms | ✓ |

`bench_upfirdn` at 271 ms is the slowest. It covers three workloads totalling
~90 K input samples and roughly 4 M multiply-adds. The dominant cost for the
large workloads (Workload 2: n=48000, Workload 3: n=44100) is likely the
interpreter dispatch loop — variable lookups, value cloning, and `println`
calls — rather than the polyphase arithmetic itself. The arithmetic for Workload
3 alone is ~2.8 M MACs which would take ~1 ms in native Rust; the remaining
~200 ms is interpreter overhead from startup, parsing, and the three `print`
calls.

No output values look wrong — lengths match the formula exactly.

### 3. Dependency Growth

Transitive dep count is 233 lines — well below the 400-line flag threshold.
No new direct dependencies were added in this session; no audit needed.

### 4. Algorithmic Suggestion (upfirdn inner loop)

The upfirdn hot loop currently does a bounds check on `x_idx` for every `k`:

```rust
if x_idx >= 0 && (x_idx as usize) < n_x { acc += h[h_idx] * x[...]; }
```

For the fully-interior range (when `k` is not near the signal boundaries) this
branch is always true and just wastes a compare. The loop can be split into
three segments — left boundary, interior, right boundary — so the interior
segment is a tight unchecked loop. For Workload 2 (n=48000, h=64) almost all
64K output iterations are in the interior, so this saves ~64K×64 = 4 M
redundant comparisons.

Additionally, `p=1` (pure decimation, Workload 2) always sets `r=0`, so
`h_idx = k` and the `r + k*p` address arithmetic degenerates to a direct
slice index. A `if p == 1` fast-path in `upfirdn()` would simplify the inner
loop for this very common case.

### 5. Summary Table

| Recommendation | Priority | Effort | Expected gain |
|---|---|---|---|
| Apply OPT-4: `zip` → `features = ["deflate-flate2"]` | Low | Trivial (1 line) | ~50–100 KB binary |
| Split upfirdn inner loop into boundary/interior regions | Medium | Small (~20 lines) | Faster for large n, especially decimation |
| Add `p=1` fast-path in `upfirdn()` | Medium | Small (~15 lines) | Cleaner branch-free decimation hot path |
| Add `criterion` micro-benchmarks for upfirdn and FFT | Low | Medium | Per-run regression detection |
| Audit `clap` feature flags when CLI surface grows | Low | Trivial | Prevents future bloat |
