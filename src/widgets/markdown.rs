//! The `Markdown` widget: renders markdown-formatted text with styled spans.
//!
//! Supports a useful subset of CommonMark:
//! - Headings (H1–H6) with different sizes/styles
//! - Bold (**text**), italic (*text*), inline code (`code`)
//! - Code blocks (``` and indented)
//! - Blockquotes (> text)
//! - Unordered lists (- or *)
//! - Ordered lists (1. 2. etc.)
//! - Horizontal rules (---)
//! - Paragraphs with word wrapping
//! - Links rendered as underlined text
//!
//! ## Example
//!
//! ```
//! use rustui::Markdown;
//! use rustui::widgets::base::Widget;
//!
//! let md = Markdown::new("# Hello\n\nThis is **bold** and *italic*.");
//! ```

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::text::{Span, Spans};
use crate::widgets::base::{PaintCtx, Widget};
use crate::wrap::{self, Align as WrapAlign};

/// A markdown block element.
#[derive(Clone, Debug)]
enum Block {
    /// Heading (level 1-6, content).
    Heading(u8, Spans),
    /// Paragraph.
    Paragraph(Spans),
    /// Code block (language, content).
    Code(Option<String>, String),
    /// Blockquote (content blocks).
    Blockquote(Vec<Block>),
    /// Unordered list item (content).
    ListItem(Spans),
    /// Ordered list item (number, content).
    OrderedItem(u32, Spans),
    /// Horizontal rule.
    HorizontalRule,
}

/// A markdown rendering widget.
pub struct Markdown {
    /// The raw markdown source.
    pub source: String,
    /// Base text style.
    pub style: Style,
    /// Heading styles (indexed by level 1-6).
    pub heading_styles: [Style; 6],
    /// Code block style.
    pub code_style: Style,
    /// Code block background.
    pub code_bg: Color,
    /// Blockquote style.
    pub quote_style: Style,
    /// Link style.
    pub link_style: Style,
    /// List bullet character.
    pub bullet: &'static str,
    /// List bullet style.
    pub bullet_style: Style,
    /// Horizontal rule style.
    pub hr_style: Style,
    /// Text alignment.
    pub alignment: WrapAlign,
    /// Scroll offset (lines from top).
    pub scroll: u16,
    /// Flex grow.
    pub grow: f32,
    /// Background color.
    pub bg: Color,
}

impl Markdown {
    /// Construct a markdown widget from a string.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            style: Style::empty().fg(Color::rgb(220, 220, 220)),
            heading_styles: [
                Style::empty()
                    .fg(Color::rgb(255, 255, 255))
                    .bold()
                    .underline(),
                Style::empty().fg(Color::rgb(240, 240, 240)).bold(),
                Style::empty().fg(Color::rgb(220, 220, 220)).bold(),
                Style::empty().fg(Color::rgb(200, 200, 200)).bold(),
                Style::empty().fg(Color::rgb(180, 180, 180)).bold(),
                Style::empty().fg(Color::rgb(160, 160, 160)).bold(),
            ],
            code_style: Style::empty().fg(Color::rgb(180, 220, 180)),
            code_bg: Color::rgb(30, 35, 40),
            quote_style: Style::empty().fg(Color::rgb(150, 150, 170)).italic(),
            link_style: Style::empty().fg(Color::rgb(100, 180, 255)).underline(),
            bullet: "•",
            bullet_style: Style::empty().fg(Color::rgb(200, 200, 200)),
            hr_style: Style::empty().fg(Color::rgb(80, 80, 80)),
            alignment: WrapAlign::Left,
            scroll: 0,
            grow: 1.0,
            bg: Color::TRANSPARENT,
        }
    }

    /// Set the base style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set the scroll offset.
    #[must_use]
    pub fn scroll(mut self, s: u16) -> Self {
        self.scroll = s;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }

    /// Set background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    /// Set text alignment.
    #[must_use]
    pub fn alignment(mut self, a: WrapAlign) -> Self {
        self.alignment = a;
        self
    }

    /// Parse the markdown source into blocks.
    fn parse_blocks(&self) -> Vec<Block> {
        parse_markdown(&self.source)
    }

    /// Render blocks into lines of Spans, given a width.
    fn render_lines(&self, blocks: &[Block], width: u16) -> Vec<wrap::Line> {
        let mut lines = Vec::new();
        for block in blocks {
            match block {
                Block::Heading(level, content) => {
                    let style = self
                        .heading_styles
                        .get((*level as usize).saturating_sub(1))
                        .copied()
                        .unwrap_or(self.style);
                    let prefixed =
                        Spans::new().push_styled("#".repeat(*level as usize) + " ", style);
                    let mut full = prefixed;
                    for span in &content.spans {
                        full = full.push(Span::styled(span.text.clone(), span.style.over(style)));
                    }
                    let wrapped = wrap::word_wrap(&full, usize::from(width));
                    lines.extend(wrapped);
                    // Add a blank line after heading.
                    lines.push(wrap::Line::default());
                }
                Block::Paragraph(content) => {
                    let wrapped = wrap::word_wrap(content, usize::from(width));
                    lines.extend(wrapped);
                    lines.push(wrap::Line::default());
                }
                Block::Code(lang, code) => {
                    let _ = lang;
                    // Draw code with background and monospace style.
                    for line in code.lines() {
                        let spans = Spans::new().push_styled(line, self.code_style);
                        lines.push(wrap::Line::new(spans));
                    }
                    lines.push(wrap::Line::default());
                }
                Block::Blockquote(inner) => {
                    let inner_lines = self.render_lines(inner, width.saturating_sub(2));
                    for mut line in inner_lines {
                        // Prefix with "> ".
                        let mut prefixed = Spans::new().push_styled("> ", self.quote_style);
                        for span in &line.spans.spans {
                            prefixed = prefixed.push(Span::styled(
                                span.text.clone(),
                                span.style.over(self.quote_style),
                            ));
                        }
                        line.spans = prefixed;
                        lines.push(line);
                    }
                    lines.push(wrap::Line::default());
                }
                Block::ListItem(content) => {
                    let mut prefixed =
                        Spans::new().push_styled(format!("{} ", self.bullet), self.bullet_style);
                    for span in &content.spans {
                        prefixed = prefixed.push(span.clone());
                    }
                    let wrapped = wrap::word_wrap(&prefixed, usize::from(width));
                    lines.extend(wrapped);
                }
                Block::OrderedItem(num, content) => {
                    let mut prefixed =
                        Spans::new().push_styled(format!("{num}. "), self.bullet_style);
                    for span in &content.spans {
                        prefixed = prefixed.push(span.clone());
                    }
                    let wrapped = wrap::word_wrap(&prefixed, usize::from(width));
                    lines.extend(wrapped);
                }
                Block::HorizontalRule => {
                    let rule: String = "─".repeat(usize::from(width));
                    lines.push(wrap::Line::new(
                        Spans::new().push_styled(rule, self.hr_style),
                    ));
                    lines.push(wrap::Line::default());
                }
            }
        }
        lines
    }
}

impl Widget for Markdown {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        let mut node = props.to_node(Vec::new());
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }

        let blocks = self.parse_blocks();
        let lines = self.render_lines(&blocks, w);

        let start_line = usize::from(self.scroll).min(lines.len());
        let visible_lines = &lines[start_line..];
        let max_visible = usize::from(h);

        for (row, line) in visible_lines.iter().take(max_visible).enumerate() {
            let cy = y + row as u16;

            // Check if this is a code line (has code_bg).
            // For simplicity, we detect code blocks by checking if the line
            // has the code_style. In a real implementation, we'd track this.
            // For now, just render the line.

            // Apply alignment.
            let aligned = if line.width < usize::from(w) && self.alignment != WrapAlign::Left {
                wrap::align_line(line, usize::from(w), self.alignment)
            } else {
                line.spans.clone()
            };

            let mut cx = x;
            for span in &aligned.spans {
                let style = span.style.over(self.style);
                for g in crate::unicode::graphemes(&span.text) {
                    let gw = crate::unicode::grapheme_width(g);
                    if gw == 0 {
                        continue;
                    }
                    if cx + gw as u16 > x + w {
                        break;
                    }
                    ctx.buffer.print(cx, cy, g, style);
                    cx += gw as u16;
                }
            }
        }
    }
}

/// Parse markdown text into a list of blocks.
fn parse_markdown(source: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines = source.lines().peekable();
    let mut paragraph_text = String::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Check for code block fence.
        if trimmed.starts_with("```") {
            // Flush pending paragraph.
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            let lang = trimmed.strip_prefix("```").unwrap_or("").trim().to_string();
            let lang = if lang.is_empty() { None } else { Some(lang) };
            let mut code = String::new();
            for code_line in lines.by_ref() {
                if code_line.trim().starts_with("```") {
                    break;
                }
                code.push_str(code_line);
                code.push('\n');
            }
            blocks.push(Block::Code(lang, code));
            continue;
        }

        // Check for heading.
        if let Some(rest) = trimmed.strip_prefix("# ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::Heading(1, parse_inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::Heading(2, parse_inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::Heading(3, parse_inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("#### ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::Heading(4, parse_inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("##### ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::Heading(5, parse_inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("###### ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::Heading(6, parse_inline(rest)));
            continue;
        }

        // Check for horizontal rule.
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::HorizontalRule);
            continue;
        }

        // Check for blockquote.
        if let Some(rest) = trimmed.strip_prefix("> ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            // Collect blockquote lines.
            let mut quote_text = rest.to_string();
            while let Some(&next) = lines.peek() {
                if let Some(rest) = next.trim().strip_prefix("> ") {
                    quote_text.push('\n');
                    quote_text.push_str(rest);
                    lines.next();
                } else if next.trim() == ">" {
                    quote_text.push('\n');
                    lines.next();
                } else {
                    break;
                }
            }
            blocks.push(Block::Blockquote(parse_markdown(&quote_text)));
            continue;
        }

        // Check for unordered list item.
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            let rest = &trimmed[2..];
            blocks.push(Block::ListItem(parse_inline(rest)));
            continue;
        }

        // Check for ordered list item.
        if let Some(rest) = parse_ordered_list_item(trimmed) {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            blocks.push(Block::OrderedItem(rest.0, parse_inline(rest.1)));
            continue;
        }

        // Check for indented code block (4+ spaces).
        if line.starts_with("    ") {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            let mut code = line.strip_prefix("    ").unwrap_or("").to_string();
            code.push('\n');
            while let Some(&next) = lines.peek() {
                if next.starts_with("    ") {
                    code.push_str(next.strip_prefix("    ").unwrap_or(""));
                    code.push('\n');
                    lines.next();
                } else if next.trim().is_empty() {
                    lines.next();
                    // Check if next line is also indented.
                    if let Some(&after) = lines.peek() {
                        if after.starts_with("    ") {
                            code.push('\n');
                            continue;
                        }
                    }
                    break;
                } else {
                    break;
                }
            }
            blocks.push(Block::Code(None, code));
            continue;
        }

        // Empty line — paragraph break.
        if trimmed.is_empty() {
            if !paragraph_text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(
                    std::mem::take(&mut paragraph_text).trim_end(),
                )));
            }
            continue;
        }

        // Accumulate paragraph text.
        if !paragraph_text.is_empty() {
            paragraph_text.push(' ');
        }
        paragraph_text.push_str(trimmed);
    }

    // Flush remaining paragraph.
    if !paragraph_text.is_empty() {
        blocks.push(Block::Paragraph(parse_inline(paragraph_text.trim_end())));
    }

    blocks
}

/// Parse an ordered list item prefix (e.g., "1. text") and return (number, rest).
fn parse_ordered_list_item(s: &str) -> Option<(u32, &str)> {
    let dot_pos = s.find(". ")?;
    let num_str = &s[..dot_pos];
    if num_str.chars().all(|c| c.is_ascii_digit()) && !num_str.is_empty() {
        let num: u32 = num_str.parse().ok()?;
        let rest = &s[dot_pos + 2..];
        Some((num, rest))
    } else {
        None
    }
}

/// Parse inline markdown formatting (bold, italic, code, links) into Spans.
fn parse_inline(text: &str) -> Spans {
    let mut spans = Spans::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' | '_' if chars.peek() == Some(&'*') || chars.peek() == Some(&'_') => {
                let marker = ch;
                chars.next(); // consume second * or _
                if !current.is_empty() {
                    spans = spans.push_plain(std::mem::take(&mut current));
                }
                // Read until closing ** or __
                let mut content = String::new();
                let mut found_close = false;
                while let Some(&c) = chars.peek() {
                    if c == marker {
                        chars.next();
                        if chars.peek() == Some(&marker) {
                            chars.next();
                            found_close = true;
                            break;
                        }
                        content.push(c);
                    } else {
                        content.push(c);
                        chars.next();
                    }
                }
                if found_close {
                    spans = spans.push_styled(content, Style::empty().bold());
                } else {
                    spans = spans.push_plain(format!("{marker}{marker}{content}"));
                }
            }
            '*' | '_' => {
                let marker = ch;
                if !current.is_empty() {
                    spans = spans.push_plain(std::mem::take(&mut current));
                }
                let mut content = String::new();
                let mut found_close = false;
                while let Some(&c) = chars.peek() {
                    if c == marker {
                        chars.next();
                        found_close = true;
                        break;
                    }
                    content.push(c);
                    chars.next();
                }
                if found_close {
                    spans = spans.push_styled(content, Style::empty().italic());
                } else {
                    spans = spans.push_plain(format!("{marker}{content}"));
                }
            }
            '`' => {
                if !current.is_empty() {
                    spans = spans.push_plain(std::mem::take(&mut current));
                }
                let mut content = String::new();
                let mut found_close = false;
                while let Some(&c) = chars.peek() {
                    if c == '`' {
                        chars.next();
                        found_close = true;
                        break;
                    }
                    content.push(c);
                    chars.next();
                }
                if found_close {
                    spans =
                        spans.push_styled(content, Style::empty().fg(Color::rgb(180, 220, 180)));
                } else {
                    spans = spans.push_plain(format!("`{content}"));
                }
            }
            '[' => {
                if !current.is_empty() {
                    spans = spans.push_plain(std::mem::take(&mut current));
                }
                let mut link_text = String::new();
                let mut found_bracket = false;
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        found_bracket = true;
                        break;
                    }
                    link_text.push(c);
                    chars.next();
                }
                if found_bracket && chars.peek() == Some(&'(') {
                    chars.next(); // consume (
                                  // Skip the URL until )
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == ')' {
                            break;
                        }
                    }
                    spans = spans.push_styled(
                        link_text,
                        Style::empty().fg(Color::rgb(100, 180, 255)).underline(),
                    );
                } else {
                    spans = spans.push_plain(format!("[{link_text}"));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        spans = spans.push_plain(&current);
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::style::Attr;
    use std::time::Duration;

    fn paint_widget(widget: &impl Widget, w: u16, h: u16) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rects = vec![Rect::new(0, 0, w, h)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        widget.paint(&mut ctx);
        buf
    }

    #[test]
    fn parse_heading() {
        let blocks = parse_markdown("# Hello World");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Heading(level, _) => {
                assert_eq!(*level, 1);
            }
            _ => panic!("Expected heading"),
        }
    }

    #[test]
    fn parse_heading_levels() {
        assert!(matches!(parse_markdown("# H1")[0], Block::Heading(1, _)));
        assert!(matches!(parse_markdown("## H2")[0], Block::Heading(2, _)));
        assert!(matches!(parse_markdown("### H3")[0], Block::Heading(3, _)));
        assert!(matches!(parse_markdown("#### H4")[0], Block::Heading(4, _)));
    }

    #[test]
    fn parse_code_block() {
        let blocks = parse_markdown("```rust\nfn main() {}\n```");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Code(lang, code) => {
                assert_eq!(lang.as_deref(), Some("rust"));
                assert!(code.contains("fn main()"));
            }
            _ => panic!("Expected code block"),
        }
    }

    #[test]
    fn parse_unordered_list() {
        let blocks = parse_markdown("- item 1\n- item 2");
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::ListItem(_)));
        assert!(matches!(&blocks[1], Block::ListItem(_)));
    }

    #[test]
    fn parse_ordered_list() {
        let blocks = parse_markdown("1. first\n2. second");
        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            Block::OrderedItem(num, _) => assert_eq!(*num, 1),
            _ => panic!("Expected ordered list item"),
        }
    }

    #[test]
    fn parse_blockquote() {
        let blocks = parse_markdown("> This is a quote");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Blockquote(_)));
    }

    #[test]
    fn parse_horizontal_rule() {
        let blocks = parse_markdown("---");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::HorizontalRule));
    }

    #[test]
    fn parse_inline_bold() {
        let spans = parse_inline("hello **world** foo");
        assert_eq!(spans.spans.len(), 3);
        assert!(spans.spans[1].style.attr.contains(Attr::BOLD));
        assert_eq!(spans.spans[1].text, "world");
    }

    #[test]
    fn parse_inline_italic() {
        let spans = parse_inline("hello *world* foo");
        assert_eq!(spans.spans.len(), 3);
        assert!(spans.spans[1].style.attr.contains(Attr::ITALIC));
    }

    #[test]
    fn parse_inline_code() {
        let spans = parse_inline("use `cargo` to build");
        assert_eq!(spans.spans.len(), 3);
        assert_eq!(spans.spans[1].text, "cargo");
    }

    #[test]
    fn parse_inline_link() {
        let spans = parse_inline("see [docs](https://example.com) for info");
        assert_eq!(spans.spans.len(), 3);
        assert_eq!(spans.spans[1].text, "docs");
        assert!(spans.spans[1].style.attr.contains(Attr::UNDERLINE));
    }

    #[test]
    fn markdown_renders_heading() {
        let md = Markdown::new("# Hello");
        let buf = paint_widget(&md, 40, 5);
        // Should render "# Hello" on the first line.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "#");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "H");
    }

    #[test]
    fn markdown_renders_bold() {
        let md = Markdown::new("**bold text**");
        let buf = paint_widget(&md, 40, 3);
        // The bold text should be rendered (after the ** prefix).
        let has_bold = (0..40).any(|x| {
            buf.cell(x, 0)
                .is_some_and(|c| c.style.attr.contains(Attr::BOLD))
        });
        assert!(has_bold);
    }

    #[test]
    fn markdown_renders_code_block() {
        let md = Markdown::new("```rust\nfn main() {}\n```");
        let buf = paint_widget(&md, 40, 5);
        // Code should be rendered on line 0.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "f");
    }

    #[test]
    fn markdown_renders_list() {
        let md = Markdown::new("- item 1\n- item 2");
        let buf = paint_widget(&md, 40, 5);
        // First line should have bullet "•".
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "•");
        // Second line should also have bullet.
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "•");
    }

    #[test]
    fn markdown_renders_paragraph() {
        let md = Markdown::new("This is a paragraph.");
        let buf = paint_widget(&md, 40, 3);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "T");
    }
}
