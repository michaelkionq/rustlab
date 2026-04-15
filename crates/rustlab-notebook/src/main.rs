use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use rustlab_plot::Theme;

#[derive(Parser)]
#[command(
    name = "rustlab-notebook",
    version = env!("CARGO_PKG_VERSION"),
    about = "Render Markdown notebooks with rustlab code blocks",
    long_about = "Render Markdown notebooks with rustlab code blocks.\n\n\
        Executes ```rustlab fenced code blocks through the evaluator, captures\n\
        text output and plots, and produces self-contained HTML, LaTeX, or PDF.\n\
        Supports template interpolation (${expr}), KaTeX math, syntax highlighting,\n\
        and multi-notebook directory rendering with index generation."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, ValueEnum)]
enum CliFormat {
    Html,
    Latex,
    Pdf,
}

#[derive(Clone, ValueEnum)]
enum CliTheme {
    Dark,
    Light,
}

#[derive(Subcommand)]
enum Command {
    /// Render a notebook (or directory of notebooks) to HTML, LaTeX, or PDF
    #[command(
        long_about = "Render a notebook (or directory of notebooks) to HTML, LaTeX, or PDF.\n\n\
            Examples:\n  \
            rustlab-notebook render analysis.md              # → analysis.html\n  \
            rustlab-notebook render analysis.md -f pdf       # → analysis.pdf\n  \
            rustlab-notebook render analysis.md -f latex     # → analysis.tex\n  \
            rustlab-notebook render analysis.md -o out.html  # custom output path\n  \
            rustlab-notebook render notebooks/               # render all .md → .html + index.html\n  \
            rustlab-notebook render notebooks/ -f pdf        # render all .md → .pdf\n  \
            rustlab-notebook render analysis.md -t light     # light theme\n\n\
            HTML output is self-contained with interactive Plotly charts and KaTeX math.\n\
            PDF output requires pdflatex or tectonic installed."
    )]
    Render {
        /// Input .md file or directory of .md files
        input: PathBuf,
        /// Output file or directory (default: <input_stem>.<ext> or same directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format: html (default), latex, pdf
        #[arg(short, long, value_enum, default_value = "html")]
        format: CliFormat,
        /// Color theme: dark (default), light
        #[arg(short, long, value_enum, default_value = "dark")]
        theme: CliTheme,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { input, output, format, theme } => {
            let theme = match theme {
                CliTheme::Dark => Theme::Dark,
                CliTheme::Light => Theme::Light,
            };
            let colors = theme.colors();
            let format = match format {
                CliFormat::Html => rustlab_notebook::Format::Html,
                CliFormat::Latex => rustlab_notebook::Format::Latex,
                CliFormat::Pdf => rustlab_notebook::Format::Pdf,
            };
            if input.is_dir() {
                rustlab_notebook::cmd_render_dir(input, output, format, colors);
            } else {
                rustlab_notebook::cmd_render(input, output, format, colors);
            }
        }
    }
}
