//! The `Input` widget: a single-line text input with a cursor.
//!
//! The actual input state (text buffer, cursor position) lives in your
//! [`crate::app::Context`] state — the widget is a stateless view. Pass the
//! current text and cursor column in each frame.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A single-line text input.
pub struct Input {
    /// The current text content.
    pub text: String,
    /// Cursor column (byte offset into `text`; the widget converts to a
    /// display column on render).
    pub cursor: usize,
    /// Base style.
    pub style: Style,
    /// Cursor style (the cell under the cursor).
    pub cursor_style: Style,
    /// Placeholder text shown when `text` is empty.
    pub placeholder: Option<String>,
    /// Flex properties.
    pub flex: FlexProps,
}

impl Input {
    /// Construct an empty input.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            style: Style::empty(),
            cursor_style: Style::empty().bg(Color::WHITE).fg(Color::BLACK),
            placeholder: None,
            flex: FlexProps::column(),
        }
    }

    /// Set the text content.
    #[must_use]
    pub fn text(mut self, s: impl Into<String>) -> Self {
        self.text = s.into();
        self
    }

    /// Set the cursor byte offset.
    #[must_use]
    pub fn cursor(mut self, c: usize) -> Self {
        self.cursor = c.min(self.text.len());
        self
    }

    /// Set the base style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set the cursor style.
    #[must_use]
    pub fn cursor_style(mut self, s: Style) -> Self {
        self.cursor_style = s;
        self
    }

    /// Set placeholder text.
    #[must_use]
    pub fn placeholder(mut self, s: impl Into<String>) -> Self {
        self.placeholder = Some(s.into());
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Input {
    fn layout(&self) -> LayoutNode {
        self.flex.to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        let row = y;
        if self.text.is_empty() {
            if let Some(ph) = &self.placeholder {
                let ph_style = self
                    .style
                    .fg(Color::TRANSPARENT)
                    .over(Style::empty().fg(Color::palette256(8)));
                let mut cx = x;
                for g in crate::unicode::graphemes(ph) {
                    let gw = crate::unicode::grapheme_width(g);
                    if gw == 0 || cx + gw as u16 > x + w {
                        continue;
                    }
                    ctx.buffer.print(cx, row, g, ph_style);
                    cx += gw as u16;
                }
            }
            // Draw cursor at start.
            ctx.buffer.print(x, row, " ", self.cursor_style);
            return;
        }
        // Render text.
        let mut cx = x;
        let mut cursor_display_col = 0usize;
        let mut byte_offset = 0usize;
        for g in crate::unicode::graphemes(&self.text) {
            let gw = crate::unicode::grapheme_width(g);
            if byte_offset == self.cursor {
                cursor_display_col = (cx - x) as usize;
            }
            if cx + gw as u16 <= x + w {
                ctx.buffer.print(cx, row, g, self.style);
            }
            cx += gw as u16;
            byte_offset += g.len();
        }
        if byte_offset == self.cursor {
            cursor_display_col = (cx - x) as usize;
        }
        // Draw cursor cell.
        let cursor_x = x + cursor_display_col as u16;
        if cursor_x < x + w {
            // Invert the cell under the cursor, or draw a block if at end.
            if let Some(cell) = ctx.buffer.cell(cursor_x, row) {
                if cell.is_blank() {
                    ctx.buffer.print(cursor_x, row, " ", self.cursor_style);
                } else {
                    // Re-print the same grapheme with cursor style.
                    let g = cell.grapheme.clone();
                    ctx.buffer.print(cursor_x, row, &g, self.cursor_style);
                }
            }
        }
    }
}
