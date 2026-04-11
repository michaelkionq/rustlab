# rustlab — Agent Reference

This file is the authoritative guide for AI coding tools working on this codebase.
Read it before making any changes.

---

## Project Overview

**rustlab** is a Rust CLI and scripting toolkit for matrix algebra and digital signal processing (DSP).
It provides a simple scripting language (`.r` files), an interactive REPL, and direct CLI commands for filter design, convolution, and plotting.

Key properties:
- All numeric types are complex by default (`Complex<f64>`)
- Scripting language uses 1-based indexing, `:` range syntax, and suppression with `;`
- Terminal plotting via `ratatui` + `crossterm` (braille-pixel charts, alternate screen)
- Five-crate Cargo workspace with strict no-cycle dependency order

---

## Repository Layout

```
rustlab/
├── Cargo.toml              # workspace root — shared deps, resolver = "2"
├── AGENTS.md               # this file
├── PLAN.md                 # original architecture plan
├── README.md               # user-facing documentation
├── llms.txt                # AI reference — pointers to docs files
├── crates/
│   ├── rustlab-core/       # primitive types and traits — no DSP, no plotting
│   ├── rustlab-dsp/        # DSP algorithms — depends on core only
│   ├── rustlab-plot/       # ratatui charts — depends on core only
│   ├── rustlab-script/     # .r language interpreter — depends on core, dsp, plot
│   └── rustlab-cli/        # binary `rustlab` — depends on all crates
├── dev/
│   └── plans/              # multi-phase development plans (see section below)
├── perf/                   # performance benchmarks and reports
├── examples/               # 19+ top-level scripts, plus subdirectories:
│   ├── controls/           # 14 control systems examples (tf, bode, lqr, ode, etc.)
│   ├── audio/              # real-time audio: filter, spectrum monitor, platform launchers
│   │   ├── filter.r          # FIR lowpass script used by all launchers
│   │   ├── passthrough.r     # minimal stdin→stdout loopback
│   │   ├── spectrum_monitor.r  # live two-panel terminal plot (waveform + FFT)
│   │   ├── spectrum_monitor.sh # platform-aware launcher (macOS/Linux/synthetic)
│   │   ├── macos.sh          # sox-based live audio pipeline (macOS)
│   │   ├── linux.sh          # arecord/aplay pipeline (Linux ALSA)
│   │   ├── wsl.sh            # PulseAudio / WSL2 pipeline
│   │   ├── tcp.sh            # socat/nc TCP streaming (cross-platform)
│   │   └── test_filter.sh    # CI-friendly end-to-end test (no mic/speakers)
│   ├── complex_basics.r, vectors.r, lowpass.r, bandpass.r, fft.r, ...
│   ├── firpm.r, upfirdn.r, fixed_point.r, ml_activations.r, ...
│   └── lambda.r, profiling.r, save_load.r, ...
└── docs/
    ├── examples.md         # annotated walkthroughs of each example script
    ├── functions.md        # full function reference with signatures and examples
    └── quickref.md         # concise capability index kept in sync with actual builtins
```

**Dependency order (no cycles):**
```
rustlab-core
    ↑           ↑
rustlab-dsp   rustlab-plot
    ↑           ↑
    └─────┬─────┘
    rustlab-script
          ↑
    rustlab-cli
```

---

## Performance Analysis

### Running benchmarks

```sh
make perf
```

This builds the release binary, times every script in `perf/`, measures
binary size, and writes `perf/report.md`.

### When an AI agent sees `perf/report.md`

If `perf/report.md` exists and was generated in the current session (or the
user asks for a performance review), the agent **must**:

1. Read `perf/report.md` in full.
2. Work through every task listed in the `<!-- AI_ANALYSIS -->` block inside
   that file.
3. Append a `## AI Analysis` section directly to `perf/report.md` with
   findings and a prioritised recommendation table.
4. If any **open** optimisation (OPT-1, OPT-2, OPT-3, …) can be applied
   without breaking tests, implement it, re-run `make perf`, and update the
   report with before/after numbers.

Do not skip the analysis or produce it only as a chat response — it must be
written into the report file so it is preserved across sessions.

---

## Development Plans

Multi-phase feature plans live in `dev/plans/`.  Each plan tracks current phase
and per-item status.  **Follow this protocol at the start and end of every
session that touches a plan:**

### Reading plans at session start

1. Check `dev/plans/` for any plan whose **Status** is not `complete`.
2. Read the active plan, identify the **Current phase** and which items in it
   are `not started` vs `in progress` vs `done`.
3. If the user has not already given direction, briefly surface the active plan:
   > "The controls plan is on **Phase 1** (Language Foundations).
   >  Would you like to continue with Phase 1, or work on something else?"

### Implementing a phase

- Work through every item in the phase top-to-bottom.
- After each item, mark it `done` in the plan file.
- On completion of the full phase:
  1. Update the plan: set the phase **Status** to `complete` and advance
     **Current phase** to the next phase.
  2. Run `cargo test --workspace` and confirm it passes.
  3. Ask the user: *"Phase N is complete.  Ready to start Phase N+1
     ([short description])?"*  Do not begin the next phase without an
     explicit yes.

### Plan file conventions

Each phase block contains a `**Status:**` line.  Valid values:
- `not started` — work has not begun
- `in progress` — partially implemented
- `complete` — all items done, tests pass

Update the top-level **Current phase** line and the per-phase **Status** line
together whenever a phase finishes.

---

## Active Plans

| Plan | File | Status |
|------|------|--------|
| Control Systems Toolbox | `dev/plans/controls.md` | Complete — all 6 phases |
| Controls Bootcamp Functions | `dev/plans/controls_bootcamp.md` | Complete — logspace, rk4, lyap, gram, care, dare, place, freqresp, svd |
| Lambda / Anonymous Functions | `dev/plans/lambda.md` | Complete — both phases (lambdas, arrayfun, feval) |
| Function Call Profiling | `dev/plans/profiling.md` | Complete — both phases (profile(), --profile flag) |
| Real-Time Audio Streaming | `dev/plans/audio_streaming.md` | Complete — all 3 phases (while loop, FirState, audio I/O) |
| Live Plot & Spectrum Monitor | `dev/plans/live_plot.md` | Complete — all 3 phases (LiveFigure, builtins, mag2db) |
| Sparse Vectors and Matrices | `dev/plans/sparse.md` | Complete — all 4 phases (types, conversion, arithmetic, solver/utilities) |

---

## Workflow Rules

These three rules apply to every task, no exceptions.

### 1. Plan first, implement second

Before writing any code for a non-trivial change, produce a written plan and present it to the user for review. The plan must cover:
- What will change and why
- Which files and crates are affected
- Any trade-offs or risks
- The test strategy for the new code

Do not begin implementation until the plan is explicitly approved.

### 2. Tests are required for new features

Every new DSP algorithm, builtin function, or scripting language feature must ship with at least one meaningful unit test. "Meaningful" means:
- It exercises a concrete, verifiable property (e.g. lowpass coefficients sum to 1, convolution with a delta is identity, `inv(A) * A ≈ I`)
- It would catch a regression if the implementation were broken
- It runs headlessly without a TTY (`cargo test --workspace`)

Add tests in the same PR/commit as the feature — never defer them. Good locations:
- `crates/rustlab-dsp/src/tests.rs` — DSP algorithms
- `crates/rustlab-script/src/tests.rs` — interpreter and builtins (use `run()` to evaluate snippets)
- `crates/rustlab-cli/tests/examples.rs` — integration / example scripts

### 3. Every new feature ships with docs and REPL help

Any commit that adds or changes a builtin function, scripting construct, or CLI feature **must** include all three of the following in the same commit — not as a follow-up:

1. **`docs/functions.md`** — add or update the function's section with its full signature, description, and at least one usage example.
2. **REPL `HelpEntry`** — add a `HelpEntry { name, brief, detail }` record in `crates/rustlab-cli/src/commands/repl.rs`.
3. **Category list** — add the function name to the appropriate category slice in `print_help_list()` in the same file.

A feature is not done until a user can type `help foo` in the REPL and get a useful answer. Do not treat documentation as optional cleanup.

### 4. Never commit or push without explicit approval

Do not run `git commit` or `git push` automatically, even when work is complete and all tests pass. Present a summary of what changed and wait for the user to explicitly say to commit and/or push.

### 5. Keep `docs/functions.md` current

`docs/functions.md` is the canonical scripting reference. It must be updated in the same commit as any change that affects it:

- **New builtin function** — add its signature, description, and example to the appropriate section.
- **New Value type** — document its fields and how to use it.
- **New language construct** — add syntax and example to the Language section.
- **New toolbox feature** (controls, DSP, etc.) — add it to the relevant toolbox section.

`llms.txt` at the repo root is a short pointer to the four main docs files (`docs/quickref.md`, `docs/functions.md`, `docs/examples.md`, `README.md`); it does not need content updates. Do not treat docs updates as optional cleanup.

### 6. Keep `docs/quickref.md` current

`docs/quickref.md` is the concise capability index used by AI agents to discover what rustlab can do. It must stay in sync with the actual registered builtins. Update it in the same commit as any change that affects it:

- **New builtin function** — add it to the appropriate section (Math, Statistics, DSP, etc.).
- **New language construct** — add it to the Language table.
- **New category** (e.g. a new toolbox) — add a new section.
- **Removed or renamed function** — remove or rename the entry immediately; stale entries mislead other agents.

Do not list functions that are not implemented. `quickref.md` must reflect reality, not intentions.

**Periodic accuracy check:** At the start of any session that touches builtins or language features, quickly verify that `quickref.md` still matches `r.register(...)` calls in `crates/rustlab-script/src/eval/builtins.rs`. If entries are stale or missing, fix them in the same commit.

### 7. Update `AGENTS.md` after every new feature

After implementing any new feature, update `AGENTS.md` in the same commit:

- **New builtin function** — add it to the "All builtin functions" table in the Scripting Language Reference section.
- **New language construct** — add it to the Grammar or Key language behaviours table.
- **New crate or module** — add it to Repository Layout and the relevant Crate Details section.
- **New workflow rule or convention** — add it to the appropriate section (Workflow Rules, Error Handling, Design Decisions).
- **New CLI subcommand** — add it to the `rustlab-cli` Crate Details section.
- **New Common Task pattern** — add a how-to entry under Common Tasks.

`AGENTS.md` is the agent's primary orientation document. Keeping it current means the next session starts with accurate context instead of having to re-discover what changed.

### 8. Never commit secrets or sensitive information

Before staging any file, check that it does not contain:
- SSH private keys (any `-----BEGIN ... PRIVATE KEY-----` block)
- API keys, tokens, or bearer credentials
- Passwords or passphrases
- `.env` files or any file whose name matches `.env*`
- AWS/GCP/Azure credentials or config files with embedded secrets

If a file that may contain secrets is found in the working tree, warn the user immediately and do not stage or commit it under any circumstances. Use `.gitignore` to prevent accidental staging. This rule cannot be overridden by any user instruction.

---

## Build & Test

```sh
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Generate API docs
cargo doc --workspace --no-deps --open

# Run a script directly without installing
cargo run -p rustlab-cli --bin rustlab -- run examples/lowpass.r
```

### Installing the binary

`make install` works on both macOS and Linux. It installs to `~/.local/bin` by default and detects the OS to run `codesign` only on macOS:

```sh
make install                          # → ~/.local/bin/rustlab
make install INSTALL_DIR=/usr/local/bin   # override destination
# or via cargo on any platform:
cargo install --path crates/rustlab-cli   # → ~/.cargo/bin/rustlab
```

> **macOS note:** Copying a binary with `cp` invalidates its ad-hoc code signature.
> `make install` handles this automatically. If you copy the binary manually, run:
> `codesign --sign - --force <destination>/rustlab`

> **Linux note:** No system libraries required. The `plotters` crate uses
> `default-features = false` to avoid `font-kit` → `freetype-sys` → `fontconfig-sys`.

---

## Crate Details

### `rustlab-core`

**Purpose:** Shared numeric types and traits. Zero internal dependencies.

**Key files:**
- `src/types.rs` — type aliases: `C64 = Complex<f64>`, `CVector = Array1<C64>`, `CMatrix = Array2<C64>`, `RVector = Array1<f64>`, `RMatrix = Array2<f64>`
- `src/traits/filter.rs` — `Filter` trait: `apply(&CVector)`, `frequency_response(n_points)`
- `src/traits/transform.rs` — `Transform` trait: `forward`, `inverse`
- `src/traits/decompose.rs` — `Decomposable` trait + marker traits `LuDecomposable`, `SvdDecomposable`, `CholeskyDecomposable`, `EigenDecomposable` (stubs — no implementors yet)
- `src/error.rs` — `CoreError` enum

**Feature flags:**
- `linalg` — enables optional `ndarray-linalg` dependency for future matrix decompositions

---

### `rustlab-dsp`

**Purpose:** DSP algorithms. Depends on `rustlab-core` only.

**Key files:**
- `src/window/mod.rs` — `WindowFunction` enum: `Rectangular`, `Hann`, `Hamming`, `Blackman`, `Kaiser { beta }`. Methods: `generate(length) -> RVector`, `from_str(s, beta)`
- `src/fir/design.rs` — `FirFilter` struct + `fir_lowpass`, `fir_highpass`, `fir_bandpass` (windowed-sinc method). `FirFilter` implements `Filter`.
- `src/iir/butterworth.rs` — `IirFilter { b: Vec<f64>, a: Vec<f64> }` + `butterworth_lowpass`, `butterworth_highpass` (bilinear transform, cascade of biquad sections). `IirFilter` implements `Filter`.
- `src/convolution.rs` — `convolve(x, h)` (direct O(nm)), `overlap_add(x, h, block_size)` (FFT-based)
- `src/error.rs` — `DspError` (wraps `CoreError`)

---

### `rustlab-plot`

**Purpose:** Terminal charts. Depends on `rustlab-core` only.

**Key files:**
- `src/ascii.rs` — `plot_real`, `plot_complex`, `stem_real`, and the shared `draw_subplots(f, subplots, rows, cols)` helper used by both `render_figure_terminal` and `LiveFigure::redraw`.
- `src/live.rs` — `LiveFigure` struct: `new(rows, cols)`, `update_panel(idx, x, y)`, `set_panel_labels(idx, title, xlabel, ylabel)`, `redraw()`. `Drop` impl restores the terminal.

**Behavior:** Static plot functions enter the ratatui alternate screen, draw a braille-pixel chart, wait for a keypress, then restore the terminal. `LiveFigure` keeps the alternate screen open across multiple `redraw()` calls and only restores on `Drop`. Neither should be called in non-TTY contexts (`render_figure_terminal` silently skips; `LiveFigure::new` returns `Err(PlotError::NotATty)`).

---

### `rustlab-script`

**Purpose:** Interpreter for `.r` script files and the REPL. Depends on core, dsp, and plot.

**Key files:**
- `src/lexer.rs` — hand-written lexer → `Vec<Spanned<Token>>`
- `src/parser.rs` — recursive-descent parser → `Vec<Stmt>`
- `src/ast.rs` — `Stmt` (Assign, Expr, FunctionDef, FieldAssign, Return), `Expr` (Number, Str, Var, BinOp, UnaryMinus, Call, Matrix, Range, Transpose, Field, Lambda, FuncHandle), `BinOp`
- `src/eval/mod.rs` — `Evaluator` struct: holds `env`, `builtins`, `user_fns`, `in_function`, `profiler: profile::Profiler`; public API: `run()`, `run_script()`, `enable_profiling()`, `has_profile_data()`, `take_profile()`
- `src/eval/value.rs` — `Value` enum: `Scalar(f64)`, `Complex(C64)`, `Vector(CVector)`, `Matrix(CMatrix)`, `Str(String)`, `Struct(HashMap<String,Value>)`, `Bool(bool)`, `Lambda { params, body, captured_env }`, `FuncHandle(String)`, `QFmt`, `FirState(Arc<Mutex<Vec<C64>>>)`, `AudioIn { sample_rate, frame_size }`, `AudioOut { sample_rate, frame_size }`, `LiveFigure(Arc<Mutex<Option<rustlab_plot::LiveFigure>>>)`, `All`, `None`
- `src/eval/builtins.rs` — `BuiltinRegistry`: `HashMap<String, BuiltinFn>` where `BuiltinFn = fn(Vec<Value>) -> Result<Value, ScriptError>`
- `src/eval/profile.rs` — `Profiler` struct (opt-in, zero overhead when disabled); `print_report()` prints table to stderr
- `src/lib.rs` — public entry points: `run(source)`, `run_profiled(source)`

**Pre-populated environment constants:** `j = Complex(0,1)`, `i = Complex(0,1)`, `pi = 3.14159…`, `e = 2.71828…`, `Inf = f64::INFINITY`, `NaN = f64::NAN`, `true = Bool(true)`, `false = Bool(false)`

**`BUILTIN_CONSTS`:** These constant names (`i`, `j`, `pi`, `e`, `Inf`, `NaN`, `true`, `false`) survive `clear_vars()` — they are re-inserted automatically so the REPL never loses them.

**How `Call` nodes are evaluated:** At eval time, if the name exists in `env` as a `Vector` or `Matrix`, it is treated as 1-based indexing — `end` is temporarily bound to the vector length. If the name holds a `Lambda`, it is called with its captured environment. Otherwise it is a `BuiltinRegistry` call.

**Lambda / anonymous functions:** `@(x, y) expr` creates a `Value::Lambda` that captures the current env by snapshot. `@name` creates a `Value::FuncHandle` that lazily resolves to a lambda clone (if `name` holds a lambda) or dispatches to a builtin/user function. `arrayfun(f, v)` maps any callable over a vector, returning a `Vector` (all scalar outputs) or a `Matrix` (all vector outputs of equal length). `feval("name", args...)` calls a function by string name.

**Profiling:** `profile(fn1, fn2)` inside a script enables selective tracking of named functions. `profile()` with no args tracks all calls. `profile_report()` prints a mid-script report to stderr. `--profile` CLI flag (on `rustlab run`) tracks all calls without modifying the script. `Profiler` uses a `higher_order_depth` counter so inner callbacks inside `arrayfun` or user functions are not recorded individually — only the outer call's total time is captured. Zero overhead when disabled.

**Adding a new builtin function:**
1. Write `fn builtin_foo(args: Vec<Value>) -> Result<Value, ScriptError>` in `src/eval/builtins.rs`
2. Add `r.register("foo", builtin_foo);` in `BuiltinRegistry::with_defaults()`
3. No parser or grammar changes required

---

### `rustlab-cli`

**Purpose:** Binary crate. Wires clap subcommands to the other crates.

**Key files:**
- `src/main.rs` — calls `Cli::parse().execute()`
- `src/cli.rs` — `Cli` struct with `Option<Commands>` (None → REPL)
- `src/commands/repl.rs` — interactive REPL using `rustyline`; persistent `Evaluator` across inputs
- `src/commands/run.rs` — reads a file, calls `rustlab_script::run`
- `src/commands/filter.rs` — `fir` and `iir` subcommands
- `src/commands/convolve.rs` — reads CSV signals, calls `convolve` or `overlap_add`
- `src/commands/window.rs` — generates window, prints values, optional `--plot`
- `src/commands/plot.rs` — reads CSV, dispatches to plot functions

**Default behaviour:** `rustlab` with no arguments starts the REPL.

---

## Scripting Language Reference

Scripts use the `.r` extension. Run with `rustlab run script.r` or enter statements interactively in the REPL.

### Grammar (informal)

```
program     = stmt*
stmt        = IDENT "=" range_expr [";"] "\n"              # assignment
            | IDENT "(" arglist ")" "=" range_expr [";"] "\n"  # indexed assignment
            | IDENT "." IDENT "=" range_expr [";"] "\n"    # struct field assignment
            | range_expr [";"] "\n"                         # expression
            | "function" [IDENT "="] IDENT "(" params ")"  # function definition
                stmt* "end"
            | "return" [";"] "\n"                          # early return (inside function)
            | "for" IDENT "=" range_expr "\n"              # for loop
                stmt* "end"
            | "while" range_expr "\n"                      # while loop
                stmt* "end"
            | "#" ... "\n"                                  # comment

range_expr  = expr (":" expr (":" expr)?)?     # a:b or a:step:b → Vector

expr        = term (("+"|"-") term)*
term        = factor (("*"|"/"|".*"|"./") factor)*
factor      = postfix (("^"|".^") factor)?     # right-associative
postfix     = primary ("'" | ".'" | "." IDENT ["(" arglist ")"] | "(" arglist ")")*
                # ' = conjugate transpose; .' = plain transpose
                # .field = struct access; .method(args) = method-call sugar
                # (args) after any non-Var expr = chained index: f(a)(i)

primary     = NUMBER | STRING | IDENT
            | IDENT "(" range_arglist ")"       # call or 1-based index
            | "[" range_row (";" range_row)* "]"
            | "(" range_expr ")"
            | "-" primary
            | "@" "(" params ")" expr           # anonymous function (lambda)
            | "@" IDENT                         # function handle
```

### Key language behaviours

| Feature | Syntax | Notes |
|---|---|---|
| Imaginary unit | `j` | Predefined constant `Complex(0,1)` |
| Complex number | `1.5 + j*2.0` | Standard arithmetic |
| Suppress output | `x = expr;` | Trailing `;` on any statement |
| Range | `1:10`, `0:0.5:2`, `10:-1:1` | Creates a real `Vector` |
| 1-based index | `v(3)`, `v(2:5)`, `v(end)` | `end` = `len(v)`; slice returns Vector |
| Indexed assign | `v(i) = val`, `M(r,c) = val` | Vectors auto-created/grown; matrices must exist |
| Chained index | `f(a,b)(i)` | Index return value of any call without a temp variable |
| For loop | `for i = 1:n ... end` | Iterates over range or vector; loop var stays in scope |
| While loop | `while cond ... end` | Repeats body while cond is truthy; cond may be Bool, Scalar (nonzero), or Complex |
| Lambda | `f = @(x) x^2` | Creates anonymous function; captures env by snapshot at creation |
| Function handle | `@sin`, `@myFn` | Reference to builtin or user-defined function |
| Higher-order | `arrayfun(@sin, v)` | Maps callable over vector; scalar outputs → Vector, vector outputs → Matrix |
| Dynamic call | `feval("name", args...)` | Call function by string name |
| Profile | `profile(fn1, fn2)` / `profile()` | Track named functions (or all); `profile_report()` prints mid-script |
| Concatenation | `[v1, v2]` | Vectors inside `[...]` are flattened |
| Transpose | `v'` | Conjugate transpose |
| Element-wise | `.*` `./` `.^` | Always element-wise on vectors/matrices |
| Matrix literal | `[1,2; 3,4]` | `;` separates rows |
| Sparse types | `SparseVector`, `SparseMatrix` | COO format; 0-based internal, 1-based in script; auto-promote to dense in binops |

### All builtin functions

| Function | Signature | Returns |
|---|---|---|
| `abs` | `abs(x)` | Magnitude (element-wise) |
| `angle` | `angle(x)` | Phase in radians (element-wise) |
| `real` | `real(x)` | Real part |
| `imag` | `imag(x)` | Imaginary part |
| `cos` | `cos(x)` | Cosine (element-wise) |
| `sin` | `sin(x)` | Sine (element-wise) |
| `sqrt` | `sqrt(x)` | Square root (element-wise) |
| `exp` | `exp(x)` | e^x (element-wise) |
| `log` | `log(x)` | Natural log (element-wise) |
| `zeros` | `zeros(n)` / `zeros(n, m)` | Complex zero vector of length n, or n×m zero matrix |
| `ones` | `ones(n)` / `ones(n, m)` | Complex ones vector of length n, or n×m ones matrix |
| `linspace` | `linspace(start, stop, n)` | Real vector of n points |
| `len` | `len(v)` | Number of elements |
| `length` | `length(v)` | Alias for `len` |
| `numel` | `numel(x)` | Total elements (rows×cols for matrices) |
| `size` | `size(x)` | `[rows, cols]` as a Vector |
| `print` | `print(x)` | Print to stdout |
| `plot` | `plot(x)` | Terminal line chart (blocks until keypress) |
| `stem` | `stem(x)` | Terminal stem chart (blocks until keypress) |
| `window` | `window(name, n)` | Real window vector |
| `fir_lowpass` | `fir_lowpass(taps, cutoff_hz, sr, window)` | FIR coefficient Vector |
| `fir_highpass` | `fir_highpass(taps, cutoff_hz, sr, window)` | FIR coefficient Vector |
| `fir_bandpass` | `fir_bandpass(taps, low_hz, high_hz, sr, window)` | FIR coefficient Vector |
| `butterworth_lowpass` | `butterworth_lowpass(order, cutoff_hz, sr)` | IIR b-coefficient Vector |
| `butterworth_highpass` | `butterworth_highpass(order, cutoff_hz, sr)` | IIR b-coefficient Vector |
| `median` | `median(v)` | Median of real parts; scalar for odd length, average of two middles for even |
| `convolve` | `convolve(x, h)` | Convolved Vector (length = len(x)+len(h)-1) |
| `filtfilt` | `filtfilt(b, a, x)` | Zero-phase forward-backward filter; uses odd-reflection signal extension + steady-state IC (matches Octave); use `a=[1]` for FIR |
| `prod` | `prod(v)` | Product of all elements (Vector or Matrix); returns Scalar |
| `firpmq` | `firpmq(n_taps, bands, desired [, weights [, bits [, n_iter]]])` | Integer-coefficient Parks-McClellan; defaults bits=16, n_iter=8. Returns integer-valued taps. For unit-gain passband, `sum(h_int)` equals the scale factor — use `freqz(h_int / sum(h_int), ...)` to verify. |
| `arrayfun` | `arrayfun(f, v)` | Apply callable `f` to each element of `v`; scalar outputs → Vector, vector outputs → Matrix |
| `feval` | `feval("name", args...)` | Call function by string name |
| `profile` | `profile(fn1, ...)` / `profile()` | Enable selective (or all-function) call profiling in-script |
| `profile_report` | `profile_report()` | Print profiling table to stderr immediately |
| `logspace` | `logspace(a, b, n)` | n log-spaced points from 10^a to 10^b |
| `rk4` | `rk4(f, x0, t)` | Fixed-step 4th-order Runge-Kutta; f(x,t)→x_dot; returns vector (1-state) or n×T matrix |
| `lyap` | `lyap(A, Q)` | Solve A*X + X*A' + Q = 0 (Kronecker vectorization; n≤50 practical) |
| `gram` | `gram(A, B, "c")` / `gram(A, C, "o")` | Controllability or observability Gramian via lyap |
| `care` | `care(A, B, Q, R)` | Continuous Algebraic Riccati Equation → P |
| `dare` | `dare(A, B, Q, R)` | Discrete Algebraic Riccati Equation → P |
| `place` | `place(A, B, poles)` | Ackermann pole placement (SISO only) → gain vector K |
| `freqresp` | `freqresp(A, B, C, D, w)` | H(jω) at each ω; SISO → complex vector, MIMO → complex matrix |
| `svd` | `svd(A)` | SVD via symmetric eigendecomposition of A'A (real); returns Tuple [U, sigma_vector, V] where sigma is sorted descending |
| `state_init` | `state_init(n)` | Allocate FirState history buffer of length n; returns `Value::FirState` |
| `filter_stream` | `filter_stream(frame, h, state)` | Overlap-save FIR frame filter; returns Tuple `[y, state]`; history updated in-place |
| `audio_in` | `audio_in(sr, frame_size)` | Create `Value::AudioIn` descriptor (metadata only; no I/O) |
| `audio_out` | `audio_out(sr, frame_size)` | Create `Value::AudioOut` descriptor (metadata only; no I/O) |
| `audio_read` | `audio_read(src)` | Read one frame of f32 LE samples from stdin; raises `ScriptError::AudioEof` on clean EOF |
| `audio_write` | `audio_write(dst, y)` | Write real parts of frame as f32 LE to stdout; flushes after each call |
| `figure_live` | `figure_live(rows, cols)` | Open persistent live terminal plot; returns `Value::LiveFigure`; errors if not a tty |
| `plot_update` | `plot_update(fig, panel, y)` / `plot_update(fig, panel, x, y)` | Replace panel data (1-based panel); no immediate redraw |
| `figure_draw` | `figure_draw(fig)` | Flush all panels to terminal in one atomic refresh |
| `figure_close` | `figure_close(fig)` | Drop `LiveFigure`, restoring terminal; also fires via `Drop` on script exit |
| `mag2db` | `mag2db(X)` | 20·log10(|X|) element-wise, floored at −200 dB (1e-10 floor) |
| `sparse` | `sparse(I, J, V, m, n)` / `sparse(A)` | Build sparse matrix from COO triples (1-based), or convert dense→sparse |
| `sparsevec` | `sparsevec(I, V, n)` | Build sparse vector of length n from 1-based indices and values |
| `speye` | `speye(n)` | n×n sparse identity matrix |
| `spzeros` | `spzeros(m, n)` | m×n all-zero sparse matrix |
| `full` | `full(S)` | Convert sparse to dense; identity for dense inputs |
| `nnz` | `nnz(S)` | Number of stored non-zero entries; numel for dense |
| `issparse` | `issparse(x)` | 1 if sparse, 0 otherwise |
| `nonzeros` | `nonzeros(S)` | Vector of non-zero values in storage order |
| `find` | `find(S)` | `[I,J,V]` tuple for sparse matrix, `[I,V]` for sparse vector (1-based) |
| `spsolve` | `spsolve(A, b)` | Solve A×x = b where A is sparse (converts to dense internally) |
| `spdiags` | `spdiags(V, D, m, n)` | Build sparse matrix from diagonals; D=0 main, >0 super, <0 sub |
| `sprand` | `sprand(m, n, density)` | Random sparse matrix with ~density×m×n non-zeros, values in [0,1) |

Window names: `"hann"`, `"hamming"`, `"blackman"`, `"rectangular"`, `"kaiser"`

---

## Common Tasks

### Add a new DSP algorithm

1. Implement the function in `crates/rustlab-dsp/src/` (create a new module if needed)
2. Implement the `Filter` trait if appropriate
3. Export from `crates/rustlab-dsp/src/lib.rs`
4. Add a builtin wrapper in `crates/rustlab-script/src/eval/builtins.rs` and register it in `with_defaults()`
5. Add a CLI subcommand in `crates/rustlab-cli/src/commands/` if useful from the command line

### Add a new builtin function

1. In `crates/rustlab-script/src/eval/builtins.rs`, write:
   ```rust
   fn builtin_foo(args: Vec<Value>) -> Result<Value, ScriptError> {
       check_args("foo", &args, 1)?;
       // ... extract args with .to_scalar()/.to_cvector()/.to_str()/.to_usize()
       Ok(Value::Scalar(...))
   }
   ```
2. Register: `r.register("foo", builtin_foo);` in `with_defaults()`
3. No grammar changes needed
4. Add a `HelpEntry` in `crates/rustlab-cli/src/commands/repl.rs` and add the name to the appropriate category in `print_help_list()` — required, not optional (see Workflow Rule 3)
5. Add the function to `docs/functions.md` with its signature, description, and an example (see Workflow Rule 5)
6. Write at least one unit test in `crates/rustlab-script/src/tests.rs` (see Workflow Rule 2)

### Add a new `Value` type

1. Add variant to `Value` enum in `src/eval/value.rs`
2. Add `negate`, `binop`, `Display` match arms
3. Add `to_*` conversion method if needed
4. Update `from_matrix_rows` if the type can appear in `[...]` literals

### Add matrix decompositions (future)

1. Create `crates/rustlab-linalg/` depending on `rustlab-core` with `linalg` feature
2. Implement `Decomposable` + the appropriate marker trait on `CMatrix`
3. Enable feature in workspace: `rustlab-core = { ..., features = ["linalg"] }`

---

## Design Decisions

| Decision | Rationale |
|---|---|
| All numbers are `Complex<f64>` | Avoids type promotion complexity; real signals just have `im = 0` |
| `j` is a constant not a syntax token | Keeps the lexer simple; `j*x` works naturally through arithmetic |
| 1-based indexing | Consistent with signal processing convention |
| Trailing `;` suppresses output | Familiar to anyone who has used a scientific computing language |
| `BuiltinRegistry` is a `HashMap` | Adding a function never requires touching the parser or grammar |
| `Decomposable` stubs exist now | Ensures the trait boundary is defined before any implementors are written |
| `ratatui` for plotting | Braille-pixel rendering in the terminal; alternate screen leaves no scrollback artifacts |
| `rustyline` for REPL | Provides readline history and line editing with minimal code |
| No `todo!()` stubs in production code | All implemented paths are complete; unimplemented paths return `CoreError::NotImplemented` |

---

## Error Handling Conventions

- `rustlab-core` → `CoreError`
- `rustlab-dsp` → `DspError` (wraps `CoreError` via `#[from]`)
- `rustlab-plot` → `PlotError`
- `rustlab-script` → `ScriptError` (wraps `CoreError`, `DspError`, `PlotError` via `#[from]`)
- `rustlab-cli` → `anyhow::Error` (converts all library errors at the boundary)

Use `?` to propagate. Do not panic except in `unreachable!()` for truly impossible arms.

**Special case — `ScriptError::AudioEof`:** Raised by `audio_read` when stdin closes cleanly mid-frame (the upstream producer finished). `rustlab-cli/src/commands/run.rs` intercepts this variant and maps it to `Ok(())` (exit code 0, no error message). It is never printed to the user — it is the normal end-of-stream signal for streaming pipelines.

---

## How to Add Tests

Tests are **required** for every new feature (see Workflow Rules above). Run the full suite with:

```sh
cargo test --workspace
```

### DSP algorithm tests — `crates/rustlab-dsp/src/tests.rs`

Test concrete mathematical properties:

```rust
#[test]
fn lowpass_coefficients_sum_to_one() {
    // A lowpass FIR with rectangular window has DC gain = 1
    let f = fir_lowpass(31, 0.25 * 44100.0, 44100.0, WindowFunction::Rectangular).unwrap();
    let sum: f64 = f.coefficients.iter().map(|c| c.re).sum();
    assert!((sum - 1.0).abs() < 1e-6, "DC gain was {sum}");
}

#[test]
fn convolution_with_delta_is_identity() {
    let x = Array1::from_vec(vec![1.0, 2.0, 3.0]);
    let delta = Array1::from_vec(vec![1.0]);
    let y = convolve(&x, &delta);
    assert_eq!(y.len(), x.len());
    for (a, b) in x.iter().zip(y.iter()) { assert!((a - b).abs() < 1e-12); }
}
```

### Interpreter / builtin tests — `crates/rustlab-script/src/tests.rs`

Use `run()` to evaluate snippets and inspect the returned environment:

```rust
#[test]
fn inv_times_a_is_identity() {
    let src = "A = [1,2;3,4]; B = inv(A) * A";
    let mut ev = Evaluator::new();
    ev.run(src).unwrap();
    // B should be approximately the 2×2 identity
    if let Value::Matrix(m) = ev.get("B").unwrap() {
        assert!((m[[0,0]].re - 1.0).abs() < 1e-10);
        assert!((m[[0,1]].re).abs() < 1e-10);
    } else { panic!("expected Matrix"); }
}
```

### Integration tests — `crates/rustlab-cli/tests/examples.rs`

Run example scripts and assert they exit cleanly:

```rust
#[test]
fn example_lowpass_runs() {
    let status = Command::new(env!("CARGO_BIN_EXE_rustlab"))
        .args(["run", "examples/lowpass.r"])
        .status().unwrap();
    assert!(status.success());
}
```
