//! Output capture for the evaluator.
//!
//! By default, evaluator output (`ans =`, `disp()`, `print()`, etc.) goes to
//! stdout via `print!`/`println!`. When capture is active, output is appended
//! to a thread-local buffer instead. This lets `rustlab-notebook` intercept
//! text output from code blocks without touching stdout.

use std::cell::RefCell;

thread_local! {
    static CAPTURE_BUFFER: RefCell<Option<String>> = RefCell::new(None);
}

/// Begin capturing evaluator output. Any previous capture is discarded.
pub fn start_capture() {
    CAPTURE_BUFFER.with(|b| *b.borrow_mut() = Some(String::new()));
}

/// Stop capturing and return the captured output. Returns empty string if
/// capture was not active.
pub fn stop_capture() -> String {
    CAPTURE_BUFFER.with(|b| b.borrow_mut().take().unwrap_or_default())
}

/// Returns true if output capture is currently active.
pub fn capturing() -> bool {
    CAPTURE_BUFFER.with(|b| b.borrow().is_some())
}

/// Print a string. If capture is active, append to the buffer; otherwise
/// print to stdout.
pub fn script_print(s: &str) {
    CAPTURE_BUFFER.with(|b| {
        let mut b = b.borrow_mut();
        if let Some(buf) = b.as_mut() {
            buf.push_str(s);
        } else {
            print!("{}", s);
        }
    });
}

/// Print a string followed by a newline. If capture is active, append to
/// the buffer; otherwise println to stdout.
pub fn script_println(s: &str) {
    CAPTURE_BUFFER.with(|b| {
        let mut b = b.borrow_mut();
        if let Some(buf) = b.as_mut() {
            buf.push_str(s);
            buf.push('\n');
        } else {
            println!("{}", s);
        }
    });
}
