//! Report generator: collect figures during a session and render them into
//! a self-contained HTML page with navigation.
//!
//! Designed so that notebook-style rendering (Option B) can layer on top by
//! adding `ReportEntry::Markdown` / `ReportEntry::Code` variants later.

use std::cell::RefCell;

use crate::figure::{FigureState, FIGURE};
use crate::html::render_figure_plotly_div;
use crate::theme::{Theme, ThemeColors};

// ─── Report data model ───────────────────────────────────────────────────────

/// A single entry in a report.
#[derive(Debug, Clone)]
pub enum ReportEntry {
    /// A captured figure with an optional section title.
    Figure {
        title: String,
        state: FigureState,
    },
}

/// A report being assembled.
#[derive(Debug, Clone)]
pub struct Report {
    pub title: String,
    pub entries: Vec<ReportEntry>,
}

// ─── Thread-local active report session ──────────────────────────────────────

thread_local! {
    static ACTIVE_REPORT: RefCell<Option<Report>> = RefCell::new(None);
}

/// Start a new report session. Any previous session is discarded.
pub fn report_start(title: &str) {
    ACTIVE_REPORT.with(|r| {
        *r.borrow_mut() = Some(Report {
            title: title.to_string(),
            entries: Vec::new(),
        });
    });
}

/// Returns true if a report session is active.
pub fn report_active() -> bool {
    ACTIVE_REPORT.with(|r| r.borrow().is_some())
}

/// Auto-capture: if a report is active and the current figure has data,
/// snapshot it into the report. Called automatically by `clf` and `figure()`.
pub fn report_auto_capture() {
    ACTIVE_REPORT.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(report) = r.as_mut() {
            let fig = FIGURE.with(|f| f.borrow().clone());
            if fig.subplots.iter().any(|s| !s.series.is_empty() || s.heatmap.is_some()) {
                let title = fig.subplots.first()
                    .map(|s| s.title.clone())
                    .unwrap_or_default();
                report.entries.push(ReportEntry::Figure { title, state: fig });
            }
        }
    });
}

/// Snapshot the current FIGURE and add it to the active report.
/// `section` is an optional heading shown above the figure.
/// Returns `Err` if no report session is active.
pub fn report_add(section: &str) -> Result<(), String> {
    ACTIVE_REPORT.with(|r| {
        let mut r = r.borrow_mut();
        let report = r.as_mut().ok_or("no report session — use 'report start \"title\"' first")?;
        let fig = FIGURE.with(|f| f.borrow().clone());
        let title = if section.is_empty() {
            fig.subplots.first()
                .map(|s| s.title.clone())
                .unwrap_or_default()
        } else {
            section.to_string()
        };
        report.entries.push(ReportEntry::Figure { title, state: fig });
        Ok(())
    })
}

/// End the report session and render to an HTML file.
/// Auto-captures the current figure if it has data before saving.
/// Returns the number of figures saved, or `Err` if no session is active.
pub fn report_save(path: &str) -> Result<usize, String> {
    // Auto-capture any remaining figure before saving
    report_auto_capture();

    let report = ACTIVE_REPORT.with(|r| r.borrow_mut().take())
        .ok_or("no report session — use 'report start \"title\"' first")?;

    let count = report.entries.len();
    let html = render_report_html(&report);

    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create directory: {e}"))?;
        }
    }
    std::fs::write(path, html)
        .map_err(|e| format!("cannot write report: {e}"))?;

    Ok(count)
}

/// Discard the active report session without saving.
pub fn report_end() {
    ACTIVE_REPORT.with(|r| *r.borrow_mut() = None);
}

/// Number of entries captured so far (0 if no session).
pub fn report_len() -> usize {
    ACTIVE_REPORT.with(|r| {
        r.borrow().as_ref().map_or(0, |rep| rep.entries.len())
    })
}

// ─── HTML rendering ──────────────────────────────────────────────────────────

fn render_report_html(report: &Report) -> String {
    let c = Theme::default().colors();
    render_report_html_themed(report, c)
}

fn render_report_html_themed(report: &Report, c: &ThemeColors) -> String {
    let mut nav_items = String::new();
    let mut sections = String::new();

    for (i, entry) in report.entries.iter().enumerate() {
        match entry {
            ReportEntry::Figure { title, state } => {
                let id = format!("fig-{}", i + 1);
                let display_title = if title.is_empty() {
                    format!("Figure {}", i + 1)
                } else {
                    title.clone()
                };

                // Navigation item
                nav_items.push_str(&format!(
                    "<a href=\"#{}\" class=\"nav-item\">{}</a>\n",
                    id,
                    escape_html(&display_title),
                ));

                // Section with plot
                let div_id = format!("plot-{}", i + 1);
                let plotly_div = render_figure_plotly_div(state, &div_id, c);
                sections.push_str(&format!(
                    "<section id=\"{id}\">\n<h2>{title}</h2>\n\
                     <div class=\"plot-container\">\n{plotly}\n</div>\n</section>\n",
                    id = id,
                    title = escape_html(&display_title),
                    plotly = plotly_div,
                ));
            }
        }
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<script src="https://cdn.plot.ly/plotly-2.35.0.min.js"></script>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background: {bg};
    color: {text};
    display: flex;
    min-height: 100vh;
  }}
  nav {{
    position: fixed;
    top: 0;
    left: 0;
    width: 220px;
    height: 100vh;
    background: {bg_secondary};
    border-right: 1px solid {border};
    padding: 1.5rem 0;
    overflow-y: auto;
  }}
  nav .nav-title {{
    font-size: 1.1rem;
    font-weight: 700;
    color: {accent_primary};
    padding: 0 1rem 1rem;
    border-bottom: 1px solid {border};
    margin-bottom: 0.5rem;
  }}
  nav .nav-item {{
    display: block;
    padding: 0.5rem 1rem;
    color: {text_dim};
    text-decoration: none;
    font-size: 0.9rem;
    transition: background 0.15s, color 0.15s;
  }}
  nav .nav-item:hover {{
    background: {border};
    color: {text};
  }}
  main {{
    margin-left: 220px;
    flex: 1;
    padding: 2rem;
    max-width: 1200px;
  }}
  main h1 {{
    font-size: 1.8rem;
    color: {accent_primary};
    margin-bottom: 2rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid {border};
  }}
  section {{
    margin-bottom: 3rem;
  }}
  section h2 {{
    font-size: 1.2rem;
    color: {accent_secondary};
    margin-bottom: 1rem;
  }}
  .plot-container {{
    background: {bg};
    border: 1px solid {border};
    border-radius: 8px;
    height: 500px;
  }}
</style>
</head>
<body>
<nav>
  <div class="nav-title">{title}</div>
{nav}
</nav>
<main>
<h1>{title}</h1>
{sections}
<footer style="color:{footer_text}; font-size:0.8rem; margin-top:2rem; padding-top:1rem; border-top:1px solid {border};">
  Generated by RustLab
</footer>
</main>
</body>
</html>
"##,
        title = escape_html(&report.title),
        nav = nav_items,
        sections = sections,
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
