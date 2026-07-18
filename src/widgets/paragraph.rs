//! The `Paragraph` widget: renders multi-line text with wrapping, alignment,
//! and scrolling.
//!
//! This is the primary text-display widget. Unlike [`crate::widgets::Text`],
//! which does simple character wrapping, `Paragraph` uses the
//! [`crate::wrap`] module for word-aware wrapping, supports text alignment,
//! and can scroll through content larger than its allocated area.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::text::Spans;
use crate::widgets::base::{PaintCtx, Widget};
use crate::wrap::{self, Align as WrapAlign};

/// A multi-line text widget with word wrapping, alignment, and scrolling.
pub struct Paragraph {
    /// The content to render.
    pub content: Spans,
    /// Base style applied to every span.
    pub style: Style,
    /// Text alignment within the widget.
    pub alignment: WrapAlign,
    /// Whether to use word-aware wrapping (default: true).
    /// If false, uses character-level wrapping.
    pub word_wrap: bool,
    /// Scroll offset: number of lines scrolled from the top.
    pub scroll: u16,
    /// Flex properties.
    pub flex: FlexProps,
}

impl Paragraph {
    /// Construct a paragraph from anything convertible to [`Spans`].
    pub fn new(content: impl Into<Spans>) -> Self {
        Self {
            content: content.into(),
            style: Style::empty(),
            alignment: WrapAlign::Left,
            word_wrap: true,
            scroll: 0,
            flex: FlexProps::column(),
        }
    }

    /// Set the base style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set foreground color.
    #[must_use]
    pub fn fg(mut self, c: Color) -> Self {
        self.style = self.style.fg(c);
        self
    }

    /// Set background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.style = self.style.bg(c);
        self
    }

    /// Set text alignment.
    #[must_use]
    pub fn alignment(mut self, a: WrapAlign) -> Self {
        self.alignment = a;
        self
    }

    /// Enable or disable word-aware wrapping.
    #[must_use]
    pub fn word_wrap(mut self, enable: bool) -> Self {
        self.word_wrap = enable;
        self
    }

    /// Set the scroll offset (lines from top).
    #[must_use]
    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }

    /// Compute the wrapped lines for this paragraph at the given width.
    fn compute_lines(&self, width: u16) -> Vec<wrap::Line> {
        if width == 0 {
            return Vec::new();
        }
        if self.word_wrap {
            wrap::word_wrap(&self.content, usize::from(width))
        } else {
            wrap::char_wrap(&self.content, usize::from(width))
        }
    }
}

impl Widget for Paragraph {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        let lines = self.compute_lines(w);
        let total_lines = lines.len();

        // Apply scroll offset.
        let start_line = usize::from(self.scroll).min(total_lines);
        let visible_lines = &lines[start_line..];
        let max_visible = usize::from(h);

        for (row, line) in visible_lines.iter().take(max_visible).enumerate() {
            let cy = y + row as u16;
            // Apply alignment within the line width.
            let aligned = if line.width < usize::from(w) && self.alignment != WrapAlign::Left {
                wrap::align_line(line, usize::from(w), self.alignment)
            } else {
                line.spans.clone()
            };

            // Render each span in the line.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::color::Color;
    use crate::style::Attr;
    use std::time::Duration;

    #[test]
    fn paragraph_wraps_text() {
        let para = Paragraph::new("hello world foo bar baz qux").word_wrap(true);
        let mut buf = Buffer::empty(10, 5);
        let rects = vec![Rect::new(0, 0, 10, 5)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        para.paint(&mut ctx);
        // "hello" on row 0, "world foo" on row 1, etc.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "h");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "w");
    }

    #[test]
    fn paragraph_scroll() {
        let para = Paragraph::new("line1\nline2\nline3\nline4\nline5").scroll(2);
        let mut buf = Buffer::empty(10, 2);
        let rects = vec![Rect::new(0, 0, 10, 2)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        para.paint(&mut ctx);
        // Scrolled 2 lines, so row 0 should show "line3"
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "l");
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "l");
        // Check that it's "line3" not "line1"
        let row0: String = (0..5)
            .map(|i| {
                buf.cell(i, 0)
                    .map(|c| c.grapheme.to_string())
                    .unwrap_or_default()
            })
            .collect();
        assert_eq!(row0, "line3");
    }

    #[test]
    fn paragraph_alignment_center() {
        let para = Paragraph::new("hi").alignment(WrapAlign::Center);
        let mut buf = Buffer::empty(10, 1);
        let rects = vec![Rect::new(0, 0, 10, 1)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        para.paint(&mut ctx);
        // "hi" centered in 10 cols = 4 spaces + "hi" + 4 spaces
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "h");
        assert_eq!(buf.cell(5, 0).unwrap().grapheme, "i");
    }

    #[test]
    fn paragraph_preserves_styles() {
        let content = Spans::plain("hello ").push_styled("world", Style::empty().bold());
        let para = Paragraph::new(content);
        let mut buf = Buffer::empty(20, 1);
        let rects = vec![Rect::new(0, 0, 20, 1)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        para.paint(&mut ctx);
        // "world" at position 6 should be bold
        let cell = buf.cell(6, 0).unwrap();
        assert_eq!(cell.grapheme, "w");
        assert!(cell.style.attr.contains(Attr::BOLD));
    }

    #[test]
    fn paragraph_fg_color() {
        let para = Paragraph::new("hello").fg(Color::GREEN);
        let mut buf = Buffer::empty(10, 1);
        let rects = vec![Rect::new(0, 0, 10, 1)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        para.paint(&mut ctx);
        assert_eq!(buf.cell(0, 0).unwrap().style.fg, Color::GREEN);
    }
}
