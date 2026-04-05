#!/usr/bin/env bash
# perf/run_perf.sh — build release binary, time every benchmark, write report.md
# Usage: ./perf/run_perf.sh  (or via  make perf)

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BINARY="$ROOT/target/release/rustlab"
REPORT="$ROOT/perf/report.md"
PERF_DIR="$ROOT/perf"

# ── helpers ───────────────────────────────────────────────────────────────────

ms() { python3 -c "import time; print(int(time.time()*1000))"; }

# Run one bench script; sets BENCH_OUT, BENCH_MS, BENCH_STATUS
run_bench() {
    local script="$1"
    local t0 t1
    t0=$(ms)
    if BENCH_OUT=$("$BINARY" run "$script" 2>&1); then
        BENCH_STATUS="PASS"
    else
        BENCH_STATUS="FAIL"
    fi
    t1=$(ms)
    BENCH_MS=$(( t1 - t0 ))
}

# ── build ─────────────────────────────────────────────────────────────────────

echo "Building release binary..."
cd "$ROOT"
cargo build --release -q
echo "  done."

# ── binary size ───────────────────────────────────────────────────────────────

BINARY_SIZE_H=$(ls -lh "$BINARY" | awk '{print $5}')

cp "$BINARY" /tmp/rustlab_perf_stripped
strip /tmp/rustlab_perf_stripped
STRIPPED_SIZE_H=$(ls -lh /tmp/rustlab_perf_stripped | awk '{print $5}')
rm /tmp/rustlab_perf_stripped

# ── run benchmarks ────────────────────────────────────────────────────────────

echo "Running benchmarks..."

echo "  bench_upfirdn..."
run_bench "$PERF_DIR/bench_upfirdn.r"
MS_UPFIRDN=$BENCH_MS; OUT_UPFIRDN=$BENCH_OUT; ST_UPFIRDN=$BENCH_STATUS

echo "  bench_fft..."
run_bench "$PERF_DIR/bench_fft.r"
MS_FFT=$BENCH_MS; OUT_FFT=$BENCH_OUT; ST_FFT=$BENCH_STATUS

echo "  bench_linalg..."
run_bench "$PERF_DIR/bench_linalg.r"
MS_LINALG=$BENCH_MS; OUT_LINALG=$BENCH_OUT; ST_LINALG=$BENCH_STATUS

# ── dependency counts ─────────────────────────────────────────────────────────

DIRECT_DEPS=$(cargo tree --depth 1 -p rustlab-cli 2>/dev/null | grep -c "^[├└]" || echo "?")
TOTAL_DEPS=$(cargo tree -p rustlab-cli 2>/dev/null | wc -l | tr -d ' ')
DEP_TREE=$(cargo tree --depth 1 -p rustlab-cli 2>/dev/null || echo "n/a")

# ── write report ──────────────────────────────────────────────────────────────

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
PLATFORM=$(uname -srm)
VERSION=$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
SIZE_TABLE=$(size "$BINARY" 2>/dev/null || echo "n/a")

{
cat << HEADER
# Performance Report

| | |
|---|---|
| **Generated** | $TIMESTAMP |
| **Version** | $VERSION |
| **Platform** | $PLATFORM |
| **Binary** | \`target/release/rustlab\` |

---

## Binary Size

| Build | Size |
|---|---|
| Release (with debug symbols) | $BINARY_SIZE_H |
| After \`strip\` | $STRIPPED_SIZE_H |

**Section sizes:**
\`\`\`
$SIZE_TABLE
\`\`\`

---

## Benchmark Results

### \`bench_upfirdn\` — polyphase upsample / filter / downsample

| | |
|---|---|
| **Wall time** | ${MS_UPFIRDN} ms |
| **Status** | $ST_UPFIRDN |

\`\`\`
$OUT_UPFIRDN
\`\`\`

### \`bench_fft\` — FFT / IFFT round-trip

| | |
|---|---|
| **Wall time** | ${MS_FFT} ms |
| **Status** | $ST_FFT |

\`\`\`
$OUT_FFT
\`\`\`

### \`bench_linalg\` — matrix multiply, inverse, eigenvalues

| | |
|---|---|
| **Wall time** | ${MS_LINALG} ms |
| **Status** | $ST_LINALG |

\`\`\`
$OUT_LINALG
\`\`\`

---

## Dependency Summary

| | |
|---|---|
| Direct dependencies (rustlab-cli) | $DIRECT_DEPS |
| Total transitive dependency lines | $TOTAL_DEPS |

\`\`\`
$DEP_TREE
\`\`\`

---

## Known Optimisation Opportunities

These were identified during the initial audit in \`perf/performance.md\`.
An AI agent reviewing this report should check whether each item is still
open, estimate its impact given the numbers above, and prioritise them.

| ID | Change | Expected saving | Status |
|----|--------|----------------|--------|
| OPT-1 | \`zip\`: \`default-features = false, features = ["deflate"]\` | ~300–500 KB | **open** |
| OPT-2 | \`ndarray\`: remove \`features = ["rayon"]\` (unused) | ~150–250 KB | **open** |
| OPT-3 | \`[profile.release]\` — \`lto = "thin"\`, \`codegen-units = 1\`, \`strip = "symbols"\` | ~400–800 KB + runtime | **open** |

---

<!-- AI_ANALYSIS

You are an AI assistant reviewing a rustlab performance report.
Produce a section headed "## AI Analysis" and append it to this file.

Work through these tasks in order:

1. BINARY SIZE
   - Current unstripped size: $BINARY_SIZE_H  stripped: $STRIPPED_SIZE_H
   - Are OPT-1, OPT-2, OPT-3 still open?  Apply whichever are still pending
     and re-measure, or estimate the new size if you cannot rebuild.
   - Flag any new dependencies added since the last report that have not
     been audited for unnecessary default features.

2. BENCHMARK TIMINGS
   Thresholds for concern (flag if exceeded):
   - bench_upfirdn total: > 500 ms
   - bench_fft total:     > 100 ms
   - bench_linalg total:  > 200 ms
   Current values: upfirdn=${MS_UPFIRDN}ms  fft=${MS_FFT}ms  linalg=${MS_LINALG}ms
   For each benchmark that exceeds its threshold, suggest a specific next
   profiling step (e.g. samply, flamegraph, criterion micro-benchmark).

3. DEPENDENCY GROWTH
   Total transitive dependency lines: $TOTAL_DEPS
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
HEADER
} > "$REPORT"

# ── console summary ───────────────────────────────────────────────────────────

echo ""
echo "Report written to perf/report.md"
echo ""
echo "  Binary (unstripped): $BINARY_SIZE_H"
echo "  Binary (stripped):   $STRIPPED_SIZE_H"
printf "  %-32s %d ms\n" "bench_upfirdn:" "$MS_UPFIRDN"
printf "  %-32s %d ms\n" "bench_fft:"     "$MS_FFT"
printf "  %-32s %d ms\n" "bench_linalg:"  "$MS_LINALG"
echo ""
echo "Run the AI analysis:"
echo "  claude --print 'Read perf/report.md and follow the AI_ANALYSIS instructions.'"
