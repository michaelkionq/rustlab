use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlotError {
    #[error("terminal error: {0}")]
    Terminal(String),
    #[error("empty data")]
    EmptyData,
    #[error(transparent)]
    Core(#[from] rustlab_core::CoreError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file output error: {0}")]
    FileOutput(String),
    #[error("figure_live requires a real terminal (stdout is not a tty)")]
    NotATty,
}
