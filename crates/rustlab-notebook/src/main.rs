mod parse;
mod execute;
mod render;

use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustlab-notebook", about = "Render Markdown notebooks with rustlab code blocks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render a notebook to HTML
    Render {
        /// Input .md file
        input: PathBuf,
        /// Output .html file (default: <input_stem>.html)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { input, output } => cmd_render(input, output),
    }
}

fn cmd_render(input: PathBuf, output: Option<PathBuf>) {
    // Read input
    let source = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {e}", input.display());
            std::process::exit(1);
        }
    };

    // Set working directory to the notebook's directory so relative paths
    // in code blocks (load, toml_read, etc.) resolve correctly.
    if let Some(dir) = input.parent() {
        if dir.as_os_str().len() > 0 {
            let _ = std::env::set_current_dir(dir);
        }
    }

    // Derive title from first # heading or filename
    let title = extract_title(&source, &input);

    // Parse → execute → render
    let blocks = parse::parse_notebook(&source);
    let rendered = execute::execute_notebook(&blocks);
    let html = render::render_html(&title, &rendered);

    // Determine output path
    let out_path = output.unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default();
        PathBuf::from(format!("{}.html", stem.to_string_lossy()))
    });

    // Write output
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error: cannot create directory {}: {e}", parent.display());
                std::process::exit(1);
            }
        }
    }
    if let Err(e) = std::fs::write(&out_path, html) {
        eprintln!("error: cannot write {}: {e}", out_path.display());
        std::process::exit(1);
    }

    // Count results
    let n_code = rendered.iter().filter(|b| matches!(b, execute::Rendered::Code { .. })).count();
    let n_plots = rendered.iter().filter(|b| matches!(b, execute::Rendered::Code { figure: Some(_), .. })).count();
    let n_errors = rendered.iter().filter(|b| matches!(b, execute::Rendered::Code { error: Some(_), .. })).count();

    print!("Rendered {} → {} ({} code blocks, {} plots",
        input.display(), out_path.display(), n_code, n_plots);
    if n_errors > 0 {
        print!(", {} errors", n_errors);
    }
    println!(")");
}

/// Extract a title from the first `# ...` heading, or fall back to the filename.
fn extract_title(source: &str, path: &PathBuf) -> String {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            return trimmed[2..].trim().to_string();
        }
    }
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
