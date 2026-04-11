use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("lex error at line {line}: {msg}")]
    Lex { line: usize, msg: String },
    #[error("parse error at line {line}: {msg}")]
    Parse { line: usize, msg: String },
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("type error: {0}")]
    Type(String),
    #[error("undefined variable: {0}")]
    Undefined(String),
    #[error("undefined function: {0}")]
    UndefinedFn(String),
    #[error("wrong number of arguments for {name}: expected {expected}, got {got}")]
    ArgCount { name: String, expected: usize, got: usize },
    #[error("wrong number of arguments for {name}: expected {min}..{max}, got {got}")]
    ArgCountRange { name: String, min: usize, max: usize, got: usize },
    #[error(transparent)]
    Dsp(#[from] rustlab_dsp::error::DspError),
    #[error(transparent)]
    Core(#[from] rustlab_core::CoreError),
    #[error(transparent)]
    Plot(#[from] rustlab_plot::PlotError),
    /// Internal signal: `return` statement in a function body.  Never shown to users.
    #[error("return")]
    EarlyReturn,
    /// stdin closed while audio_read was waiting for a full frame.
    /// Treated as a clean exit by the CLI (exit code 0, no error message).
    #[error("stdin closed")]
    AudioEof,
    /// User pressed Ctrl-C or 'q' while a live figure was active.
    /// Treated as a clean exit by the CLI (exit code 0, no error message).
    #[error("interrupted")]
    Interrupted,
}
