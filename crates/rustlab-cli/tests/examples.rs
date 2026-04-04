/// Integration tests for non-interactive example scripts.
///
/// Each test runs `rustlab run <example>` in a temporary directory so that
/// any output files (*.svg, *.npy, *.csv, *.npz) are automatically cleaned
/// up when the TempDir is dropped.
///
/// Only examples that produce no interactive terminal UI (plot / stem /
/// plotdb / histogram) are included here; the rest require a real terminal
/// and are covered by `make examples`.
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Absolute path to the workspace root, derived from this crate's location.
fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR == .../crates/rustlab-cli
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap() // .../crates
        .parent().unwrap() // workspace root
        .to_path_buf()
}

/// Run one example script in a fresh temp directory.
/// Returns the process exit status.
fn run_example(name: &str) -> std::process::ExitStatus {
    let dir     = TempDir::new().expect("failed to create temp dir");
    let script  = workspace_root().join("examples").join(format!("{name}.r"));
    let bin     = env!("CARGO_BIN_EXE_rustlab");

    Command::new(bin)
        .args(["run", script.to_str().unwrap()])
        .current_dir(dir.path()) // output files land here and are auto-deleted
        .status()
        .unwrap_or_else(|e| panic!("failed to launch rustlab for example '{name}': {e}"))
    // dir is dropped here → temp directory deleted automatically
}

// ── Non-interactive examples ───────────────────────────────────────────────

#[test]
fn example_complex_basics() {
    let status = run_example("complex_basics");
    assert!(status.success(), "example 'complex_basics' exited with {status}");
}

#[test]
fn example_save_load() {
    let status = run_example("save_load");
    assert!(status.success(), "example 'save_load' exited with {status}");
}

#[test]
fn example_firpm() {
    let status = run_example("firpm");
    assert!(status.success(), "example 'firpm' exited with {status}");
}

#[test]
fn example_fixed_point() {
    let dir    = TempDir::new().expect("failed to create temp dir");
    let script = workspace_root().join("examples").join("fixed_point.r");
    let bin    = env!("CARGO_BIN_EXE_rustlab");

    let output = Command::new(bin)
        .args(["run", script.to_str().unwrap()])
        .current_dir(dir.path())
        .output()
        .expect("failed to launch rustlab for example 'fixed_point'");

    assert!(output.status.success(),
        "fixed_point exited with {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr));

    // Parse the 5 SNR values printed after each "SNR (dB):" label line.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let snr_values: Vec<f64> = lines
        .windows(2)
        .filter(|w| w[0].contains("SNR (dB)"))
        .map(|w| w[1].trim().parse::<f64>()
            .unwrap_or_else(|_| panic!("could not parse SNR value from: {:?}", w[1])))
        .collect();

    assert_eq!(snr_values.len(), 5,
        "expected 5 SNR values, got {}; stdout:\n{}", snr_values.len(), stdout);

    // SNR must strictly increase with bitwidth (8 → 10 → 12 → 14 → 16 bit).
    for (i, w) in snr_values.windows(2).enumerate() {
        assert!(w[1] > w[0],
            "SNR not monotonically increasing at step {i}: {:.1} → {:.1} dB\nAll: {snr_values:?}",
            w[0], w[1]);
    }

    // Loose absolute bounds: 8-bit ~30 dB, 16-bit ~74 dB.
    assert!(snr_values[0] > 20.0 && snr_values[0] < 45.0,
        "8-bit SNR out of expected range [20, 45] dB: {:.1}", snr_values[0]);
    assert!(snr_values[4] > 60.0,
        "16-bit SNR below expected floor of 60 dB: {:.1}", snr_values[4]);
}
