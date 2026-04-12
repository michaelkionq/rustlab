use pulldown_cmark::{Parser, Options, html::push_html};
use rustlab_plot::render_figure_plotly_div;
use crate::execute::Rendered;

/// Render executed notebook blocks into a self-contained HTML string.
pub fn render_html(title: &str, blocks: &[Rendered]) -> String {
    let mut nav_items = String::new();
    let mut body = String::new();
    let mut heading_idx = 0;
    let mut plot_idx = 0;

    for block in blocks {
        match block {
            Rendered::Markdown(md) => {
                // Convert markdown to HTML
                let mut opts = Options::empty();
                opts.insert(Options::ENABLE_TABLES);
                opts.insert(Options::ENABLE_STRIKETHROUGH);
                let parser = Parser::new_ext(md, opts);
                let mut html = String::new();
                push_html(&mut html, parser);

                // Extract headings for nav and inject IDs
                let html = inject_heading_ids(&html, &mut nav_items, &mut heading_idx);

                body.push_str("<div class=\"prose\">\n");
                body.push_str(&html);
                body.push_str("</div>\n");
            }
            Rendered::Code { source, text_output, error, figure, hidden } => {
                body.push_str("<div class=\"code-block\">\n");

                // Source code (unless hidden)
                if !hidden {
                    body.push_str("<pre class=\"source\"><code>");
                    body.push_str(&highlight_rustlab(source));
                    body.push_str("</code></pre>\n");
                }

                // Text output (if any)
                let trimmed_output = text_output.trim();
                if !trimmed_output.is_empty() {
                    body.push_str("<pre class=\"output\">");
                    body.push_str(&escape_html(trimmed_output));
                    body.push_str("</pre>\n");
                }

                // Error (if any)
                if let Some(err) = error {
                    body.push_str("<pre class=\"error\">");
                    body.push_str(&escape_html(err));
                    body.push_str("</pre>\n");
                }

                // Plot (if any)
                if let Some(fig) = figure {
                    plot_idx += 1;
                    let div_id = format!("plot-{plot_idx}");
                    let plotly_div = render_figure_plotly_div(fig, &div_id);
                    body.push_str("<div class=\"plot-container\">\n");
                    body.push_str(&plotly_div);
                    body.push_str("\n</div>\n");
                }

                body.push_str("</div>\n");
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
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.21/dist/katex.min.css">
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.21/dist/katex.min.js"></script>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.21/dist/contrib/auto-render.min.js"
  onload="renderMathInElement(document.body, {{
    delimiters: [
      {{left: '$$', right: '$$', display: true}},
      {{left: '$', right: '$', display: false}}
    ]
  }});"></script>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background: #1e1e2e;
    color: #cdd6f4;
    display: flex;
    min-height: 100vh;
  }}
  /* ── Navigation sidebar ── */
  nav {{
    position: fixed;
    top: 0;
    left: 0;
    width: 220px;
    height: 100vh;
    background: #181825;
    border-right: 1px solid #313244;
    padding: 1.5rem 0;
    overflow-y: auto;
    z-index: 100;
    transition: transform 0.25s ease;
  }}
  nav .nav-title {{
    font-size: 1.1rem;
    font-weight: 700;
    color: #cba6f7;
    padding: 0 1rem 1rem;
    border-bottom: 1px solid #313244;
    margin-bottom: 0.5rem;
  }}
  nav a {{
    display: block;
    padding: 0.4rem 1rem;
    color: #a6adc8;
    text-decoration: none;
    font-size: 0.9rem;
    transition: background 0.15s, color 0.15s;
  }}
  nav a:hover {{
    background: #313244;
    color: #cdd6f4;
  }}
  nav a.h2 {{
    padding-left: 1.8rem;
    font-size: 0.85rem;
  }}
  nav a.h3 {{
    padding-left: 2.6rem;
    font-size: 0.8rem;
  }}
  /* ── Hamburger toggle (hidden on desktop) ── */
  .nav-toggle {{
    display: none;
    position: fixed;
    top: 0.7rem;
    left: 0.7rem;
    z-index: 200;
    background: #313244;
    border: 1px solid #45475a;
    border-radius: 6px;
    color: #cdd6f4;
    font-size: 1.3rem;
    width: 2.4rem;
    height: 2.4rem;
    cursor: pointer;
    line-height: 1;
  }}
  /* ── Main content ── */
  main {{
    margin-left: 220px;
    flex: 1;
    padding: 2rem 2.5rem;
    max-width: 960px;
    min-width: 0;
  }}
  .prose {{
    line-height: 1.7;
    margin-bottom: 1.5rem;
  }}
  .prose h1 {{
    font-size: 1.8rem;
    color: #cba6f7;
    margin: 2rem 0 1rem;
    padding-bottom: 0.4rem;
    border-bottom: 1px solid #313244;
  }}
  .prose h2 {{
    font-size: 1.4rem;
    color: #89b4fa;
    margin: 1.8rem 0 0.8rem;
  }}
  .prose h3 {{
    font-size: 1.15rem;
    color: #74c7ec;
    margin: 1.4rem 0 0.6rem;
  }}
  .prose p {{
    margin-bottom: 1rem;
  }}
  .prose code {{
    background: #313244;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
    font-size: 0.9em;
  }}
  .prose table {{
    border-collapse: collapse;
    margin: 1rem 0;
  }}
  .prose th, .prose td {{
    border: 1px solid #45475a;
    padding: 0.5rem 0.8rem;
    text-align: left;
  }}
  .prose th {{
    background: #313244;
    color: #cba6f7;
    font-weight: 600;
  }}
  .prose ul, .prose ol {{
    margin: 0.5rem 0 1rem 1.5rem;
  }}
  .prose li {{
    margin-bottom: 0.3rem;
  }}
  .prose blockquote {{
    border-left: 3px solid #cba6f7;
    padding-left: 1rem;
    color: #a6adc8;
    margin: 1rem 0;
  }}
  .code-block {{
    margin-bottom: 1.5rem;
  }}
  .source {{
    background: #11111b;
    border: 1px solid #313244;
    border-radius: 6px;
    padding: 1rem;
    overflow-x: auto;
    font-family: "SF Mono", "Fira Code", "JetBrains Mono", monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    color: #cdd6f4;
  }}
  .output {{
    background: #181825;
    border: 1px solid #313244;
    border-radius: 6px;
    padding: 0.8rem 1rem;
    margin-top: 0.5rem;
    color: #a6adc8;
    font-family: "SF Mono", "Fira Code", "JetBrains Mono", monospace;
    font-size: 0.85rem;
    white-space: pre-wrap;
    line-height: 1.5;
  }}
  .error {{
    background: #1e0a0a;
    border: 1px solid #f38ba8;
    border-radius: 6px;
    padding: 0.8rem 1rem;
    margin-top: 0.5rem;
    color: #f38ba8;
    font-family: "SF Mono", "Fira Code", "JetBrains Mono", monospace;
    font-size: 0.85rem;
    white-space: pre-wrap;
  }}
  .plot-container {{
    background: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 8px;
    margin-top: 0.5rem;
    height: 450px;
  }}
  .plot-container > div {{
    width: 100%;
    height: 100%;
  }}
  footer {{
    color: #585b70;
    font-size: 0.8rem;
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid #313244;
  }}
  /* ── Syntax highlighting (Catppuccin Mocha) ── */
  .syn-kw  {{ color: #cba6f7; }}           /* keywords: if, for, function, ... */
  .syn-fn  {{ color: #89b4fa; }}           /* function calls */
  .syn-num {{ color: #fab387; }}           /* numbers */
  .syn-str {{ color: #a6e3a1; }}           /* strings */
  .syn-com {{ color: #6c7086; font-style: italic; }}  /* comments */
  .syn-op  {{ color: #89dceb; }}           /* operators */
  /* ── Responsive: collapse sidebar on narrow screens ── */
  @media (max-width: 768px) {{
    nav {{
      transform: translateX(-100%);
    }}
    nav.open {{
      transform: translateX(0);
    }}
    .nav-toggle {{
      display: block;
    }}
    main {{
      margin-left: 0;
      padding: 3rem 1rem 2rem;
    }}
  }}
</style>
</head>
<body>
<button class="nav-toggle" onclick="document.querySelector('nav').classList.toggle('open')" aria-label="Toggle navigation">&#9776;</button>
<nav>
  <div class="nav-title">{title}</div>
{nav}
</nav>
<main>
{body}
<footer>Generated by rustlab-notebook</footer>
</main>
</body>
</html>
"##,
        title = escape_html(title),
        nav = nav_items,
        body = body,
    )
}

/// Scan HTML for <h1>–<h3> tags. For each heading found:
/// 1. Inject an `id` attribute so nav links can scroll to it.
/// 2. Append a nav link to `nav`.
/// Returns the modified HTML.
fn inject_heading_ids(html: &str, nav: &mut String, idx: &mut usize) -> String {
    let mut result = html.to_string();
    for tag in ["h1", "h2", "h3"] {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let mut search_from = 0;
        loop {
            let Some(start) = result[search_from..].find(&open) else { break };
            let abs_open = search_from + start;
            let content_start = abs_open + open.len();
            let Some(rel_end) = result[content_start..].find(&close) else { break };
            let content = result[content_start..content_start + rel_end].to_string();
            let clean = strip_tags(&content);
            if !clean.is_empty() {
                *idx += 1;
                let id = format!("heading-{idx}");
                // Replace <hN> with <hN id="heading-N">
                let new_open = format!("<{tag} id=\"{id}\">");
                result.replace_range(abs_open..abs_open + open.len(), &new_open);
                // Build nav link
                nav.push_str(&format!(
                    "  <a href=\"#{id}\" class=\"{tag}\">{text}</a>\n",
                    id = id,
                    tag = tag,
                    text = escape_html(&clean),
                ));
                search_from = abs_open + new_open.len() + rel_end + close.len();
            } else {
                search_from = content_start + rel_end + close.len();
            }
        }
    }
    result
}

/// Strip HTML tags from a string.
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

// ── Syntax highlighting ─────────────────────────────────────────────────────

const KEYWORDS: &[&str] = &[
    "function", "end", "return", "if", "elseif", "else",
    "for", "while", "switch", "case", "otherwise",
];

/// Produce syntax-highlighted HTML for a rustlab code snippet.
/// Returns HTML with <span class="syn-*"> wrappers (already escaped).
fn highlight_rustlab(source: &str) -> String {
    let mut out = String::with_capacity(source.len() * 2);
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Comment: % to end of line
        if ch == '%' {
            out.push_str("<span class=\"syn-com\">");
            while i < len && chars[i] != '\n' {
                push_escaped_char(&mut out, chars[i]);
                i += 1;
            }
            out.push_str("</span>");
            continue;
        }

        // String: "..." or '...' (single-char or multi-char)
        if ch == '"' || (ch == '\'' && is_string_quote(&chars, i)) {
            let quote = ch;
            out.push_str("<span class=\"syn-str\">");
            push_escaped_char(&mut out, ch);
            i += 1;
            while i < len && chars[i] != quote && chars[i] != '\n' {
                push_escaped_char(&mut out, chars[i]);
                i += 1;
            }
            if i < len && chars[i] == quote {
                push_escaped_char(&mut out, chars[i]);
                i += 1;
            }
            out.push_str("</span>");
            continue;
        }

        // Number: digits, optionally with . or e
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            out.push_str("<span class=\"syn-num\">");
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E'
                || ((chars[i] == '+' || chars[i] == '-') && i > 0 && (chars[i-1] == 'e' || chars[i-1] == 'E')))
            {
                push_escaped_char(&mut out, chars[i]);
                i += 1;
            }
            // Trailing 'i' or 'j' for complex literals
            if i < len && (chars[i] == 'i' || chars[i] == 'j') {
                push_escaped_char(&mut out, chars[i]);
                i += 1;
            }
            out.push_str("</span>");
            continue;
        }

        // Identifier or keyword
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            if KEYWORDS.contains(&word.as_str()) {
                out.push_str("<span class=\"syn-kw\">");
                out.push_str(&escape_html(&word));
                out.push_str("</span>");
            } else if i < len && chars[i] == '(' {
                // Function call
                out.push_str("<span class=\"syn-fn\">");
                out.push_str(&escape_html(&word));
                out.push_str("</span>");
            } else {
                out.push_str(&escape_html(&word));
            }
            continue;
        }

        // Operators
        if is_operator(ch) {
            out.push_str("<span class=\"syn-op\">");
            // Handle two-char operators
            if i + 1 < len {
                let next = chars[i + 1];
                let two: String = [ch, next].iter().collect();
                if matches!(two.as_str(), ".^" | ".*" | "./" | ".'" | "==" | "~=" | "<=" | ">=" | "&&" | "||") {
                    push_escaped_char(&mut out, ch);
                    push_escaped_char(&mut out, next);
                    i += 2;
                    out.push_str("</span>");
                    continue;
                }
            }
            push_escaped_char(&mut out, ch);
            i += 1;
            out.push_str("</span>");
            continue;
        }

        // Everything else (whitespace, parens, etc.)
        push_escaped_char(&mut out, ch);
        i += 1;
    }

    out
}

/// Determine if a single quote at position `i` starts a string literal
/// (as opposed to being the transpose operator).
fn is_string_quote(chars: &[char], i: usize) -> bool {
    if i == 0 { return true; }
    let prev = chars[i - 1];
    // After ), ], identifier char, or digit — it's transpose
    if prev == ')' || prev == ']' || prev.is_ascii_alphanumeric() || prev == '_' || prev == '.' {
        return false;
    }
    true
}

fn is_operator(ch: char) -> bool {
    matches!(ch, '+' | '-' | '*' | '/' | '\\' | '^' | '=' | '<' | '>' | '~' | '&' | '|' | ':' | ';' | ',')
}

fn push_escaped_char(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        _ => out.push(ch),
    }
}
