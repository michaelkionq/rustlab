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
├── crates/
│   ├── rustlab-core/       # primitive types and traits — no DSP, no plotting
│   ├── rustlab-dsp/        # DSP algorithms — depends on core only
│   ├── rustlab-plot/       # ratatui charts — depends on core only
│   ├── rustlab-script/     # .r language interpreter — depends on core, dsp, plot
│   └── rustlab-cli/        # binary `rustlab` — depends on all crates
├── dev/
│   └── plans/              # multi-phase development plans (see section below)
│       └── controls.md     # control systems toolbox plan
├── examples/
│   ├── complex_basics.r
│   ├── controls/
│   │   └── classical_control.r
│   ├── lowpass.r
│   ├── bandpass.r
│   └── vectors.r
└── docs/
    └── examples.md         # annotated walkthroughs of each example script
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

| Plan | File | Current Phase | Status |
|------|------|--------------|--------|
| Control Systems Toolbox | `dev/plans/controls.md` | Phase 5 — Optimal Control | Phases 1–4 complete |

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

`llms.txt` at the repo root is a short pointer to `docs/functions.md`; it does not need content updates. Do not treat docs updates as optional cleanup.

### 6. Never commit secrets or sensitive information

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
- `src/ascii.rs` — `plot_real(&RVector, title)`, `plot_complex(&CVector, title)`, `stem_real(&RVector, title)`

**Behavior:** All three functions enter the ratatui alternate screen, draw a braille-pixel chart, wait for a keypress, then restore the terminal. They are blocking and interactive — do not call them in non-TTY contexts.

---

### `rustlab-script`

**Purpose:** Interpreter for `.r` script files and the REPL. Depends on core, dsp, and plot.

**Key files:**
- `src/lexer.rs` — hand-written lexer → `Vec<Spanned<Token>>`
- `src/parser.rs` — recursive-descent parser → `Vec<Stmt>`
- `src/ast.rs` — `Stmt` (Assign, Expr, FunctionDef, FieldAssign, Return), `Expr` (Number, Str, Var, BinOp, UnaryMinus, Call, Matrix, Range, Transpose, Field), `BinOp`
- `src/eval/mod.rs` — `Evaluator` struct: holds `env: HashMap<String, Value>`, `builtins: BuiltinRegistry`, `user_fns: HashMap<String, UserFn>`, `in_function: bool`
- `src/eval/value.rs` — `Value` enum: `Scalar(f64)`, `Complex(C64)`, `Vector(CVector)`, `Matrix(CMatrix)`, `Str(String)`, `Struct(HashMap<String,Value>)`, `Bool(bool)`, `QFmt`, `All`, `None`
- `src/eval/builtins.rs` — `BuiltinRegistry`: `HashMap<String, BuiltinFn>` where `BuiltinFn = fn(Vec<Value>) -> Result<Value, ScriptError>`
- `src/lib.rs` — public entry point: `run(source: &str)`, `Evaluator`

**Pre-populated environment constants:** `j = Complex(0,1)`, `pi = 3.14159…`, `e = 2.71828…`

**How `Call` nodes are evaluated:** At eval time, if the name exists in `env` as a `Vector` or `Matrix`, it is treated as 1-based indexing — `end` is temporarily bound to the vector length. Otherwise it is a `BuiltinRegistry` call.

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
| Concatenation | `[v1, v2]` | Vectors inside `[...]` are flattened |
| Transpose | `v'` | Conjugate transpose |
| Element-wise | `.*` `./` `.^` | Always element-wise on vectors/matrices |
| Matrix literal | `[1,2; 3,4]` | `;` separates rows |

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
| `zeros` | `zeros(n)` | Complex zero vector of length n |
| `ones` | `ones(n)` | Complex ones vector of length n |
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
| `convolve` | `convolve(x, h)` | Convolved Vector (length = len(x)+len(h)-1) |

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
