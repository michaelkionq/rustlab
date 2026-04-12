use std::path::Path;
use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd, HeadingLevel};
use crate::execute::Rendered;

/// Render executed notebook blocks into a LaTeX document string.
/// Plot images are written to `plot_dir` as PNG files and referenced
/// via \includegraphics.
pub fn render_latex(title: &str, blocks: &[Rendered], plot_dir: &Path) -> String {
    let mut body = String::new();
    let mut plot_idx = 0;

    // Ensure plot directory exists
    let _ = std::fs::create_dir_all(plot_dir);

    for block in blocks {
        match block {
            Rendered::Markdown(md) => {
                body.push_str(&markdown_to_latex(md));
            }
            Rendered::Code { source, text_output, error, figure } => {
                // Source code
                body.push_str("\\begin{verbatim}\n");
                body.push_str(source);
                body.push_str("\n\\end{verbatim}\n\n");

                // Text output
                let trimmed = text_output.trim();
                if !trimmed.is_empty() {
                    body.push_str("\\begin{quote}\n\\ttfamily\\small\n\\begin{verbatim}\n");
                    body.push_str(trimmed);
                    body.push_str("\n\\end{verbatim}\n\\end{quote}\n\n");
                }

                // Error
                if let Some(err) = error {
                    body.push_str("\\begin{quote}\n{\\color{red}\\ttfamily\\small\n\\begin{verbatim}\n");
                    body.push_str(err);
                    body.push_str("\n\\end{verbatim}\n}\\end{quote}\n\n");
                }

                // Plot
                if let Some(fig) = figure {
                    plot_idx += 1;
                    let plot_file = plot_dir.join(format!("plot-{plot_idx}.svg"));
                    if let Err(e) = rustlab_plot::render_figure_state_to_file(fig, &plot_file.to_string_lossy()) {
                        eprintln!("warning: could not render plot-{plot_idx}: {e}");
                    } else {
                        // Use relative path from the .tex file's perspective
                        let rel_path = plot_dir.file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        body.push_str(&format!(
                            "\\begin{{center}}\n\\includesvg[width=0.9\\textwidth]{{{}/{}}}\n\\end{{center}}\n\n",
                            rel_path,
                            format!("plot-{plot_idx}"),
                        ));
                    }
                }
            }
        }
    }

    format!(
        r#"\documentclass[11pt,a4paper]{{article}}
\usepackage[utf8]{{inputenc}}
\usepackage[T1]{{fontenc}}
\usepackage{{geometry}}
\geometry{{margin=1in}}
\usepackage{{graphicx}}
\usepackage{{svg}}
\usepackage{{amsmath,amssymb}}
\usepackage{{xcolor}}
\usepackage{{booktabs}}
\usepackage{{hyperref}}
\hypersetup{{colorlinks=true,linkcolor=blue,urlcolor=blue}}

\title{{{title}}}
\date{{\today}}

\begin{{document}}
\maketitle

{body}
\end{{document}}
"#,
        title = escape_latex(title),
        body = body,
    )
}

/// Convert a markdown string to LaTeX using pulldown-cmark events.
fn markdown_to_latex(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_MATH);
    let parser = Parser::new_ext(md, opts);

    let mut out = String::new();
    let mut in_table = false;
    let mut table_alignments: Vec<pulldown_cmark::Alignment> = Vec::new();
    let mut table_cell_idx = 0;
    let mut table_in_head = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let cmd = match level {
                        HeadingLevel::H1 => "section",
                        HeadingLevel::H2 => "subsection",
                        HeadingLevel::H3 => "subsubsection",
                        _ => "paragraph",
                    };
                    out.push_str(&format!("\\{cmd}{{"));
                }
                Tag::Paragraph => {}
                Tag::Emphasis => out.push_str("\\emph{"),
                Tag::Strong => out.push_str("\\textbf{"),
                Tag::CodeBlock(_) => {
                    // Fenced code blocks in markdown (non-rustlab) treated as verbatim
                    out.push_str("\\begin{verbatim}\n");
                }
                Tag::BlockQuote(_) => out.push_str("\\begin{quote}\n"),
                Tag::List(Some(1)) => out.push_str("\\begin{enumerate}\n"),
                Tag::List(Some(_)) => out.push_str("\\begin{enumerate}\n"),
                Tag::List(None) => out.push_str("\\begin{itemize}\n"),
                Tag::Item => out.push_str("\\item "),
                Tag::Table(alignments) => {
                    in_table = true;
                    table_alignments = alignments;
                    let cols: String = table_alignments.iter().map(|a| match a {
                        pulldown_cmark::Alignment::Left | pulldown_cmark::Alignment::None => 'l',
                        pulldown_cmark::Alignment::Center => 'c',
                        pulldown_cmark::Alignment::Right => 'r',
                    }).collect();
                    out.push_str(&format!("\\begin{{tabular}}{{{cols}}}\n\\toprule\n"));
                }
                Tag::TableHead => {
                    table_in_head = true;
                    table_cell_idx = 0;
                }
                Tag::TableRow => {
                    table_cell_idx = 0;
                }
                Tag::TableCell => {
                    if table_cell_idx > 0 {
                        out.push_str(" & ");
                    }
                }
                Tag::Link { dest_url, .. } => {
                    out.push_str(&format!("\\href{{{}}}", dest_url));
                    out.push('{');
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => out.push_str("}\n\n"),
                TagEnd::Paragraph => out.push_str("\n\n"),
                TagEnd::Emphasis => out.push('}'),
                TagEnd::Strong => out.push('}'),
                TagEnd::CodeBlock => out.push_str("\\end{verbatim}\n\n"),
                TagEnd::BlockQuote(_) => out.push_str("\\end{quote}\n"),
                TagEnd::List(true) => out.push_str("\\end{enumerate}\n\n"),
                TagEnd::List(false) => out.push_str("\\end{itemize}\n\n"),
                TagEnd::Item => out.push('\n'),
                TagEnd::Table => {
                    out.push_str("\\bottomrule\n\\end{tabular}\n\n");
                    in_table = false;
                }
                TagEnd::TableHead => {
                    out.push_str(" \\\\\n\\midrule\n");
                    table_in_head = false;
                }
                TagEnd::TableRow => {
                    if !table_in_head {
                        out.push_str(" \\\\\n");
                    }
                }
                TagEnd::TableCell => {
                    table_cell_idx += 1;
                }
                TagEnd::Link => out.push('}'),
                _ => {}
            },
            Event::Text(text) => {
                if in_table {
                    // Don't escape $ in tables — formulas should pass through
                    out.push_str(&escape_latex_preserving_math(&text));
                } else {
                    out.push_str(&escape_latex_preserving_math(&text));
                }
            }
            Event::Code(code) => {
                out.push_str(&format!("\\texttt{{{}}}", escape_latex(&code)));
            }
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("\\\\\n"),
            Event::InlineMath(math) => {
                out.push('$');
                out.push_str(&math);
                out.push('$');
            }
            Event::DisplayMath(math) => {
                out.push_str("\\[\n");
                out.push_str(&math);
                out.push_str("\n\\]\n");
            }
            Event::Html(html) => {
                // HTML comments / directives — skip
                let _ = html;
            }
            _ => {}
        }
    }

    out
}

/// Escape special LaTeX characters, but preserve $...$ math delimiters.
fn escape_latex_preserving_math(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_math = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '$' {
            in_math = !in_math;
            out.push('$');
            i += 1;
            continue;
        }
        if in_math {
            out.push(ch);
        } else {
            match ch {
                '&' => out.push_str("\\&"),
                '%' => out.push_str("\\%"),
                '#' => out.push_str("\\#"),
                '_' => out.push_str("\\_"),
                '{' => out.push_str("\\{"),
                '}' => out.push_str("\\}"),
                '~' => out.push_str("\\textasciitilde{}"),
                '^' => out.push_str("\\textasciicircum{}"),
                '\\' => out.push_str("\\textbackslash{}"),
                _ => out.push(ch),
            }
        }
        i += 1;
    }

    out
}

/// Escape special LaTeX characters (no math preservation).
fn escape_latex(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '\\' => out.push_str("\\textbackslash{}"),
            '$' => out.push_str("\\$"),
            _ => out.push(ch),
        }
    }
    out
}
