use std::path::PathBuf;
use clap::{Args, Subcommand, ValueEnum};
use anyhow::Result;
use rustlab_plot::Theme;

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
pub enum NotebookCommands {
    /// Render a notebook (or directory of notebooks) to HTML, LaTeX, or PDF
    Render(RenderArgs),
}

#[derive(Args)]
pub struct RenderArgs {
    /// Input .md file or directory of .md files
    input: PathBuf,
    /// Output file or directory
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Output format: html (default), latex, pdf
    #[arg(short, long, value_enum, default_value = "html")]
    format: CliFormat,
    /// Color theme: dark (default), light
    #[arg(short, long, value_enum, default_value = "dark")]
    theme: CliTheme,
}

pub fn execute(cmd: NotebookCommands) -> Result<()> {
    match cmd {
        NotebookCommands::Render(args) => {
            let theme = match args.theme {
                CliTheme::Dark => Theme::Dark,
                CliTheme::Light => Theme::Light,
            };
            let colors = theme.colors();
            let format = match args.format {
                CliFormat::Html => rustlab_notebook::Format::Html,
                CliFormat::Latex => rustlab_notebook::Format::Latex,
                CliFormat::Pdf => rustlab_notebook::Format::Pdf,
            };
            if args.input.is_dir() {
                rustlab_notebook::cmd_render_dir(args.input, args.output, format, colors);
            } else {
                rustlab_notebook::cmd_render(args.input, args.output, format, colors);
            }
            Ok(())
        }
    }
}
