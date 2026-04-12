/// A block in a parsed notebook.
#[derive(Debug, Clone)]
pub enum Block {
    /// Raw markdown text (to be rendered as HTML).
    Markdown(String),
    /// A ```rustlab fenced code block (source to be executed).
    Code {
        source: String,
        /// If true, the source code is hidden in the rendered output
        /// (the block is still executed for its side effects).
        hidden: bool,
    },
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
    let mut code_hidden = false;

    for line in src.lines() {
        let trimmed = line.trim();

        if in_rustlab {
            if trimmed == "```" {
                // End of rustlab code block
                blocks.push(Block::Code { source: code_buf.clone(), hidden: code_hidden });
                code_buf.clear();
                in_rustlab = false;
                code_hidden = false;
            } else {
                if !code_buf.is_empty() {
                    code_buf.push('\n');
                }
                code_buf.push_str(line);
            }
        } else if trimmed == "```rustlab" || trimmed.starts_with("```rustlab ") {
            // Check if the preceding markdown line is a <!-- hide --> directive
            let hide = markdown_buf.lines().last()
                .map(|l| l.trim())
                .map_or(false, |l| l == "<!-- hide -->");
            if hide {
                // Remove the directive from the markdown buffer
                if let Some(pos) = markdown_buf.rfind("<!-- hide -->") {
                    markdown_buf.truncate(pos);
                    // Trim trailing whitespace/newlines left behind
                    let trimmed_len = markdown_buf.trim_end().len();
                    markdown_buf.truncate(trimmed_len);
                }
            }
            // Start of rustlab code block — flush markdown buffer
            if !markdown_buf.is_empty() {
                blocks.push(Block::Markdown(markdown_buf.clone()));
                markdown_buf.clear();
            }
            in_rustlab = true;
            code_hidden = hide;
        } else {
            if !markdown_buf.is_empty() {
                markdown_buf.push('\n');
            }
            markdown_buf.push_str(line);
        }
    }

    // Flush remaining content
    if in_rustlab && !code_buf.is_empty() {
        // Unclosed code block — treat as code anyway
        blocks.push(Block::Code { source: code_buf, hidden: code_hidden });
    }
    if !markdown_buf.is_empty() {
        blocks.push(Block::Markdown(markdown_buf));
    }

    blocks
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

    #[test]
    fn simple_notebook() {
        let src = "# Title\n\nSome text.\n\n```rustlab\nx = 1:10\n```\n\nMore text.";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Title")));
        assert!(matches!(&blocks[1], Block::Code { source, hidden } if source == "x = 1:10" && !hidden));
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
        assert!(matches!(&blocks[1], Block::Code { source, hidden } if source == "a = 1" && !hidden));
        assert!(matches!(&blocks[2], Block::Markdown(_)));
        assert!(matches!(&blocks[3], Block::Code { source, hidden } if source == "b = 2" && !hidden));
        assert!(matches!(&blocks[4], Block::Markdown(_)));
    }

    #[test]
    fn hide_directive() {
        let src = "Setup:\n\n<!-- hide -->\n```rustlab\nx = 42\n```\n\nVisible:\n\n```rustlab\ndisp(x)\n```";
        let blocks = parse_notebook(src);
        assert_eq!(blocks.len(), 4);
        assert!(matches!(&blocks[0], Block::Markdown(s) if s.contains("Setup:")));
        assert!(matches!(&blocks[1], Block::Code { source, hidden } if source == "x = 42" && *hidden));
        assert!(matches!(&blocks[2], Block::Markdown(s) if s.contains("Visible:")));
        assert!(matches!(&blocks[3], Block::Code { source, hidden } if source == "disp(x)" && !hidden));
    }
}
