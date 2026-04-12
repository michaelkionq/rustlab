mod parse;
mod execute;
mod render;
mod render_latex;

use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "rustlab-notebook", about = "Render Markdown notebooks with rustlab code blocks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, ValueEnum)]
enum Format {
    Html,
    Latex,
    Pdf,
}

#[derive(Subcommand)]
enum Command {
    /// Render a notebook to HTML, LaTeX, or PDF
    Render {
        /// Input .md file
        input: PathBuf,
        /// Output file (default: <input_stem>.<ext>)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format
        #[arg(short, long, value_enum, default_value = "html")]
        format: Format,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { input, output, format } => cmd_render(input, output, format),
    }
}

fn cmd_render(input: PathBuf, output: Option<PathBuf>, format: Format) {
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

    // Parse → execute
    let blocks = parse::parse_notebook(&source);
    let rendered = execute::execute_notebook(&blocks);

    // Determine default extension and output path
    let ext = match format {
        Format::Html => "html",
        Format::Latex => "tex",
        Format::Pdf => "pdf",
    };
    let out_path = output.unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default();
        PathBuf::from(format!("{}.{ext}", stem.to_string_lossy()))
    });

    // Render to chosen format
    match format {
        Format::Html => {
            let html = render::render_html(&title, &rendered);
            write_output(&out_path, html.as_bytes());
        }
        Format::Latex => {
            let plot_dir = plot_dir_for(&out_path);
            let tex = render_latex::render_latex(&title, &rendered, &plot_dir);
            write_output(&out_path, tex.as_bytes());
        }
        Format::Pdf => {
            // Generate .tex first, then compile with pdflatex
            let tex_path = out_path.with_extension("tex");
            let plot_dir = plot_dir_for(&tex_path);
            let tex = render_latex::render_latex(&title, &rendered, &plot_dir);
            write_output(&tex_path, tex.as_bytes());

            compile_pdf(&tex_path, &out_path);
        }
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

/// Derive the plot image directory from the .tex output path.
/// e.g. `analysis.tex` → `analysis_plots/`
fn plot_dir_for(tex_path: &PathBuf) -> PathBuf {
    let stem = tex_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = tex_path.parent().unwrap_or(std::path::Path::new("."));
    parent.join(format!("{stem}_plots"))
}

/// Write bytes to a file, creating parent directories as needed.
fn write_output(path: &PathBuf, data: &[u8]) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error: cannot create directory {}: {e}", parent.display());
                std::process::exit(1);
            }
        }
    }
    if let Err(e) = std::fs::write(path, data) {
        eprintln!("error: cannot write {}: {e}", path.display());
        std::process::exit(1);
    }
}

/// Compile a .tex file to PDF using pdflatex or tectonic.
fn compile_pdf(tex_path: &PathBuf, pdf_path: &PathBuf) {
    let tex_dir = tex_path.parent().unwrap_or(std::path::Path::new("."));

    // Try pdflatex first, then tectonic
    let (cmd, args): (&str, Vec<&str>) = if which_exists("pdflatex") {
        ("pdflatex", vec!["-interaction=nonstopmode", "-halt-on-error"])
    } else if which_exists("tectonic") {
        ("tectonic", vec![])
    } else {
        eprintln!("error: neither pdflatex nor tectonic found in PATH");
        eprintln!("  Install TeX Live: https://tug.org/texlive/");
        eprintln!("  Or tectonic:      https://tectonic-typesetting.github.io/");
        std::process::exit(1);
    };

    eprintln!("Compiling PDF with {cmd}...");
    let status = std::process::Command::new(cmd)
        .args(&args)
        .arg(tex_path.file_name().unwrap())
        .current_dir(tex_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => {
            // pdflatex writes output next to the .tex file; move if needed
            let generated = tex_path.with_extension("pdf");
            if generated != *pdf_path {
                let _ = std::fs::rename(&generated, pdf_path);
            }
            // Clean up intermediate files
            for ext in &["aux", "log", "out"] {
                let _ = std::fs::remove_file(tex_path.with_extension(ext));
            }
        }
        Ok(s) => {
            eprintln!("error: {cmd} exited with status {s}");
            eprintln!("  Check {} for details", tex_path.with_extension("log").display());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: failed to run {cmd}: {e}");
            std::process::exit(1);
        }
    }
}

/// Check if a command exists in PATH.
fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
