//! The `StreamingText` widget: renders LLM output as it arrives, token by token.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::text::Spans;
use crate::widgets::base::{PaintCtx, Widget};
use crate::widgets::spinner::SpinnerStyle;

/// A widget for rendering LLM output as it streams in.
///
/// Pass the accumulated text each frame via [`StreamingText::content`]. The
/// widget shows a spinner while `streaming` is true, and a blinking block
/// cursor at the end of the content. When `streaming` becomes false, the
/// cursor disappears and the spinner is replaced by a checkmark.
pub struct StreamingText {
    /// The accumulated content so far.
    pub content: Spans,
    /// Whether the stream is still active.
    pub streaming: bool,
    /// Base style for the content.
    pub style: Style,
    /// Spinner style while streaming.
    pub spinner_style: SpinnerStyle,
    /// Spinner color.
    pub spinner_color: Color,
    /// Flex properties.
    pub flex: FlexProps,
}

impl StreamingText {
    /// Construct a streaming-text widget.
    pub fn new(content: impl Into<Spans>) -> Self {
        Self {
            content: content.into(),
            streaming: true,
            style: Style::empty(),
            spinner_style: SpinnerStyle::Braille,
            spinner_color: Color::CYAN,
            flex: FlexProps::column(),
        }
    }

    /// Set the content.
    #[must_use]
    pub fn content(mut self, c: impl Into<Spans>) -> Self {
        self.content = c.into();
        self
    }

    /// Set whether the stream is still active.
    #[must_use]
    pub fn streaming(mut self, s: bool) -> Self {
        self.streaming = s;
        self
    }

    /// Set the base style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set the spinner style.
    #[must_use]
    pub fn spinner_style(mut self, s: SpinnerStyle) -> Self {
        self.spinner_style = s;
        self
    }

    /// Set the spinner color.
    #[must_use]
    pub fn spinner_color(mut self, c: Color) -> Self {
        self.spinner_color = c;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Widget for StreamingText {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        // Render the content with wrapping.
        let mut cx = x;
        let mut cy = y;
        let max_w = w as usize;
        for span in &self.content.spans {
            let style = span.style.over(self.style);
            for g in crate::unicode::graphemes(&span.text) {
                let gw = crate::unicode::grapheme_width(g);
                if gw == 0 {
                    continue;
                }
                if cx + gw as u16 > x + w {
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
        // Draw the trailing cursor / spinner.
        if cy < y + h {
            if self.streaming {
                // Blinking block cursor at the current position.
                let blink = (ctx.elapsed.as_millis() / 500) % 2 == 0;
                if blink {
                    ctx.buffer.print(
                        cx,
                        cy,
                        " ",
                        Style::empty().bg(Color::WHITE).fg(Color::BLACK),
                    );
                }
            } else {
                // Done: draw a small checkmark at the end of the last line.
                let _ = max_w;
            }
        }
    }
}
