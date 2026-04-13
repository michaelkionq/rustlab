use rustlab_script::Evaluator;
use rustlab_plot::{FIGURE, FigureState};
use crate::parse::Block;

/// A rendered block ready for HTML output.
#[derive(Debug)]
pub enum Rendered {
    /// Markdown prose (raw markdown text, not yet converted to HTML).
    Markdown(String),
    /// An executed code block with its results.
    Code {
        source: String,
        text_output: String,
        error: Option<String>,
        figure: Option<FigureState>,
        /// If true, source code should be hidden in rendered output.
        hidden: bool,
    },
}

/// Execute a parsed notebook, returning rendered blocks.
///
/// Code blocks run in sequence through a shared evaluator (variables
/// persist across blocks). After each code block, the current figure
/// is captured if it has any series data, and any text output (from
/// assignments, `disp()`, `print()`, etc.) is captured via the
/// evaluator's output buffer.
pub fn execute_notebook(blocks: &[Block]) -> Vec<Rendered> {
    let mut ev = Evaluator::new();
    let mut rendered = Vec::with_capacity(blocks.len());

    for block in blocks {
        match block {
            Block::Markdown(text) => {
                rendered.push(Rendered::Markdown(text.clone()));
            }
            Block::Code { source, hidden } => {
                // Reset figure before each code block so we only capture
                // what this block produces — unless hold is on, in which
                // case we preserve the figure state for multi-block overlays.
                let hold_active = FIGURE.with(|fig| fig.borrow().hold);
                if !hold_active {
                    FIGURE.with(|fig| fig.borrow_mut().reset());
                }

                // Capture text output during execution
                rustlab_script::start_capture();
                let error = run_code_block(&mut ev, source);
                let text_output = rustlab_script::stop_capture();

                // Capture figure if it has data
                let figure = FIGURE.with(|fig| {
                    let f = fig.borrow().clone();
                    if f.subplots.iter().any(|s| !s.series.is_empty()) {
                        Some(f)
                    } else {
                        None
                    }
                });

                rendered.push(Rendered::Code {
                    source: source.clone(),
                    text_output,
                    error,
                    figure,
                    hidden: *hidden,
                });
            }
        }
    }

    rendered
}

/// Run a code block through the evaluator. Returns `Some(error_message)` on failure.
fn run_code_block(ev: &mut Evaluator, source: &str) -> Option<String> {
    let tokens = match rustlab_script::lexer::tokenize(source) {
        Ok(t) => t,
        Err(e) => return Some(format!("{e}")),
    };
    let stmts = match rustlab_script::parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => return Some(format!("{e}")),
    };
    for stmt in &stmts {
        if let Err(e) = ev.exec_stmt(stmt) {
            return Some(format!("{e}"));
        }
    }
    None
}
