//! Styled text: a run of graphemes with associated [`Style`]s.
//!
//! This is the unit of content that widgets like [`crate::widgets::Input`],
//! [`crate::agent::StreamingText`], and [`crate::agent::DiffViewer`] consume.

use crate::style::Style;

/// A styled span of text.
#[derive(Clone, Debug, Default)]
pub struct Span {
    /// The text content.
    pub text: compact_str::CompactString,
    /// The style for this span.
    pub style: Style,
}

impl Span {
    /// Construct a plain (default-styled) span.
    pub fn plain(text: impl Into<compact_str::CompactString>) -> Self {
        Self {
            text: text.into(),
            style: Style::empty(),
        }
    }
    /// Construct a styled span.
    pub fn styled(text: impl Into<compact_str::CompactString>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

/// A sequence of [`Span`]s that should be laid out as a single text block.
#[derive(Clone, Debug, Default)]
pub struct Spans {
    /// The spans.
    pub spans: Vec<Span>,
}

impl Spans {
    /// An empty text block.
    pub fn new() -> Self {
        Self::default()
    }
    /// A single plain span.
    pub fn plain(s: impl Into<compact_str::CompactString>) -> Self {
        Self {
            spans: vec![Span::plain(s)],
        }
    }
    /// A single styled span.
    pub fn styled(s: impl Into<compact_str::CompactString>, style: Style) -> Self {
        Self {
            spans: vec![Span::styled(s, style)],
        }
    }
    /// Append a span.
    #[must_use]
    pub fn push(mut self, span: Span) -> Self {
        self.spans.push(span);
        self
    }
    /// Append a plain-text span.
    #[must_use]
    pub fn push_plain(self, s: impl Into<compact_str::CompactString>) -> Self {
        self.push(Span::plain(s))
    }
    /// Append a styled span.
    #[must_use]
    pub fn push_styled(self, s: impl Into<compact_str::CompactString>, style: Style) -> Self {
        self.push(Span::styled(s, style))
    }
    /// Total display width of all spans on a single line (no wrapping).
    pub fn width(&self) -> usize {
        self.spans
            .iter()
            .map(|s| crate::unicode::str_width(&s.text))
            .sum()
    }
    /// Whether the text block is empty.
    pub fn is_empty(&self) -> bool {
        self.spans.iter().all(|s| s.text.is_empty())
    }
    /// Concatenate all spans into a single plain string.
    pub fn to_plain(&self) -> String {
        let mut s = String::new();
        for span in &self.spans {
            s.push_str(&span.text);
        }
        s
    }
}

impl From<&str> for Spans {
    fn from(s: &str) -> Self {
        Self::plain(s)
    }
}

impl From<String> for Spans {
    fn from(s: String) -> Self {
        Self::plain(s)
    }
}
