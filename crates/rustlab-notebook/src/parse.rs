/// Directives that modify how a code block is displayed.
#[derive(Debug, Clone, Default)]
pub struct CodeDirectives {
    /// Hide the source code (still executed).
    pub hidden: bool,
    /// Wrap output in a collapsible disclosure widget with this title.
    pub details: Option<String>,
    /// Tile figure outputs N-across in a grid layout.
    pub grid_cols: Option<usize>,
}

/// The kind of callout box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutKind {
    Note,
    Tip,
    Warning,
}

/// A block in a parsed notebook.
#[derive(Debug, Clone)]
pub enum Block {
    /// Raw markdown text (to be rendered as HTML).
    Markdown(String),
    /// A ```rustlab fenced code block (source to be executed).
    Code {
        source: String,
        directives: CodeDirectives,
    },
    /// A callout box (note, tip, warning).
    Callout { kind: CalloutKind, content: String },
    /// Start of a numbered exercise.
    ExerciseStart,
    /// Start of a solution (collapsed by default).
    SolutionStart,
}

/// Parse a markdown notebook into a sequence of blocks.
///
/// - Optional YAML frontmatter (`---` delimited) is stripped.
/// - ` ```rustlab ` fenced code blocks become `Block::Code`.
/// - Everything else (including other fenced blocks) becomes `Block::Markdown`.
pub fn parse_notebook(src: &str) -> Vec<Block> {
    let src = strip_frontmatter(src);
    let mut blocks = Vec::new();
    let mut markdown_buf = String::new();
    let mut code_buf = String::new();
    let mut in_rustlab = false;
    let mut code_directives = CodeDirectives::default();

    // State for exercise/solution scope tracking
    let mut in_exercise = false;
    let mut in_solution = false;

    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if in_rustlab {
            if trimmed == "```" {
                // End of rustlab code block
                blocks.push(Block::Code {
                    source: code_buf.clone(),
                    directives: code_directives.clone(),
                });
                code_buf.clear();
                in_rustlab = false;
                code_directives = CodeDirectives::default();
            } else {
                if !code_buf.is_empty() {
                    code_buf.push('\n');
                }
                code_buf.push_str(line);
            }
            i += 1;
        } else if trimmed == "```rustlab" || trimmed.starts_with("```rustlab ") {
            // Extract stacked code-block directives from the tail of the markdown buffer
            code_directives = extract_code_directives(&mut markdown_buf);
            // Start of rustlab code block — flush markdown buffer
            if !markdown_buf.is_empty() {
                blocks.push(Block::Markdown(markdown_buf.clone()));
                markdown_buf.clear();
            }
            in_rustlab = true;
            i += 1;
        } else if let Some(directive) = parse_markdown_directive(trimmed) {
            match directive {
                MarkdownDirective::Callout(kind) => {
                    // Flush markdown buffer
                    if !markdown_buf.is_empty() {
                        blocks.push(Block::Markdown(markdown_buf.clone()));
                        markdown_buf.clear();
                    }
                    // Collect callout content until blank line, heading, or closing tag
                    let closing_tag = match kind {
                        CalloutKind::Note => "<!-- /note -->",
                        CalloutKind::Tip => "<!-- /tip -->",
                        CalloutKind::Warning => "<!-- /warning -->",
                    };
                    let mut content = String::new();
                    i += 1;
                    while i < lines.len() {
                        let cl = lines[i];
                        let ct = cl.trim();
                        if ct == closing_tag {
                            i += 1; // consume closing tag
                            break;
                        }
                        if ct.is_empty() || ct.starts_with('#') {
                            break; // blank line or heading ends single-paragraph callout
                        }
                        if !content.is_empty() {
                            content.push('\n');
                        }
                        content.push_str(cl);
                        i += 1;
                    }
                    if !content.is_empty() {
                        blocks.push(Block::Callout { kind, content });
                    }
                }
                MarkdownDirective::ExerciseStart => {
                    // Flush markdown buffer
                    if !markdown_buf.is_empty() {
                        blocks.push(Block::Markdown(markdown_buf.clone()));
                        markdown_buf.clear();
                    }
                    // Auto-close any open solution/exercise
                    if in_solution {
                        blocks.push(Block::SolutionStart); // close marker handled by renderer
                        in_solution = false;
                    }
                    if in_exercise {
                        // previous exercise had no solution — that's ok
                    }
                    blocks.push(Block::ExerciseStart);
                    in_exercise = true;
                    i += 1;
                }
                MarkdownDirective::SolutionStart => {
                    // Flush markdown buffer
                    if !markdown_buf.is_empty() {
                        blocks.push(Block::Markdown(markdown_buf.clone()));
                        markdown_buf.clear();
                    }
                    blocks.push(Block::SolutionStart);
                    in_solution = true;
                    i += 1;
                }
            }
        } else {
            if !markdown_buf.is_empty() {
                markdown_buf.push('\n');
            }
            markdown_buf.push_str(line);
            i += 1;
        }
    }

    // Flush remaining content
    if in_rustlab && !code_buf.is_empty() {
        // Unclosed code block — treat as code anyway
        blocks.push(Block::Code { source: code_buf, directives: code_directives });
    }
    if !markdown_buf.is_empty() {
        blocks.push(Block::Markdown(markdown_buf));
    }

    blocks
}

/// Directives that appear in the markdown flow (not before code blocks).
enum MarkdownDirective {
    Callout(CalloutKind),
    ExerciseStart,
    SolutionStart,
}

/// Try to parse a trimmed line as a markdown-flow directive.
fn parse_markdown_directive(trimmed: &str) -> Option<MarkdownDirective> {
    match trimmed {
        "<!-- note -->" => Some(MarkdownDirective::Callout(CalloutKind::Note)),
        "<!-- tip -->" => Some(MarkdownDirective::Callout(CalloutKind::Tip)),
        "<!-- warning -->" => Some(MarkdownDirective::Callout(CalloutKind::Warning)),
        "<!-- exercise -->" => Some(MarkdownDirective::ExerciseStart),
        "<!-- solution -->" => Some(MarkdownDirective::SolutionStart),
        _ => None,
    }
}

/// Try to parse a trimmed line as a code-block directive (appears before a ```rustlab fence).
fn parse_code_directive(trimmed: &str) -> Option<CodeDirectiveKind> {
    if trimmed == "<!-- hide -->" {
        return Some(CodeDirectiveKind::Hide);
    }
    if let Some(rest) = trimmed.strip_prefix("<!-- details:") {
        if let Some(title) = rest.strip_suffix("-->") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(CodeDirectiveKind::Details(title.to_string()));
            }
        }
    }
    if let Some(rest) = trimmed.strip_prefix("<!-- grid:") {
        if let Some(n_str) = rest.strip_suffix("-->") {
            if let Ok(n) = n_str.trim().parse::<usize>() {
                if n > 0 {
                    return Some(CodeDirectiveKind::Grid(n));
                }
            }
        }
    }
    None
}

enum CodeDirectiveKind {
    Hide,
    Details(String),
    Grid(usize),
}

/// Scan backward from the tail of the markdown buffer to collect stacked
/// code-block directives. Matched lines are removed from the buffer.
fn extract_code_directives(markdown_buf: &mut String) -> CodeDirectives {
    let mut directives = CodeDirectives::default();

    // Repeatedly check the last line for a directive
    loop {
        let last_line = match markdown_buf.lines().last() {
            Some(l) => l.trim().to_string(),
            None => break,
        };
        match parse_code_directive(&last_line) {
            Some(CodeDirectiveKind::Hide) => directives.hidden = true,
            Some(CodeDirectiveKind::Details(title)) => directives.details = Some(title),
            Some(CodeDirectiveKind::Grid(n)) => directives.grid_cols = Some(n),
            None => break,
        }
        // Remove the directive line from the buffer
        if let Some(pos) = markdown_buf.rfind(&last_line) {
            // Find the start of this line (including preceding newline)
            let start = if pos > 0 && markdown_buf.as_bytes().get(pos - 1) == Some(&b'\n') {
                pos - 1
            } else {
                pos
            };
            markdown_buf.truncate(start);
        }
    }

    // Trim trailing whitespace left behind
    let trimmed_len = markdown_buf.trim_end().len();
    markdown_buf.truncate(trimmed_len);

    directives
}

/// Strip optional YAML frontmatter delimited by `---` lines.
fn strip_frontmatter(src: &str) -> &str {
    let trimmed = src.trim_start();
    if !trimmed.starts_with("---") {
        return src;
    }
    // Find the closing `---`
    let after_open = &trimmed[3..];
    // Skip to end of first `---` line
    let rest = match after_open.find('\n') {
        Some(pos) => &after_open[pos + 1..],
        None => return src, // just `---` with nothing after
    };
    // Find closing `---`
    for (i, line) in rest.lines().enumerate() {
        if line.trim() == "---" {
            // Return everything after the closing ---
            let consumed: usize = rest.lines().take(i + 1)
                .map(|l| l.len() + 1) // +1 for newline
                .sum();
            return &rest[consumed..];
        }
    }
    // No closing --- found, don't strip anything
    src
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_code(block: &Block, expected_src: &str, expected_hidden: bool) -> bool {
        matches!(block, Block::Code { source, directives } if source == expected_src && directives.hidden == expected_hidden)
    }

    #[test]
    fn simple_notebook() {
        let src = "# Title\n\nSome text.\n\n```rustlab\nx = 1:10\n```\n\nMore text.";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Title")));
        assert!(is_code(&blocks[1], "x = 1:10", false));
        assert!(matches!(&blocks[2], Block::Markdown(s) if s.contains("More text")));
    }

    #[test]
    fn frontmatter_stripped() {
        let src = "---\ntitle: Test\n---\n# Heading\n";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Heading")));
    }

    #[test]
    fn other_fences_are_markdown() {
        let src = "```python\nprint('hi')\n```\n";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Markdown(_)));
    }

    #[test]
    fn multiple_code_blocks() {
        let src = "Text\n\n```rustlab\na = 1\n```\n\nMiddle\n\n```rustlab\nb = 2\n```\n\nEnd";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 5);
        assert!(matches!(&blocks[0], Block::Markdown(_)));
        assert!(is_code(&blocks[1], "a = 1", false));
        assert!(matches!(&blocks[2], Block::Markdown(_)));
        assert!(is_code(&blocks[3], "b = 2", false));
        assert!(matches!(&blocks[4], Block::Markdown(_)));
    }

    #[test]
    fn hide_directive() {
        let src = "Setup:\n\n<!-- hide -->\n```rustlab\nx = 42\n```\n\nVisible:\n\n```rustlab\ndisp(x)\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 4);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Setup:")));
        assert!(is_code(&blocks[1], "x = 42", true));
        assert!(matches!(&blocks[2], Block::Markdown(s) if s.contains("Visible:")));
        assert!(is_code(&blocks[3], "disp(x)", false));
    }

    #[test]
    fn empty_input() {
        let blocks = parse_notebook("");
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn markdown_only() {
        let src = "# Title\n\nJust prose, no code.";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Just prose")));
    }

    #[test]
    fn code_only() {
        let src = "```rustlab\nx = 1\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(is_code(&blocks[0], "x = 1", false));
    }

    #[test]
    fn empty_code_block() {
        let src = "```rustlab\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Code { source, .. } if source.is_empty()));
    }

    #[test]
    fn unclosed_code_block() {
        let src = "```rustlab\nx = 1\ny = 2";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Code { source, .. } if source.contains("x = 1") && source.contains("y = 2")));
    }

    #[test]
    fn consecutive_code_blocks() {
        let src = "```rustlab\na = 1\n```\n```rustlab\nb = 2\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Code { source, .. } if source == "a = 1"));
        assert!(matches!(&blocks[1], Block::Code { source, .. } if source == "b = 2"));
    }

    #[test]
    fn hide_directive_strips_from_markdown() {
        let src = "Intro\n\n<!-- hide -->\n```rustlab\nx = 1\n```";
        let blocks = parse_notebook(src);
        if let Block::Markdown(md) = &blocks[0] {
            assert!(!md.contains("<!-- hide -->"), "directive should be stripped: {md:?}");
        }
    }

    #[test]
    fn hide_at_start_of_file() {
        let src = "<!-- hide -->\n```rustlab\nsetup_code\n```\n\nVisible text";
        let blocks = parse_notebook(src);
        assert!(matches!(&blocks[0], Block::Code { directives, .. } if directives.hidden));
    }

    #[test]
    fn multiple_hide_directives() {
        let src = "<!-- hide -->\n```rustlab\na = 1\n```\n\n<!-- hide -->\n```rustlab\nb = 2\n```\n\n```rustlab\nc = 3\n```";
        let blocks = parse_notebook(src);
        assert!(matches!(&blocks[0], Block::Code { directives, .. } if directives.hidden));
        assert!(matches!(&blocks[1], Block::Code { directives, .. } if directives.hidden));
        assert!(matches!(&blocks[2], Block::Code { directives, .. } if !directives.hidden));
    }

    #[test]
    fn rustlab_fence_with_trailing_text() {
        let src = "```rustlab ignore\nx = 1\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Code { source, .. } if source == "x = 1"));
    }

    #[test]
    fn frontmatter_no_closing() {
        let src = "---\ntitle: Test\nno closing\n# Heading";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        if let Block::Markdown(md) = &blocks[0] {
            assert!(md.contains("---"), "should preserve content when frontmatter unclosed");
        }
    }

    #[test]
    fn multiline_code_block() {
        let src = "```rustlab\nline1\nline2\nline3\n```";
        let blocks = parse_notebook(src);
        assert!(matches!(&blocks[0], Block::Code { source, .. } if source == "line1\nline2\nline3"));
    }

    // ── New directive tests ──

    #[test]
    fn details_directive() {
        let src = "<!-- details: Show Plots -->\n```rustlab\nx = 1\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Code { directives, .. }
            if directives.details.as_deref() == Some("Show Plots")));
    }

    #[test]
    fn grid_directive() {
        let src = "<!-- grid: 3 -->\n```rustlab\nx = 1\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Code { directives, .. }
            if directives.grid_cols == Some(3)));
    }

    #[test]
    fn stacked_directives() {
        let src = "<!-- hide -->\n<!-- grid: 2 -->\n<!-- details: Gallery -->\n```rustlab\nx = 1\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 1);
        if let Block::Code { directives, .. } = &blocks[0] {
            assert!(directives.hidden);
            assert_eq!(directives.grid_cols, Some(2));
            assert_eq!(directives.details.as_deref(), Some("Gallery"));
        } else {
            panic!("expected Code block");
        }
    }

    #[test]
    fn callout_note() {
        let src = "Intro\n\n<!-- note -->\nThis is a note.\n\nMore text.";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Intro")));
        assert!(matches!(&blocks[1], Block::Callout { kind: CalloutKind::Note, content }
            if content == "This is a note."));
        assert!(matches!(&blocks[2], Block::Markdown(s) if s.contains("More text")));
    }

    #[test]
    fn callout_multiline_with_close() {
        let src = "<!-- tip -->\nFirst line.\nSecond line.\n<!-- /tip -->\n\nAfter.";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Callout { kind: CalloutKind::Tip, content }
            if content.contains("First line.") && content.contains("Second line.")));
        assert!(matches!(&blocks[1], Block::Markdown(s) if s.contains("After")));
    }

    #[test]
    fn callout_ends_at_heading() {
        let src = "<!-- warning -->\nDanger!\n# Next Section";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Callout { kind: CalloutKind::Warning, content }
            if content == "Danger!"));
        assert!(matches!(&blocks[1], Block::Markdown(s) if s.contains("Next Section")));
    }

    #[test]
    fn exercise_and_solution() {
        let src = "<!-- exercise -->\nWhat is 2+2?\n\n<!-- solution -->\nThe answer is 4.\n";
        let blocks = parse_notebook(src);
        assert!(matches!(&blocks[0], Block::ExerciseStart));
        assert!(matches!(&blocks[1], Block::Markdown(s) if s.contains("What is 2+2?")));
        assert!(matches!(&blocks[2], Block::SolutionStart));
        assert!(matches!(&blocks[3], Block::Markdown(s) if s.contains("answer is 4")));
    }

    #[test]
    fn exercise_with_code_in_solution() {
        let src = "<!-- exercise -->\nCompute x.\n\n<!-- solution -->\nHere it is:\n\n```rustlab\nx = 42\n```";
        let blocks = parse_notebook(src);
        assert!(matches!(&blocks[0], Block::ExerciseStart));
        assert!(matches!(&blocks[1], Block::Markdown(s) if s.contains("Compute x")));
        assert!(matches!(&blocks[2], Block::SolutionStart));
        // Solution contains markdown + code
        assert!(matches!(&blocks[3], Block::Markdown(s) if s.contains("Here it is")));
        assert!(matches!(&blocks[4], Block::Code { source, .. } if source == "x = 42"));
    }
}
