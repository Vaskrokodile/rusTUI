//! The `Tabs` widget: a horizontal row of selectable tab labels.
//!
//! [`Tabs`] renders a row of plain-text labels separated by spaces, with one
//! tab highlighted as active (bold + accent color + underline) and the rest
//! rendered in a dim/muted style. An optional divider line can be drawn under
//! the tab bar. The widget is stateless — the active index lives in your
//! [`crate::app::Context`] state and is passed in via [`Tabs::active`] each
//! frame.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexDirection, FlexProps, LayoutNode, Length};
use crate::style::{Attr, Style};
use crate::widgets::base::{PaintCtx, Widget};

/// A horizontal row of tab labels with one tab highlighted as active.
///
/// Construct with [`Tabs::new`] (or start empty and use [`Tabs::label`] to add
/// labels one at a time), then configure the active index and styles with the
/// builder methods.
pub struct Tabs {
    /// Tab labels (plain text).
    pub labels: Vec<compact_str::CompactString>,
    /// Index of the active tab.
    pub active: usize,
    /// Style for active tab.
    pub active_style: Style,
    /// Style for inactive tabs.
    pub inactive_style: Style,
    /// Background color for the tab bar.
    pub bg: Color,
    /// Flex grow.
    pub grow: f32,
    /// Whether to draw a divider line under the tabs.
    pub divider: bool,
    /// Style for the divider line.
    pub divider_style: Style,
}

impl Tabs {
    /// Construct a tabs widget from anything convertible to labels.
    ///
    /// Each element of `labels` is converted to a [`compact_str::CompactString`].
    pub fn new(labels: impl IntoIterator<Item = impl Into<compact_str::CompactString>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            active: 0,
            active_style: Style::empty()
                .fg(Color::CYAN)
                .attr(Attr::BOLD | Attr::UNDERLINE),
            inactive_style: Style::empty().fg(Color::palette256(8)).dim(),
            bg: Color::TRANSPARENT,
            grow: 0.0,
            divider: true,
            divider_style: Style::empty().fg(Color::palette256(8)),
        }
    }

    /// Set the index of the active tab.
    #[must_use]
    pub fn active(mut self, idx: usize) -> Self {
        self.active = idx;
        self
    }

    /// Set the style for the active tab.
    #[must_use]
    pub fn active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    /// Set the style for inactive tabs.
    #[must_use]
    pub fn inactive_style(mut self, style: Style) -> Self {
        self.inactive_style = style;
        self
    }

    /// Set the background color for the tab bar.
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }

    /// Add a label to the end of the tab list.
    #[must_use]
    pub fn label(mut self, label: impl Into<compact_str::CompactString>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Enable or disable the divider line drawn under the tabs.
    #[must_use]
    pub fn divider(mut self, on: bool) -> Self {
        self.divider = on;
        self
    }

    /// Set the style for the divider line.
    #[must_use]
    pub fn divider_style(mut self, style: Style) -> Self {
        self.divider_style = style;
        self
    }
}

impl Widget for Tabs {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.direction = FlexDirection::Row;
        props.grow = self.grow;
        let mut node = props.to_node(Vec::new());
        // Tabs want to fill the available width and take a single row (plus an
        // optional divider line).
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        // Fill the tab-bar background if requested.
        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }

        // The tab labels live on the first row; the divider (if any) lives on
        // the row below it.
        let label_row = y;
        let divider_row = y.saturating_add(1);

        let mut cx = x;
        for (i, label) in self.labels.iter().enumerate() {
            // Separator space between tabs (skip before the first one).
            if i > 0 && cx < x + w {
                ctx.buffer.print(cx, label_row, " ", self.inactive_style);
                cx = cx.saturating_add(1);
            }

            let style = if i == self.active {
                self.active_style
            } else {
                self.inactive_style
            };

            // Render the label grapheme-by-grapheme so we can stop cleanly at
            // the right edge of the rect.
            for g in crate::unicode::graphemes(label.as_str()) {
                if cx >= x + w {
                    break;
                }
                let gw = crate::unicode::grapheme_width(g) as u16;
                if gw == 0 {
                    continue;
                }
                ctx.buffer.print(cx, label_row, g, style);
                cx = cx.saturating_add(gw);
            }
        }

        // Optionally draw a divider line under the tabs.
        if self.divider && h > 1 && divider_row < y + h {
            let line: String = "─".repeat(w as usize);
            ctx.buffer.print(x, divider_row, &line, self.divider_style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::style::Attr;

    fn painted_tabs(w: u16, h: u16, tabs: &Tabs) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rect = Rect::new(0, 0, w, h);
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect,
            rects: &[rect],
            elapsed: std::time::Duration::ZERO,
        };
        tabs.paint(&mut ctx);
        buf
    }

    #[test]
    fn renders_labels_separated_by_spaces() {
        let tabs = Tabs::new(["a", "b", "c"]).active(0);
        let buf = painted_tabs(10, 2, &tabs);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "a");
        assert_eq!(buf.cell(1, 0).unwrap().grapheme, " ");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "b");
        assert_eq!(buf.cell(3, 0).unwrap().grapheme, " ");
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "c");
    }

    #[test]
    fn active_tab_uses_active_style() {
        let tabs = Tabs::new(["a", "b"]).active(1).divider(false);
        let buf = painted_tabs(5, 1, &tabs);
        let active = buf.cell(2, 0).unwrap().style;
        assert_eq!(active.fg, Color::CYAN);
        assert!(active.attr.contains(Attr::BOLD));
        assert!(active.attr.contains(Attr::UNDERLINE));
        // Inactive tab is dim.
        let inactive = buf.cell(0, 0).unwrap().style;
        assert!(inactive.attr.contains(Attr::DIM));
    }

    #[test]
    fn divider_line_drawn_below_labels() {
        let tabs = Tabs::new(["a", "b"]).active(0);
        let buf = painted_tabs(4, 2, &tabs);
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "─");
        assert_eq!(buf.cell(3, 1).unwrap().grapheme, "─");
    }

    #[test]
    fn label_builder_appends_labels() {
        let tabs = Tabs::new(["a"])
            .label("b")
            .label("c")
            .active(2)
            .divider(false);
        let buf = painted_tabs(10, 1, &tabs);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "a");
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "b");
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "c");
        // The active tab (index 2 = "c") uses the accent color.
        assert_eq!(buf.cell(4, 0).unwrap().style.fg, Color::CYAN);
    }
}
