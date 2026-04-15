mod parse;
mod execute;
mod render;
mod render_latex;

use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

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
enum Format {
    Html,
    Latex,
    Pdf,
}

#[derive(Subcommand)]
enum Command {
    /// Render a notebook (or directory of notebooks) to HTML, LaTeX, or PDF
    Render {
        /// Input .md file or directory of .md files
        input: PathBuf,
        /// Output file or directory (default: <input_stem>.<ext> or same directory)
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
        Command::Render { input, output, format } => {
            if input.is_dir() {
                cmd_render_dir(input, output, format);
            } else {
                cmd_render(input, output, format);
            }
        }
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
    render_output(&out_path, &format, &title, &rendered);

    // Count results
    print_summary(&input, &out_path, &rendered);
}

fn cmd_render_dir(dir: PathBuf, output: Option<PathBuf>, format: Format) {
    let out_dir = output.unwrap_or_else(|| dir.clone());

    // Collect all .md files in the directory (non-recursive)
    let mut md_files: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
            .collect(),
        Err(e) => {
            eprintln!("error: cannot read directory {}: {e}", dir.display());
            std::process::exit(1);
        }
    };
    md_files.sort();

    if md_files.is_empty() {
        eprintln!("warning: no .md files found in {}", dir.display());
        return;
    }

    let ext = match format {
        Format::Html => "html",
        Format::Latex => "tex",
        Format::Pdf => "pdf",
    };

    let mut index_entries: Vec<(String, String)> = Vec::new();

    for md_path in &md_files {
        let source = match std::fs::read_to_string(md_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("warning: cannot read {}: {e}", md_path.display());
                continue;
            }
        };

        let title = extract_title(&source, md_path);
        let stem = md_path.file_stem().unwrap_or_default().to_string_lossy();
        let out_file = out_dir.join(format!("{stem}.{ext}"));

        // Set working dir to the notebook's directory for relative paths
        let _ = std::env::set_current_dir(&dir);

        let blocks = parse::parse_notebook(&source);
        let rendered = execute::execute_notebook(&blocks);

        render_output(&out_file, &format, &title, &rendered);
        print_summary(md_path, &out_file, &rendered);

        index_entries.push((title, format!("{stem}.{ext}")));
    }

    // Generate index page (HTML only)
    if matches!(format, Format::Html) {
        let dir_name = dir.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let index_html = generate_index_html(&dir_name, &index_entries);
        let index_path = out_dir.join("index.html");
        write_output(&index_path, index_html.as_bytes());
        println!("Generated {} ({} notebooks)", index_path.display(), index_entries.len());
    }
}

/// Render executed blocks to the chosen output format.
fn render_output(
    out_path: &PathBuf,
    format: &Format,
    title: &str,
    rendered: &[execute::Rendered],
) {
    match format {
        Format::Html => {
            let html = render::render_html(title, rendered);
            write_output(out_path, html.as_bytes());
        }
        Format::Latex => {
            let plot_dir = plot_dir_for(out_path);
            let tex = render_latex::render_latex(title, rendered, &plot_dir);
            write_output(out_path, tex.as_bytes());
        }
        Format::Pdf => {
            // Generate .tex first, then compile with pdflatex
            let tex_path = out_path.with_extension("tex");
            let plot_dir = plot_dir_for(&tex_path);
            let tex = render_latex::render_latex(title, rendered, &plot_dir);
            write_output(&tex_path, tex.as_bytes());
            compile_pdf(&tex_path, out_path);
        }
    }
}

/// Print a summary line for a rendered notebook.
fn print_summary(input: &PathBuf, out_path: &PathBuf, rendered: &[execute::Rendered]) {
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

/// Generate an index HTML page linking to all rendered notebooks.
fn generate_index_html(dir_name: &str, entries: &[(String, String)]) -> String {
    let mut links = String::new();
    for (title, filename) in entries {
        links.push_str(&format!(
            "  <li><a href=\"{filename}\">{title}</a></li>\n",
            filename = escape_html(filename),
            title = escape_html(title),
        ));
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Notebook Index</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background: #1e1e2e;
    color: #cdd6f4;
    display: flex;
    justify-content: center;
    min-height: 100vh;
    padding: 3rem 1.5rem;
  }}
  main {{
    max-width: 720px;
    width: 100%;
  }}
  h1 {{
    font-size: 2rem;
    color: #cba6f7;
    margin-bottom: 0.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #313244;
  }}
  .subtitle {{
    color: #a6adc8;
    font-size: 0.9rem;
    margin-bottom: 2rem;
  }}
  ul {{
    list-style: none;
    padding: 0;
  }}
  li {{
    margin-bottom: 0.5rem;
  }}
  a {{
    display: block;
    padding: 0.8rem 1.2rem;
    background: #181825;
    border: 1px solid #313244;
    border-radius: 8px;
    color: #89b4fa;
    text-decoration: none;
    font-size: 1.05rem;
    transition: background 0.15s, border-color 0.15s;
  }}
  a:hover {{
    background: #313244;
    border-color: #89b4fa;
  }}
  footer {{
    color: #585b70;
    font-size: 0.8rem;
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid #313244;
  }}
</style>
</head>
<body>
<main>
<h1>{title}</h1>
<p class="subtitle">{count} notebook{plural}</p>
<ul>
{links}</ul>
<footer>Generated by rustlab-notebook</footer>
</main>
</body>
</html>
"##,
        title = escape_html(dir_name),
        count = entries.len(),
        plural = if entries.len() == 1 { "" } else { "s" },
        links = links,
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_title_from_heading() {
        let source = "# My Analysis\n\nSome text.";
        let title = extract_title(source, &PathBuf::from("analysis.md"));
        assert_eq!(title, "My Analysis");
    }

    #[test]
    fn extract_title_fallback_to_filename() {
        let source = "No heading here.";
        let title = extract_title(source, &PathBuf::from("my_report.md"));
        assert_eq!(title, "my_report");
    }

    #[test]
    fn extract_title_ignores_h2() {
        let source = "## Sub Heading\n\nText.";
        let title = extract_title(source, &PathBuf::from("test.md"));
        assert_eq!(title, "test");
    }

    #[test]
    fn generate_index_basic() {
        let entries = vec![
            ("Filter Analysis".to_string(), "filter.html".to_string()),
            ("Quick Look".to_string(), "quick.html".to_string()),
        ];
        let html = generate_index_html("notebooks", &entries);
        assert!(html.contains("notebooks"));
        assert!(html.contains("2 notebooks"));
        assert!(html.contains("href=\"filter.html\""));
        assert!(html.contains("Filter Analysis"));
        assert!(html.contains("href=\"quick.html\""));
        assert!(html.contains("Quick Look"));
        assert!(html.contains("Generated by rustlab-notebook"));
    }

    #[test]
    fn generate_index_single() {
        let entries = vec![
            ("Solo".to_string(), "solo.html".to_string()),
        ];
        let html = generate_index_html("test", &entries);
        assert!(html.contains("1 notebook"));
        assert!(!html.contains("notebooks")); // singular
    }

    #[test]
    fn generate_index_empty() {
        let html = generate_index_html("empty", &[]);
        assert!(html.contains("0 notebooks"));
    }

    #[test]
    fn generate_index_escapes_html() {
        let entries = vec![
            ("A <script> & \"test\"".to_string(), "test.html".to_string()),
        ];
        let html = generate_index_html("dir", &entries);
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;"));
    }
}
