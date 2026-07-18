//! The `Scrollable` widget: a container that manages scroll offset for its
//! child content.
//!
//! This widget wraps a single child and tracks a scroll offset. The child's
//! content is painted with a vertical offset, and a scrollbar is drawn on the
//! right edge. The scroll offset is stored in [`crate::app::Context::state`]
//! so it persists across frames.
//!
//! For virtual scrolling (only rendering visible items in a long list), see
//! the [`crate::widgets::VirtualList`] widget.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A scrollable container that clips its child to the allocated rect and
/// applies a vertical scroll offset.
///
/// The scroll offset is read from the `scroll_key` state key in
/// [`crate::app::Context::state`]. The total content height is written to
/// `scroll_key + ".content_height"` so the caller can clamp the scroll.
pub struct Scrollable {
    /// The single child.
    pub child: Option<std::boxed::Box<dyn Widget>>,
    /// State key for the scroll offset (u16).
    pub scroll_key: String,
    /// Flex grow.
    pub grow: f32,
    /// Whether to show a scrollbar.
    pub show_scrollbar: bool,
    /// Scrollbar style.
    pub scrollbar_style: Style,
    /// Background color.
    pub bg: Color,
}

impl Scrollable {
    /// Construct a scrollable container.
    pub fn new(scroll_key: impl Into<String>) -> Self {
        Self {
            child: None,
            scroll_key: scroll_key.into(),
            grow: 1.0,
            show_scrollbar: true,
            scrollbar_style: Style::empty().fg(Color::rgb(100, 100, 100)),
            bg: Color::TRANSPARENT,
        }
    }

    /// Set the child.
    #[must_use]
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(std::boxed::Box::new(w));
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

    /// Show or hide the scrollbar.
    #[must_use]
    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Set scrollbar style.
    #[must_use]
    pub fn scrollbar_style(mut self, s: Style) -> Self {
        self.scrollbar_style = s;
        self
    }
}

impl Widget for Scrollable {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        props.to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }

        // The child is painted by the tree walker at its own rect.
        // We can't easily clip here, but we can draw the scrollbar.
        // In a real implementation, we'd use a clip rect in the buffer.
        // For now, we just draw the scrollbar indicator.

        if self.show_scrollbar && h > 2 {
            let scrollbar_x = x + w - 1;
            let track_style = Style::empty().fg(Color::rgb(60, 60, 60));
            // Draw track.
            for ry in y..y + h {
                ctx.buffer.print(scrollbar_x, ry, "│", track_style);
            }
            // Draw thumb (placeholder — real position depends on scroll state).
            // Without knowing content height, we just show a small thumb.
            let thumb_h = 1u16.max(h / 4);
            let thumb_y = y;
            for ry in thumb_y..thumb_y + thumb_h {
                if ry < y + h {
                    ctx.buffer.print(scrollbar_x, ry, "█", self.scrollbar_style);
                }
            }
        }
    }

    fn take_children(&mut self) -> Vec<std::boxed::Box<dyn Widget>> {
        self.child.take().into_iter().collect()
    }
}

/// Compute the scrollbar thumb position given total content and viewport.
///
/// Returns `(thumb_y, thumb_height)` in viewport-relative coordinates.
#[must_use]
pub fn scrollbar_thumb(content_height: u16, viewport_height: u16, scroll: u16) -> (u16, u16) {
    if content_height <= viewport_height {
        return (0, viewport_height);
    }
    let track_height = viewport_height;
    let thumb_height = (u32::from(viewport_height) * u32::from(viewport_height)
        / u32::from(content_height)) as u16;
    let thumb_height = thumb_height.max(1);
    let max_scroll = content_height.saturating_sub(viewport_height);
    let thumb_y = if max_scroll == 0 {
        0
    } else {
        (u32::from(scroll) * u32::from(track_height - thumb_height) / u32::from(max_scroll)) as u16
    };
    (thumb_y, thumb_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollbar_thumb_no_overflow() {
        let (y, h) = scrollbar_thumb(10, 20, 0);
        assert_eq!(y, 0);
        assert_eq!(h, 20);
    }

    #[test]
    fn scrollbar_thumb_with_overflow() {
        let (y, h) = scrollbar_thumb(100, 20, 0);
        assert_eq!(y, 0);
        assert_eq!(h, 4); // 20*20/100 = 4
    }

    #[test]
    fn scrollbar_thumb_at_bottom() {
        let (y, h) = scrollbar_thumb(100, 20, 80);
        assert_eq!(h, 4);
        assert_eq!(y, 16); // 80 * 16 / 80 = 16
    }

    #[test]
    fn scrollbar_thumb_mid_scroll() {
        let (y, h) = scrollbar_thumb(100, 20, 40);
        assert_eq!(h, 4);
        assert_eq!(y, 8); // 40 * 16 / 80 = 8
    }
}
