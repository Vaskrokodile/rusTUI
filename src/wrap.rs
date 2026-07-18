//! Word-aware text wrapping and line layout.
//!
//! Converts a [`Spans`] block into a list of lines, each a [`Spans`] that fits
//! within a given width. Supports:
//!
//! - Character-level wrapping (break anywhere)
//! - Word-aware wrapping (break at spaces, never mid-word)
//! - Preserving explicit newlines (`\n`) in the input
//! - Truncation with ellipsis
//! - Text alignment (left, center, right)
//!
//! ## Example
//!
//! ```
//! use rustui::{Spans, wrap};
//!
//! let text = Spans::plain("hello world foo bar baz");
//! let lines = wrap::word_wrap(&text, 10);
//! assert_eq!(lines.len(), 3);
//! assert_eq!(lines[0].to_plain(), "hello");
//! assert_eq!(lines[1].to_plain(), "world foo");
//! ```

use crate::text::{Span, Spans};
use crate::unicode::{grapheme_width, graphemes, str_width};

/// Text alignment within a line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Left-align (default).
    #[default]
    Left,
    /// Center-align.
    Center,
    /// Right-align.
    Right,
}

/// A wrapped line: the spans content plus its display width.
#[derive(Clone, Debug, Default)]
pub struct Line {
    /// The spans content of this line.
    pub spans: Spans,
    /// Display width of this line in terminal columns.
    pub width: usize,
}

impl Line {
    /// Construct a line from spans, computing width.
    pub fn new(spans: Spans) -> Self {
        let width = spans.width();
        Self { spans, width }
    }

    /// Plain text of this line.
    pub fn to_plain(&self) -> String {
        self.spans.to_plain()
    }

    /// Whether this line is empty (no visible content).
    pub fn is_empty(&self) -> bool {
        self.width == 0
    }
}

/// Wrap `text` into lines of at most `width` columns, breaking at word
/// boundaries when possible.
///
/// Explicit `\n` characters in the input force a line break. Words longer
/// than `width` are broken character-by-character.
#[must_use]
pub fn word_wrap(text: &Spans, width: usize) -> Vec<Line> {
    if width == 0 {
        return vec![Line::new(text.clone())];
    }
    let mut lines = Vec::new();
    // First, split on explicit newlines.
    for segment in split_on_newlines(text) {
        wrap_segment_words(&segment, width, &mut lines);
    }
    if lines.is_empty() {
        lines.push(Line::default());
    }
    lines
}

/// Wrap `text` into lines of at most `width` columns, breaking at any
/// character boundary (no word awareness).
#[must_use]
pub fn char_wrap(text: &Spans, width: usize) -> Vec<Line> {
    if width == 0 {
        return vec![Line::new(text.clone())];
    }
    let mut lines = Vec::new();
    for segment in split_on_newlines(text) {
        wrap_segment_chars(&segment, width, &mut lines);
    }
    if lines.is_empty() {
        lines.push(Line::default());
    }
    lines
}

/// Truncate `text` to fit within `width` columns, appending `ellipsis` if
/// truncation occurs.
#[must_use]
pub fn truncate(text: &Spans, width: usize, ellipsis: &str) -> Spans {
    let total = text.width();
    if total <= width {
        return text.clone();
    }
    let ellipsis_width = str_width(ellipsis);
    if width <= ellipsis_width {
        // Not enough room for even the ellipsis — just take what fits.
        let mut result = Spans::new();
        let mut remaining = width;
        for span in &text.spans {
            if remaining == 0 {
                break;
            }
            let span_w = str_width(&span.text);
            if span_w <= remaining {
                result = result.push(span.clone());
                remaining -= span_w;
            } else {
                // Take a prefix of this span that fits.
                let mut taken = String::new();
                for g in graphemes(&span.text) {
                    let gw = grapheme_width(g);
                    if gw > remaining {
                        break;
                    }
                    taken.push_str(g);
                    remaining -= gw;
                }
                if !taken.is_empty() {
                    result = result.push(Span::styled(
                        compact_str::CompactString::from(taken.as_str()),
                        span.style,
                    ));
                }
                break;
            }
        }
        return result;
    }

    let target = width - ellipsis_width;
    let mut result = Spans::new();
    let mut remaining = target;
    for span in &text.spans {
        if remaining == 0 {
            break;
        }
        let span_w = str_width(&span.text);
        if span_w <= remaining {
            result = result.push(span.clone());
            remaining -= span_w;
        } else {
            let mut taken = String::new();
            for g in graphemes(&span.text) {
                let gw = grapheme_width(g);
                if gw > remaining {
                    break;
                }
                taken.push_str(g);
                remaining -= gw;
            }
            if !taken.is_empty() {
                result = result.push(Span::styled(
                    compact_str::CompactString::from(taken.as_str()),
                    span.style,
                ));
            }
            break;
        }
    }
    result = result.push_plain(ellipsis);
    result
}

/// Pad a line to `width` columns according to `align`, filling with spaces.
#[must_use]
pub fn align_line(line: &Line, width: usize, align: Align) -> Spans {
    if line.width >= width {
        return line.spans.clone();
    }
    let pad = width - line.width;
    match align {
        Align::Left => line.spans.clone(),
        Align::Center => {
            let left = pad / 2;
            let right = pad - left;
            let mut result = Spans::new().push_plain(" ".repeat(left));
            for span in &line.spans.spans {
                result = result.push(span.clone());
            }
            result.push_plain(" ".repeat(right))
        }
        Align::Right => {
            let mut result = Spans::new().push_plain(" ".repeat(pad));
            for span in &line.spans.spans {
                result = result.push(span.clone());
            }
            result
        }
    }
}

/// Split a [`Spans`] block on explicit `\n` characters, returning a vec of
/// segments (each without the newline).
fn split_on_newlines(text: &Spans) -> Vec<Spans> {
    let mut segments = Vec::new();
    let mut current = Spans::new();
    for span in &text.spans {
        let mut part = String::new();
        for ch in span.text.chars() {
            if ch == '\n' {
                if !part.is_empty() {
                    current = current.push(Span::styled(
                        compact_str::CompactString::from(part.as_str()),
                        span.style,
                    ));
                    part.clear();
                }
                segments.push(std::mem::take(&mut current));
            } else {
                part.push(ch);
            }
        }
        if !part.is_empty() {
            current = current.push(Span::styled(
                compact_str::CompactString::from(part.as_str()),
                span.style,
            ));
        }
    }
    segments.push(current);
    segments
}

/// Word-wrap a single line segment (no embedded newlines).
fn wrap_segment_words(segment: &Spans, width: usize, lines: &mut Vec<Line>) {
    // Tokenize into words (with their styles) and spaces.
    let tokens = tokenize_words(segment);
    if tokens.is_empty() {
        lines.push(Line::default());
        return;
    }

    let mut current_spans = Spans::new();
    let mut current_width = 0usize;
    let mut pending_spaces = 0usize;

    for token in tokens {
        match token {
            Token::Word(spans, w) => {
                // If there are pending spaces, try to add them before the word.
                if pending_spaces > 0 {
                    if current_width + pending_spaces + w > width && current_width > 0 {
                        // Flush current line (without the trailing spaces).
                        lines.push(Line {
                            spans: std::mem::take(&mut current_spans),
                            width: current_width,
                        });
                        current_width = 0;
                    } else if current_width > 0 {
                        current_spans = current_spans.push_plain(" ".repeat(pending_spaces));
                        current_width += pending_spaces;
                    }
                    pending_spaces = 0;
                }

                if w > width {
                    // Word is wider than the line — char-break it.
                    if current_width > 0 {
                        lines.push(Line {
                            spans: std::mem::take(&mut current_spans),
                            width: current_width,
                        });
                        current_width = 0;
                    }
                    let sub_lines = char_wrap(&spans, width);
                    let last_idx = sub_lines.len() - 1;
                    for (i, sl) in sub_lines.into_iter().enumerate() {
                        if i < last_idx {
                            lines.push(sl);
                        } else {
                            current_spans = sl.spans;
                            current_width = sl.width;
                        }
                    }
                } else if current_width + w > width {
                    // Word doesn't fit on current line — flush and start new.
                    lines.push(Line {
                        spans: std::mem::take(&mut current_spans),
                        width: current_width,
                    });
                    for span in &spans.spans {
                        current_spans = current_spans.push(span.clone());
                    }
                    current_width = w;
                } else {
                    for span in &spans.spans {
                        current_spans = current_spans.push(span.clone());
                    }
                    current_width += w;
                }
            }
            Token::Spaces(count) => {
                pending_spaces += count;
            }
        }
    }

    if current_width > 0 || !current_spans.spans.is_empty() {
        lines.push(Line {
            spans: current_spans,
            width: current_width,
        });
    }
}

/// Character-wrap a single line segment (no embedded newlines).
fn wrap_segment_chars(segment: &Spans, width: usize, lines: &mut Vec<Line>) {
    let mut current_spans = Spans::new();
    let mut current_width = 0usize;

    for span in &segment.spans {
        for g in graphemes(&span.text) {
            let gw = grapheme_width(g);
            if gw == 0 {
                continue;
            }
            if current_width + gw > width && current_width > 0 {
                lines.push(Line {
                    spans: std::mem::take(&mut current_spans),
                    width: current_width,
                });
                current_width = 0;
            }
            current_spans = current_spans.push(Span::styled(
                compact_str::CompactString::from(g),
                span.style,
            ));
            current_width += gw;
        }
    }

    if current_width > 0 || !current_spans.spans.is_empty() {
        lines.push(Line {
            spans: current_spans,
            width: current_width,
        });
    }
}

/// A token in word tokenization.
enum Token {
    /// A word (no spaces) with its styled spans and display width.
    Word(Spans, usize),
    /// One or more spaces (count, display width = count).
    Spaces(usize),
}

/// Tokenize a [`Spans`] block into words and space-runs.
fn tokenize_words(spans: &Spans) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut word_spans = Spans::new();
    let mut word_width = 0usize;
    let mut space_count = 0usize;

    for span in &spans.spans {
        let mut part = String::new();
        for ch in span.text.chars() {
            if ch == ' ' || ch == '\t' {
                if !part.is_empty() {
                    let w = str_width(&part);
                    word_spans = word_spans.push(Span::styled(
                        compact_str::CompactString::from(part.as_str()),
                        span.style,
                    ));
                    word_width += w;
                    part.clear();
                }
                if word_width > 0 || !word_spans.spans.is_empty() {
                    tokens.push(Token::Word(std::mem::take(&mut word_spans), word_width));
                    word_width = 0;
                }
                space_count += 1;
            } else {
                if space_count > 0 {
                    tokens.push(Token::Spaces(space_count));
                    space_count = 0;
                }
                part.push(ch);
            }
        }
        if !part.is_empty() {
            let w = str_width(&part);
            word_spans = word_spans.push(Span::styled(
                compact_str::CompactString::from(part.as_str()),
                span.style,
            ));
            word_width += w;
        }
    }

    if word_width > 0 || !word_spans.spans.is_empty() {
        tokens.push(Token::Word(word_spans, word_width));
    }
    if space_count > 0 {
        tokens.push(Token::Spaces(space_count));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    #[test]
    fn word_wrap_basic() {
        let text = Spans::plain("hello world foo bar baz");
        let lines = word_wrap(&text, 10);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].to_plain(), "hello");
        assert_eq!(lines[1].to_plain(), "world foo");
        assert_eq!(lines[2].to_plain(), "bar baz");
    }

    #[test]
    fn word_wrap_preserves_newlines() {
        let text = Spans::plain("hello\nworld\nfoo");
        let lines = word_wrap(&text, 100);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].to_plain(), "hello");
        assert_eq!(lines[1].to_plain(), "world");
        assert_eq!(lines[2].to_plain(), "foo");
    }

    #[test]
    fn word_wrap_long_word_breaks() {
        let text = Spans::plain("abcdefghij");
        let lines = word_wrap(&text, 4);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].to_plain(), "abcd");
        assert_eq!(lines[1].to_plain(), "efgh");
        assert_eq!(lines[2].to_plain(), "ij");
    }

    #[test]
    fn word_wrap_empty() {
        let text = Spans::plain("");
        let lines = word_wrap(&text, 10);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].is_empty());
    }

    #[test]
    fn char_wrap_basic() {
        let text = Spans::plain("hello world");
        let lines = char_wrap(&text, 5);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].to_plain(), "hello");
        assert_eq!(lines[1].to_plain(), " worl");
        assert_eq!(lines[2].to_plain(), "d");
    }

    #[test]
    fn truncate_short() {
        let text = Spans::plain("hello");
        let result = truncate(&text, 10, "...");
        assert_eq!(result.to_plain(), "hello");
    }

    #[test]
    fn truncate_long() {
        let text = Spans::plain("hello world foo bar");
        let result = truncate(&text, 10, "...");
        assert_eq!(result.to_plain(), "hello w...");
        assert_eq!(result.width(), 10);
    }

    #[test]
    fn truncate_very_short() {
        let text = Spans::plain("hello world");
        let result = truncate(&text, 2, "...");
        assert_eq!(result.to_plain(), "he");
    }

    #[test]
    fn align_left() {
        let line = Line::new(Spans::plain("hi"));
        let result = align_line(&line, 10, Align::Left);
        assert_eq!(result.to_plain(), "hi");
    }

    #[test]
    fn align_center() {
        let line = Line::new(Spans::plain("hi"));
        let result = align_line(&line, 10, Align::Center);
        assert_eq!(result.to_plain(), "    hi    ");
    }

    #[test]
    fn align_right() {
        let line = Line::new(Spans::plain("hi"));
        let result = align_line(&line, 10, Align::Right);
        assert_eq!(result.to_plain(), "        hi");
    }

    #[test]
    fn word_wrap_with_styles() {
        let text = Spans::plain("hello ").push_styled("world", Style::empty().bold());
        let lines = word_wrap(&text, 5);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].to_plain(), "hello");
        assert_eq!(lines[1].to_plain(), "world");
        // Second line should be bold.
        assert!(lines[1].spans.spans[0]
            .style
            .attr
            .contains(crate::style::Attr::BOLD));
    }

    #[test]
    fn word_wrap_multiple_spaces() {
        let text = Spans::plain("hello  world");
        let lines = word_wrap(&text, 100);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].to_plain(), "hello  world");
    }

    #[test]
    fn word_wrap_preserves_styles_across_breaks() {
        let text = Spans::plain("hello ").push_styled("world foo", Style::empty().bold());
        let lines = word_wrap(&text, 8);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].to_plain(), "hello");
        assert_eq!(lines[1].to_plain(), "world");
        assert_eq!(lines[2].to_plain(), "foo");
        // The "world" part should be bold.
        assert!(lines[1].spans.spans[0]
            .style
            .attr
            .contains(crate::style::Attr::BOLD));
        // The "foo" part should also be bold.
        assert!(lines[2].spans.spans[0]
            .style
            .attr
            .contains(crate::style::Attr::BOLD));
    }
}
