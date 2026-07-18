//! The `Text` widget: renders a [`crate::text::Spans`] block with wrapping.

use crate::buffer::Rect;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::text::Spans;
use crate::widgets::base::{PaintCtx, Widget};

/// A widget that renders a [`Spans`] block.
///
/// Text wraps at the right edge of its assigned rect. Set
/// [`Text::wrap`] to `false` to clip instead of wrapping.
pub struct Text {
    /// The content to render.
    pub content: Spans,
    /// Base style applied to every span (span styles compose over this).
    pub style: Style,
    /// Whether to wrap long lines. Default: `true`.
    pub wrap: bool,
    /// Flex properties for this widget's box.
    pub flex: FlexProps,
}

impl Text {
    /// Construct a text widget from anything convertible to [`Spans`].
    pub fn new(content: impl Into<Spans>) -> Self {
        Self {
            content: content.into(),
            style: Style::empty(),
            wrap: true,
            flex: FlexProps::column(),
        }
    }

    /// Set the base style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set foreground color (shorthand for `.style(Style::empty().fg(c))`).
    #[must_use]
    pub fn fg(mut self, c: crate::color::Color) -> Self {
        self.style = self.style.fg(c);
        self
    }

    /// Set background color.
    #[must_use]
    pub fn bg(mut self, c: crate::color::Color) -> Self {
        self.style = self.style.bg(c);
        self
    }

    /// Enable or disable line wrapping.
    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Widget for Text {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        // Text wants to be at least as wide as its longest line, but flex
        // will shrink it. Default to auto sizing.
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        let mut cx = x;
        let mut cy = y;
        for span in &self.content.spans {
            let style = span.style.over(self.style);
            for g in crate::unicode::graphemes(&span.text) {
                let gw = crate::unicode::grapheme_width(g);
                if gw == 0 {
                    continue;
                }
                if self.wrap && cx + gw as u16 > x + w {
                    cx = x;
                    cy += 1;
                    if cy >= y + h {
                        return;
                    }
                }
                if cy >= y + h {
                    return;
                }
                if cx + gw as u16 <= x + w {
                    ctx.buffer.print(cx, cy, g, style);
                }
                cx += gw as u16;
            }
        }
    }
}
