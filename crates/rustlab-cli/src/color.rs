//! ANSI color helpers for the REPL.
//!
//! Color is disabled when:
//! - The `NO_COLOR` environment variable is set (any value)
//! - stdout is not a TTY (piped output)

use std::io::IsTerminal;
use std::sync::OnceLock;

static COLOR_ENABLED: OnceLock<bool> = OnceLock::new();

/// Returns `true` when ANSI color codes should be emitted.
pub fn is_color_enabled() -> bool {
    *COLOR_ENABLED.get_or_init(|| {
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    })
}

// ── ANSI codes ───────────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BOLD_RED: &str = "\x1b[1;31m";
const BOLD_GREEN: &str = "\x1b[1;32m";
const BOLD_CYAN: &str = "\x1b[1;36m";
const BOLD_YELLOW: &str = "\x1b[1;33m";
// ── Wrapper functions ────────────────────────────────────────────────────────

fn wrap(code: &str, s: &str) -> String {
    if is_color_enabled() {
        format!("{code}{s}{RESET}")
    } else {
        s.to_string()
    }
}

pub fn green(s: &str) -> String { wrap(GREEN, s) }
pub fn yellow(s: &str) -> String { wrap(YELLOW, s) }
pub fn cyan(s: &str) -> String { wrap(CYAN, s) }
pub fn bold(s: &str) -> String { wrap(BOLD, s) }
pub fn dim(s: &str) -> String { wrap(DIM, s) }
pub fn bold_red(s: &str) -> String { wrap(BOLD_RED, s) }
pub fn bold_green(s: &str) -> String { wrap(BOLD_GREEN, s) }
pub fn bold_cyan(s: &str) -> String { wrap(BOLD_CYAN, s) }
pub fn bold_yellow(s: &str) -> String { wrap(BOLD_YELLOW, s) }
