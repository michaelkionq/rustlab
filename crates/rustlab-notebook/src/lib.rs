pub mod parse;
pub mod execute;
pub mod render;
pub mod render_latex;

use std::path::PathBuf;
use rustlab_plot::theme::ThemeColors;

/// Render a single notebook file to the chosen format.
pub fn cmd_render(input: PathBuf, output: Option<PathBuf>, format: Format, theme: &ThemeColors) {
    let source = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {e}", input.display());
            std::process::exit(1);
        }
    };

    if let Some(dir) = input.parent() {
        if dir.as_os_str().len() > 0 {
            let _ = std::env::set_current_dir(dir);
        }
    }

    let title = extract_title(&source, &input);
    let blocks = parse::parse_notebook(&source);
    let rendered = execute::execute_notebook(&blocks);

    let ext = format.extension();
    let out_path = output.unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default();
        PathBuf::from(format!("{}.{ext}", stem.to_string_lossy()))
    });

    render_output(&out_path, &format, &title, &rendered, theme);
    print_summary(&input, &out_path, &rendered);
}

/// Render all .md files in a directory.
pub fn cmd_render_dir(dir: PathBuf, output: Option<PathBuf>, format: Format, theme: &ThemeColors) {
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    let out_dir = output
        .map(|o| std::path::absolute(&o).unwrap_or(o))
        .unwrap_or_else(|| dir.clone());

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

    let ext = format.extension();
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

        let _ = std::env::set_current_dir(&dir);

        let blocks = parse::parse_notebook(&source);
        let rendered = execute::execute_notebook(&blocks);

        render_output(&out_file, &format, &title, &rendered, theme);
        print_summary(md_path, &out_file, &rendered);

        index_entries.push((title, format!("{stem}.{ext}")));
    }

    if matches!(format, Format::Html) {
        let dir_name = dir.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let index_html = generate_index_html(&dir_name, &index_entries, theme);
        let index_path = out_dir.join("index.html");
        write_output(&index_path, index_html.as_bytes());
        println!("Generated {} ({} notebooks)", index_path.display(), index_entries.len());
    }
}

/// Output format.
#[derive(Clone)]
pub enum Format {
    Html,
    Latex,
    Pdf,
}

impl Format {
    pub fn extension(&self) -> &'static str {
        match self {
            Format::Html => "html",
            Format::Latex => "tex",
            Format::Pdf => "pdf",
        }
    }
}

fn render_output(
    out_path: &PathBuf,
    format: &Format,
    title: &str,
    rendered: &[execute::Rendered],
    theme: &ThemeColors,
) {
    match format {
        Format::Html => {
            let html = render::render_html(title, rendered, theme);
            write_output(out_path, html.as_bytes());
        }
        Format::Latex => {
            let plot_dir = plot_dir_for(out_path);
            let tex = render_latex::render_latex(title, rendered, &plot_dir, theme);
            write_output(out_path, tex.as_bytes());
        }
        Format::Pdf => {
            let tex_path = out_path.with_extension("tex");
            let plot_dir = plot_dir_for(&tex_path);
            let tex = render_latex::render_latex(title, rendered, &plot_dir, theme);
            write_output(&tex_path, tex.as_bytes());
            compile_pdf(&tex_path, out_path);
        }
    }
}

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

fn plot_dir_for(tex_path: &PathBuf) -> PathBuf {
    let stem = tex_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = tex_path.parent().unwrap_or(std::path::Path::new("."));
    parent.join(format!("{stem}_plots"))
}

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

fn compile_pdf(tex_path: &PathBuf, pdf_path: &PathBuf) {
    let tex_dir = tex_path.parent().unwrap_or(std::path::Path::new("."));

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
            let generated = tex_path.with_extension("pdf");
            if generated != *pdf_path {
                let _ = std::fs::rename(&generated, pdf_path);
            }
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

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn extract_title(source: &str, path: &PathBuf) -> String {
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

pub fn generate_index_html(dir_name: &str, entries: &[(String, String)], theme: &ThemeColors) -> String {
    let c = theme;
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
    background: {bg};
    color: {text};
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
    color: {accent_primary};
    margin-bottom: 0.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid {border};
  }}
  .subtitle {{
    color: {text_dim};
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
    background: {bg_secondary};
    border: 1px solid {border};
    border-radius: 8px;
    color: {accent_secondary};
    text-decoration: none;
    font-size: 1.05rem;
    transition: background 0.15s, border-color 0.15s;
  }}
  a:hover {{
    background: {border};
    border-color: {accent_secondary};
  }}
  footer {{
    color: {footer_text};
    font-size: 0.8rem;
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid {border};
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
        bg = c.bg,
        bg_secondary = c.bg_secondary,
        text = c.text,
        text_dim = c.text_dim,
        border = c.border,
        accent_primary = c.accent_primary,
        accent_secondary = c.accent_secondary,
        footer_text = c.footer_text,
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
    use rustlab_plot::Theme;

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
        let html = generate_index_html("notebooks", &entries, Theme::Dark.colors());
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
        let html = generate_index_html("test", &entries, Theme::Dark.colors());
        assert!(html.contains("1 notebook"));
        assert!(!html.contains("notebooks")); // singular
    }

    #[test]
    fn generate_index_empty() {
        let html = generate_index_html("empty", &[], Theme::Dark.colors());
        assert!(html.contains("0 notebooks"));
    }

    #[test]
    fn generate_index_escapes_html() {
        let entries = vec![
            ("A <script> & \"test\"".to_string(), "test.html".to_string()),
        ];
        let html = generate_index_html("dir", &entries, Theme::Dark.colors());
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;"));
    }
}
