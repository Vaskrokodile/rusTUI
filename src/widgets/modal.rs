//! Modal and dialog overlay widgets.
//!
//! [`Modal`] is a container that renders a centered overlay dialog on top of
//! the rest of the screen, optionally dimming the background behind it. It
//! wraps a single child widget which is painted by the tree walker at the
//! modal's inner rect (inside the border).
//!
//! [`Dialog`] is a leaf widget for simple yes/no/confirm dialogs: a title, a
//! message, and a row of buttons with one highlighted.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};
use crate::widgets::BorderType;

/// Clamp a percentage into the valid `0.0..=1.0` range.
fn clamp_pct(p: f32) -> f32 {
    p.clamp(0.0, 1.0)
}

/// An overlay dialog that renders a bordered box centered on the viewport.
///
/// The modal fills the entire rect it is given (typically the whole screen),
/// optionally drawing a semi-transparent overlay behind the dialog box, then
/// draws a centered bordered box sized as a percentage of the viewport. The
/// child widget — if any — is painted by the tree walker at the modal's inner
/// rect (the area inside the border).
pub struct Modal {
    /// Title shown in the border.
    pub title: compact_str::CompactString,
    /// Body content (a child widget).
    pub child: Option<std::boxed::Box<dyn Widget>>,
    /// Modal width as a percentage of screen (0.0 to 1.0).
    pub width_pct: f32,
    /// Modal height as a percentage of screen (0.0 to 1.0).
    pub height_pct: f32,
    /// Border style.
    pub border_style: Style,
    /// Border type.
    pub border_type: BorderType,
    /// Background color (semi-transparent dark overlay).
    pub overlay_bg: Color,
    /// Modal background.
    pub bg: Color,
    /// Title style.
    pub title_style: Style,
    /// Whether to show an overlay behind the modal.
    pub show_overlay: bool,
}

impl Modal {
    /// Construct a modal with the given title and sensible defaults.
    ///
    /// The modal defaults to 60% width, 40% height, a plain border, a dark
    /// semi-transparent overlay, and an opaque dialog background.
    #[must_use]
    pub fn new(title: impl Into<compact_str::CompactString>) -> Self {
        Self {
            title: title.into(),
            child: None,
            width_pct: 0.6,
            height_pct: 0.4,
            border_style: Style::empty().fg(Color::WHITE),
            border_type: BorderType::Plain,
            overlay_bg: Color::rgba(0, 0, 0, 160),
            bg: Color::rgb(20, 20, 30),
            title_style: Style::empty().fg(Color::WHITE).bold(),
            show_overlay: true,
        }
    }

    /// Set the child widget rendered inside the modal.
    #[must_use]
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(std::boxed::Box::new(w));
        self
    }

    /// Set the modal width as a percentage of the screen (`0.0..=1.0`).
    #[must_use]
    pub fn width_pct(mut self, p: f32) -> Self {
        self.width_pct = clamp_pct(p);
        self
    }

    /// Set the modal height as a percentage of the screen (`0.0..=1.0`).
    #[must_use]
    pub fn height_pct(mut self, p: f32) -> Self {
        self.height_pct = clamp_pct(p);
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn border_style(mut self, s: Style) -> Self {
        self.border_style = s;
        self
    }

    /// Set the border glyph set.
    #[must_use]
    pub fn border_type(mut self, t: BorderType) -> Self {
        self.border_type = t;
        self
    }

    /// Set the overlay background color (drawn behind the modal).
    #[must_use]
    pub fn overlay_bg(mut self, c: Color) -> Self {
        self.overlay_bg = c;
        self
    }

    /// Set the modal background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    /// Set the title style.
    #[must_use]
    pub fn title_style(mut self, s: Style) -> Self {
        self.title_style = s;
        self
    }

    /// Enable or disable the dimming overlay behind the modal.
    #[must_use]
    pub fn show_overlay(mut self, show: bool) -> Self {
        self.show_overlay = show;
        self
    }

    /// Compute the centered modal rect within `viewport`.
    fn modal_rect(&self, viewport: Rect) -> Rect {
        let vp_w = f32::from(viewport.w);
        let vp_h = f32::from(viewport.h);
        let mw = (vp_w * clamp_pct(self.width_pct)).round() as u16;
        let mh = (vp_h * clamp_pct(self.height_pct)).round() as u16;
        // Clamp to the viewport and leave room for at least a border.
        let mw = mw.min(viewport.w).max(3);
        let mh = mh.min(viewport.h).max(3);
        let mx = viewport.x + (viewport.w.saturating_sub(mw)) / 2;
        let my = viewport.y + (viewport.h.saturating_sub(mh)) / 2;
        Rect::new(mx, my, mw, mh)
    }

    /// Draw a border with the configured glyph set and title.
    fn draw_border(&self, ctx: &mut PaintCtx, rect: Rect) {
        let Rect { x, y, w, h } = rect;
        if w == 0 || h == 0 {
            return;
        }
        let [tl, top, tr, side, bl, _bottom, br] = self.border_type.pieces();
        let right = x + w - 1;
        let bottom_row = y + h - 1;

        // Corners.
        ctx.buffer.print(x, y, tl, self.border_style);
        ctx.buffer.print(right, y, tr, self.border_style);
        ctx.buffer.print(x, bottom_row, bl, self.border_style);
        ctx.buffer.print(right, bottom_row, br, self.border_style);

        // Top and bottom edges.
        if w > 2 {
            let edge: String = top.repeat((w - 2) as usize);
            ctx.buffer.print(x + 1, y, &edge, self.border_style);
            ctx.buffer
                .print(x + 1, bottom_row, &edge, self.border_style);
        }

        // Left and right edges.
        if h > 2 {
            for ry in (y + 1)..bottom_row {
                ctx.buffer.print(x, ry, side, self.border_style);
                ctx.buffer.print(right, ry, side, self.border_style);
            }
        }

        // Title over the top border, left-aligned.
        let title_w = crate::unicode::str_width(self.title.as_str()) as u16;
        if title_w > 0 && w > 2 {
            let inner = w - 2;
            let max_w = inner.min(title_w);
            let tx = x + 1;
            // Print only as much of the title as fits.
            let mut printed = 0u16;
            for g in crate::unicode::graphemes(self.title.as_str()) {
                if printed >= max_w {
                    break;
                }
                let gw = crate::unicode::grapheme_width(g) as u16;
                if gw == 0 {
                    continue;
                }
                if printed + gw > max_w {
                    break;
                }
                ctx.buffer.print(tx + printed, y, g, self.title_style);
                printed += gw;
            }
        }
    }
}

impl Widget for Modal {
    fn layout(&self) -> LayoutNode {
        // The modal claims the whole viewport (grow) and lays its child out in
        // a column inside the bordered box.
        let mut props = FlexProps::column();
        props.grow = 1.0;
        // Reserve a 1-cell border on every side for the child.
        props.padding = [1.0; 4];
        props.to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let viewport = ctx.rect;
        if viewport.is_empty() {
            return;
        }

        // 1. Fill the overlay background across the whole viewport.
        if self.show_overlay && self.overlay_bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(viewport, self.overlay_bg);
        }

        // 2. Compute the centered modal rect.
        let modal = self.modal_rect(viewport);

        // 3. Fill the modal background.
        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(modal, self.bg);
        }

        // 4. Draw the border with the title.
        self.draw_border(ctx, modal);

        // The child is painted by the tree walker at the modal's inner rect.
        // The walker uses the layout-computed rect for the child node, which
        // the padding above reserves inside the modal's allocated area. Here
        // we only paint the modal chrome.
    }

    fn take_children(&mut self) -> Vec<std::boxed::Box<dyn Widget>> {
        self.child.take().into_iter().collect()
    }
}

/// A simple yes/no/confirm dialog: title, message, and a row of buttons.
///
/// [`Dialog`] is a leaf widget — it paints everything itself, including the
/// border, message text, and buttons. One button may be highlighted (with
/// inverted colors) to indicate the default/selected action.
pub struct Dialog {
    /// Dialog title.
    pub title: compact_str::CompactString,
    /// Message text.
    pub message: compact_str::CompactString,
    /// Button labels.
    pub buttons: Vec<compact_str::CompactString>,
    /// Index of highlighted button.
    pub highlighted: usize,
    /// Dialog style.
    pub style: Style,
    /// Button style.
    pub button_style: Style,
    /// Highlighted button style.
    pub highlighted_style: Style,
}

impl Dialog {
    /// Construct a dialog with the given title and message, no buttons.
    #[must_use]
    pub fn new(
        title: impl Into<compact_str::CompactString>,
        message: impl Into<compact_str::CompactString>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            buttons: Vec::new(),
            highlighted: 0,
            style: Style::empty().fg(Color::WHITE),
            button_style: Style::empty().fg(Color::WHITE),
            highlighted_style: Style::empty().fg(Color::BLACK).bg(Color::WHITE),
        }
    }

    /// Set the button labels.
    #[must_use]
    pub fn buttons(mut self, labels: Vec<compact_str::CompactString>) -> Self {
        self.buttons = labels;
        self
    }

    /// Set the index of the highlighted button.
    #[must_use]
    pub fn highlighted(mut self, idx: usize) -> Self {
        self.highlighted = idx;
        self
    }

    /// Set the dialog (message/title) style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set the style for non-highlighted buttons.
    #[must_use]
    pub fn button_style(mut self, s: Style) -> Self {
        self.button_style = s;
        self
    }

    /// Set the style for the highlighted button.
    #[must_use]
    pub fn highlighted_style(mut self, s: Style) -> Self {
        self.highlighted_style = s;
        self
    }

    /// Compute the total display width of all buttons plus separators.
    fn buttons_width(&self) -> usize {
        let mut total = 0usize;
        for (i, b) in self.buttons.iter().enumerate() {
            if i > 0 {
                // One space between buttons.
                total += 1;
            }
            total += crate::unicode::str_width(b.as_str());
        }
        total
    }

    /// Draw a single button label at `(x, y)` with the given style.
    fn draw_button(ctx: &mut PaintCtx, x: u16, y: u16, label: &str, style: Style) -> u16 {
        let mut cx = x;
        for g in crate::unicode::graphemes(label) {
            let gw = crate::unicode::grapheme_width(g) as u16;
            if gw == 0 {
                continue;
            }
            ctx.buffer.print(cx, y, g, style);
            cx += gw;
        }
        cx
    }
}

impl Widget for Dialog {
    fn layout(&self) -> LayoutNode {
        FlexProps::column().to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        // 1. Fill background.
        let bg = if self.style.bg == Color::TRANSPARENT {
            Color::rgb(20, 20, 30)
        } else {
            self.style.bg
        };
        ctx.buffer.fill_rect(ctx.rect, bg);

        // 2. Draw a plain border with the title.
        let border_style = self.style;
        let [tl, top, tr, side, bl, _bottom, br] = BorderType::Plain.pieces();
        let right = x + w - 1;
        let bottom_row = y + h - 1;
        ctx.buffer.print(x, y, tl, border_style);
        ctx.buffer.print(right, y, tr, border_style);
        ctx.buffer.print(x, bottom_row, bl, border_style);
        ctx.buffer.print(right, bottom_row, br, border_style);
        if w > 2 {
            let edge: String = top.repeat((w - 2) as usize);
            ctx.buffer.print(x + 1, y, &edge, border_style);
            ctx.buffer.print(x + 1, bottom_row, &edge, border_style);
        }
        if h > 2 {
            for ry in (y + 1)..bottom_row {
                ctx.buffer.print(x, ry, side, border_style);
                ctx.buffer.print(right, ry, side, border_style);
            }
        }

        // Title over the top border, left-aligned.
        let title_w = crate::unicode::str_width(self.title.as_str()) as u16;
        if title_w > 0 && w > 2 {
            let inner = w - 2;
            let max_w = inner.min(title_w);
            let tx = x + 1;
            let mut printed = 0u16;
            for g in crate::unicode::graphemes(self.title.as_str()) {
                if printed >= max_w {
                    break;
                }
                let gw = crate::unicode::grapheme_width(g) as u16;
                if gw == 0 {
                    continue;
                }
                if printed + gw > max_w {
                    break;
                }
                ctx.buffer.print(tx + printed, y, g, self.style.bold());
                printed += gw;
            }
        }

        // 3. Message text, starting one row below the top border.
        if h > 2 {
            let inner_x = x + 1;
            let inner_w = w.saturating_sub(2);
            let mut cx = inner_x;
            let mut cy = y + 1;
            for g in crate::unicode::graphemes(self.message.as_str()) {
                let gw = crate::unicode::grapheme_width(g) as u16;
                if gw == 0 {
                    continue;
                }
                // Wrap at the inner width.
                if cx + gw > inner_x + inner_w {
                    cx = inner_x;
                    cy += 1;
                    if cy >= bottom_row {
                        break;
                    }
                }
                if cy >= bottom_row {
                    break;
                }
                ctx.buffer.print(cx, cy, g, self.style);
                cx += gw;
            }
        }

        // 4. Buttons centered on the last inner row.
        if h > 2 && !self.buttons.is_empty() {
            let inner_w = w.saturating_sub(2);
            let total = self.buttons_width() as u16;
            let mut bx = x + 1 + inner_w.saturating_sub(total) / 2;
            let by = bottom_row - 1;
            if by <= y {
                return;
            }
            for (i, label) in self.buttons.iter().enumerate() {
                if i > 0 {
                    bx += 1; // separator space
                }
                let style = if i == self.highlighted {
                    self.highlighted_style
                } else {
                    self.button_style
                };
                Self::draw_button(ctx, bx, by, label.as_str(), style);
                bx += crate::unicode::str_width(label.as_str()) as u16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::widgets::Text;
    use std::time::Duration;

    fn paint_modal(w: u16, h: u16, modal: &Modal) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rect = Rect::new(0, 0, w, h);
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect,
            rects: &[rect],
            elapsed: Duration::ZERO,
        };
        modal.paint(&mut ctx);
        buf
    }

    fn paint_dialog(w: u16, h: u16, dialog: &Dialog) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rect = Rect::new(0, 0, w, h);
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect,
            rects: &[rect],
            elapsed: Duration::ZERO,
        };
        dialog.paint(&mut ctx);
        buf
    }

    #[test]
    fn modal_fills_overlay_background() {
        let modal = Modal::new("Title").show_overlay(true);
        let buf = paint_modal(20, 10, &modal);
        // Corner of the viewport should have the overlay background.
        assert_eq!(buf.cell(0, 0).unwrap().style.bg, modal.overlay_bg);
        assert_eq!(buf.cell(19, 9).unwrap().style.bg, modal.overlay_bg);
    }

    #[test]
    fn modal_no_overlay_leaves_background_blank() {
        let modal = Modal::new("Title").show_overlay(false);
        let buf = paint_modal(20, 10, &modal);
        // Without overlay, the far corner should not get the overlay color.
        assert_ne!(buf.cell(0, 0).unwrap().style.bg, modal.overlay_bg);
    }

    #[test]
    fn modal_draws_centered_border_with_title() {
        // 60% of 20 = 12 wide, 40% of 10 = 4 tall -> centered at x=4, y=3.
        let modal = Modal::new("Hi");
        let buf = paint_modal(20, 10, &modal);
        // Top-left corner of the modal box at (4, 3).
        assert_eq!(buf.cell(4, 3).unwrap().grapheme, "┌");
        // Title "Hi" starts right after the corner.
        assert_eq!(buf.cell(5, 3).unwrap().grapheme, "H");
        assert_eq!(buf.cell(6, 3).unwrap().grapheme, "i");
    }

    #[test]
    fn modal_take_children_yields_child() {
        let mut modal = Modal::new("Title").child(Text::new("body"));
        let kids = modal.take_children();
        assert_eq!(kids.len(), 1);
        // Second call drains nothing.
        assert!(modal.take_children().is_empty());
    }

    #[test]
    fn dialog_draws_title_message_and_buttons() {
        let dialog = Dialog::new("Confirm", "Are you sure?")
            .buttons(vec![
                compact_str::CompactString::new("OK"),
                compact_str::CompactString::new("Cancel"),
            ])
            .highlighted(0);
        let buf = paint_dialog(24, 5, &dialog);
        // Title "Confirm" over the top border.
        assert_eq!(buf.cell(1, 0).unwrap().grapheme, "C");
        // Message on row 1.
        assert_eq!(buf.cell(1, 1).unwrap().grapheme, "A");
        // Buttons on the last inner row (row 3). "OK" highlighted (inverted).
        // total buttons width = 2 + 1 + 6 = 9; inner_w = 22; start = 1 + (22-9)/2 = 7.
        assert_eq!(buf.cell(7, 3).unwrap().grapheme, "O");
        assert_eq!(buf.cell(8, 3).unwrap().grapheme, "K");
        // Highlighted button uses the highlighted style background.
        assert_eq!(
            buf.cell(7, 3).unwrap().style.bg,
            dialog.highlighted_style.bg
        );
        // Non-highlighted "Cancel" starts at 7 + 2 + 1 = 10.
        assert_eq!(buf.cell(10, 3).unwrap().grapheme, "C");
        assert_eq!(buf.cell(11, 3).unwrap().grapheme, "a");
        // Non-highlighted button uses the button style (no inverted bg).
        assert_ne!(
            buf.cell(10, 3).unwrap().style.bg,
            dialog.highlighted_style.bg
        );
    }
}
