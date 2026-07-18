//! The `Block` widget: a decorated container with title, subtitle, borders, and padding.
//!
//! [`Block`] is a decorator wrapper around a single child widget, similar to
//! [`crate::widgets::Box`] but richer: it supports a title and subtitle drawn
//! over the top border, a choice of border glyph sets, a background fill, and
//! per-side padding.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// The set of border glyphs used by a [`Block`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BorderType {
    /// Plain single-line: `┌─┐│└┘`.
    #[default]
    Plain,
    /// Rounded corners: `╭─╮│╰─╯`.
    Rounded,
    /// Double-line: `╔═╗║╚╝`.
    Double,
    /// Thick single-line: `┏━┓┃┗┛`.
    Thick,
}

impl BorderType {
    /// The seven glyph pieces for this border type, in the order:
    /// top-left, top, top-right, side, bottom-left, bottom, bottom-right.
    const fn pieces(self) -> [&'static str; 7] {
        match self {
            BorderType::Plain => ["┌", "─", "┐", "│", "└", "─", "┘"],
            BorderType::Rounded => ["╭", "─", "╮", "│", "╰", "─", "╯"],
            BorderType::Double => ["╔", "═", "╗", "║", "╚", "═", "╝"],
            BorderType::Thick => ["┏", "━", "┓", "┃", "┗", "━", "┛"],
        }
    }
}

/// A decorated container: title + subtitle + border + background + padding,
/// wrapping a single child widget.
///
/// Construct with [`Block::new`] and configure with the builder methods, then
/// set a child with [`Block::child`]. The block sizes to its child (it does not
/// stretch children along a main axis); use [`crate::widgets::Flex`] when you
/// need flexbox distribution.
pub struct Block {
    /// Optional title drawn over the top border.
    pub title: Option<compact_str::CompactString>,
    /// Optional subtitle drawn over the top border, right-aligned.
    pub subtitle: Option<compact_str::CompactString>,
    /// Whether the title is centered (`true`) or left-aligned (`false`).
    pub title_centered: bool,
    /// Optional border style. If `Some`, a border is drawn using
    /// [`Block::border_type`] glyphs.
    pub border: Option<Style>,
    /// Which glyph set to use for the border.
    pub border_type: BorderType,
    /// Background color. `Color::TRANSPARENT` means no fill.
    pub bg: Color,
    /// Padding (top, right, bottom, left) in cells.
    pub padding: [f32; 4],
    /// Flex grow factor.
    pub grow: f32,
    /// The single child widget, if any.
    pub child: Option<std::boxed::Box<dyn Widget>>,
}

impl Block {
    /// Construct an empty block with no border, no title, and no padding.
    pub fn new() -> Self {
        Self {
            title: None,
            subtitle: None,
            title_centered: false,
            border: None,
            border_type: BorderType::Plain,
            bg: Color::TRANSPARENT,
            padding: [0.0; 4],
            grow: 0.0,
            child: None,
        }
    }

    /// Set the title (drawn over the top border).
    #[must_use]
    pub fn title(mut self, t: impl Into<compact_str::CompactString>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Set the subtitle (drawn over the top border, right-aligned).
    #[must_use]
    pub fn subtitle(mut self, t: impl Into<compact_str::CompactString>) -> Self {
        self.subtitle = Some(t.into());
        self
    }

    /// Center the title within the top border (`true`), or left-align it (`false`).
    #[must_use]
    pub fn title_centered(mut self, centered: bool) -> Self {
        self.title_centered = centered;
        self
    }

    /// Draw a border with the given style.
    #[must_use]
    pub fn border(mut self, s: Style) -> Self {
        self.border = Some(s);
        self
    }

    /// Choose which glyph set the border uses.
    #[must_use]
    pub fn border_type(mut self, t: BorderType) -> Self {
        self.border_type = t;
        self
    }

    /// Set the background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    /// Set padding on all four sides.
    #[must_use]
    pub fn padding_all(mut self, p: f32) -> Self {
        self.padding = [p; 4];
        self
    }

    /// Set padding (top, right, bottom, left).
    #[must_use]
    pub fn padding(mut self, trbl: [f32; 4]) -> Self {
        self.padding = trbl;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }

    /// Set the single child widget.
    #[must_use]
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(std::boxed::Box::new(w));
        self
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Block {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        props.padding = self.padding;
        props.to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        // 1. Fill background.
        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }

        // 2. Draw border (if any) with the appropriate glyphs.
        if let Some(border_style) = self.border {
            let [tl, top, tr, side, bl, _bottom, br] = self.border_type.pieces();
            let right = x + w - 1;
            let bottom_row = y + h - 1;

            // Corners.
            ctx.buffer.print(x, y, tl, border_style);
            ctx.buffer.print(right, y, tr, border_style);
            ctx.buffer.print(x, bottom_row, bl, border_style);
            ctx.buffer.print(right, bottom_row, br, border_style);

            // Top and bottom edges.
            if w > 2 {
                let edge: String = top.to_string().repeat((w - 2) as usize);
                ctx.buffer.print(x + 1, y, &edge, border_style);
                ctx.buffer.print(x + 1, bottom_row, &edge, border_style);
            }

            // Left and right edges.
            if h > 2 {
                for ry in (y + 1)..bottom_row {
                    ctx.buffer.print(x, ry, side, border_style);
                    ctx.buffer.print(right, ry, side, border_style);
                }
            }

            // 3. Render the title over the top border line.
            if let Some(title) = &self.title {
                let title_w = crate::unicode::str_width(title) as u16;
                if title_w > 0 && w > 2 {
                    let inner = w - 2; // columns between the two top corners
                    let tx = if self.title_centered {
                        x + 1 + (inner.saturating_sub(title_w)) / 2
                    } else {
                        x + 1
                    };
                    // `print` stops at the right edge, so a long title is clipped.
                    ctx.buffer.print(tx, y, title.as_str(), border_style);
                }
            }

            // 4. Render the subtitle over the top border line, right-aligned.
            if let Some(sub) = &self.subtitle {
                let sub_w = crate::unicode::str_width(sub) as u16;
                if sub_w > 0 && w > 2 {
                    let inner = w - 2;
                    let sx = if sub_w >= inner {
                        x + 1
                    } else {
                        right - 1 - sub_w
                    };
                    ctx.buffer.print(sx, y, sub.as_str(), border_style);
                }
            }
        }
    }

    fn take_children(&mut self) -> Vec<std::boxed::Box<dyn Widget>> {
        self.child.take().into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    fn painted_block(w: u16, h: u16, block: &Block) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rect = Rect::new(0, 0, w, h);
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect,
            rects: &[rect],
            elapsed: std::time::Duration::ZERO,
        };
        block.paint(&mut ctx);
        buf
    }

    #[test]
    fn plain_border_corners() {
        let block = Block::new().border(Style::empty());
        let buf = painted_block(5, 3, &block);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "┌");
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "┐");
        assert_eq!(buf.cell(0, 2).unwrap().grapheme, "└");
        assert_eq!(buf.cell(4, 2).unwrap().grapheme, "┘");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "─");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "│");
    }

    #[test]
    fn rounded_border_corners() {
        let block = Block::new()
            .border(Style::empty())
            .border_type(BorderType::Rounded);
        let buf = painted_block(5, 3, &block);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "╭");
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "╮");
        assert_eq!(buf.cell(0, 2).unwrap().grapheme, "╰");
        assert_eq!(buf.cell(4, 2).unwrap().grapheme, "╯");
    }

    #[test]
    fn double_border_uses_double_glyphs() {
        let block = Block::new()
            .border(Style::empty())
            .border_type(BorderType::Double);
        let buf = painted_block(5, 3, &block);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "╔");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "═");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "║");
    }

    #[test]
    fn thick_border_uses_thick_glyphs() {
        let block = Block::new()
            .border(Style::empty())
            .border_type(BorderType::Thick);
        let buf = painted_block(5, 3, &block);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "┏");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "━");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "┃");
    }

    #[test]
    fn title_left_aligned_overwrites_top_edge() {
        let block = Block::new()
            .border(Style::empty())
            .title("Hi")
            .title_centered(false);
        let buf = painted_block(10, 3, &block);
        // Title starts right after the top-left corner.
        assert_eq!(buf.cell(1, 0).unwrap().grapheme, "H");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "i");
        // The rest of the top edge is still border glyphs.
        assert_eq!(buf.cell(3, 0).unwrap().grapheme, "─");
    }

    #[test]
    fn title_centered() {
        let block = Block::new()
            .border(Style::empty())
            .title("Hi")
            .title_centered(true);
        let buf = painted_block(10, 3, &block);
        // inner width = 8, title width = 2 -> offset = (8-2)/2 = 3 -> col 1+3 = 4
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "H");
        assert_eq!(buf.cell(5, 0).unwrap().grapheme, "i");
    }

    #[test]
    fn subtitle_right_aligned() {
        let block = Block::new()
            .border(Style::empty())
            .subtitle("ok")
            .title_centered(false);
        let buf = painted_block(10, 3, &block);
        // right edge col = 9, subtitle width = 2 -> start at 9-1-2 = 6
        assert_eq!(buf.cell(6, 0).unwrap().grapheme, "o");
        assert_eq!(buf.cell(7, 0).unwrap().grapheme, "k");
        assert_eq!(buf.cell(8, 0).unwrap().grapheme, "─");
    }

    #[test]
    fn background_fill() {
        let block = Block::new().bg(Color::RED);
        let buf = painted_block(3, 2, &block);
        assert_eq!(buf.cell(0, 0).unwrap().style.bg, Color::RED);
        assert_eq!(buf.cell(2, 1).unwrap().style.bg, Color::RED);
    }

    #[test]
    fn no_border_draws_nothing() {
        let block = Block::new();
        let buf = painted_block(4, 3, &block);
        // Everything should be blank.
        assert!(buf.cell(0, 0).unwrap().is_blank());
        assert!(buf.cell(3, 2).unwrap().is_blank());
    }

    #[test]
    fn take_children_yields_child() {
        let mut block = Block::new().child(crate::widgets::Text::new("hi"));
        let kids = block.take_children();
        assert_eq!(kids.len(), 1);
        // Second call drains nothing.
        assert!(block.take_children().is_empty());
    }
}
