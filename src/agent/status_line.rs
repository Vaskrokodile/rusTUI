//! The `StatusLine` widget: a single-row status bar for agent harnesses.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A single-row status bar showing model name, token count, cost, and a
/// right-aligned segment (e.g. mode or key hints).
pub struct StatusLine {
    /// Left-aligned text (e.g. `gpt-5 · 12.3k tokens`).
    pub left: String,
    /// Right-aligned text (e.g. `NORMAL · ^C to interrupt`).
    pub right: String,
    /// Background color.
    pub bg: Color,
    /// Text color.
    pub fg: Color,
    /// Flex properties.
    pub flex: FlexProps,
}

impl StatusLine {
    /// Construct a status line.
    pub fn new(left: impl Into<String>, right: impl Into<String>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
            bg: Color::palette256(238),
            fg: Color::palette256(250),
            flex: FlexProps::row(),
        }
    }

    /// Set the left text.
    #[must_use]
    pub fn left(mut self, s: impl Into<String>) -> Self {
        self.left = s.into();
        self
    }

    /// Set the right text.
    #[must_use]
    pub fn right(mut self, s: impl Into<String>) -> Self {
        self.right = s.into();
        self
    }

    /// Set the background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    /// Set the foreground color.
    #[must_use]
    pub fn fg(mut self, c: Color) -> Self {
        self.fg = c;
        self
    }
}

impl Widget for StatusLine {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        node.height = Length::Fixed(1.0);
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, .. } = ctx.rect;
        if w == 0 {
            return;
        }
        let style = Style::empty().fg(self.fg).bg(self.bg);
        // Fill the row.
        ctx.buffer.fill_rect(Rect::new(x, y, w, 1), self.bg);
        // Left text.
        let left = truncate_to_width(&self.left, w as usize);
        ctx.buffer.print(x, y, &left, style);
        // Right text, right-aligned.
        let right_w = crate::unicode::str_width(&self.right);
        if right_w < w as usize {
            let rx = x + w - right_w as u16;
            ctx.buffer.print(rx, y, &self.right, style);
        } else {
            let right = truncate_to_width(&self.right, w as usize);
            ctx.buffer.print(x, y, &right, style);
        }
    }
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut width = 0usize;
    let mut out = String::new();
    for g in crate::unicode::graphemes(s) {
        let gw = crate::unicode::grapheme_width(g);
        if width + gw > max_width {
            out.push('…');
            break;
        }
        out.push_str(g);
        width += gw;
    }
    out
}
