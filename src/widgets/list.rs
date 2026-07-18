//! The `List` widget: a vertical list of selectable items.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::text::Spans;
use crate::widgets::base::{PaintCtx, Widget};

/// A single item in a [`List`].
pub struct ListItem {
    /// The item's content.
    pub content: Spans,
}

impl ListItem {
    /// Construct a list item from anything convertible to [`Spans`].
    pub fn new(content: impl Into<Spans>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<T: Into<Spans>> From<T> for ListItem {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// A vertical list of items with a selection cursor.
///
/// The list itself is stateless — the selected index lives in your
/// [`crate::app::Context`] state. Pass it in via [`List::selected`] each frame.
pub struct List {
    /// The items.
    pub items: Vec<ListItem>,
    /// Index of the currently selected item, or `None` for no selection.
    pub selected: Option<usize>,
    /// Style for the selected item's highlight.
    pub highlight_style: Style,
    /// Style applied to all items (item content styles compose over this).
    pub base_style: Style,
    /// Flex properties.
    pub flex: FlexProps,
}

impl List {
    /// Construct an empty list.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            highlight_style: Style::empty().bg(Color::BLUE).fg(Color::WHITE),
            base_style: Style::empty(),
            flex: FlexProps::column(),
        }
    }

    /// Construct a list from items.
    pub fn from_items(items: impl IntoIterator<Item = ListItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            ..Self::new()
        }
    }

    /// Set the selected index.
    #[must_use]
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Set the highlight style.
    #[must_use]
    pub fn highlight(mut self, s: Style) -> Self {
        self.highlight_style = s;
        self
    }

    /// Set the base style for unselected items.
    #[must_use]
    pub fn base_style(mut self, s: Style) -> Self {
        self.base_style = s;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for List {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        // A list wants to fill available height.
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        for (i, item) in self.items.iter().enumerate() {
            let row = y + i as u16;
            if row >= y + h {
                break;
            }
            let is_selected = self.selected == Some(i);
            let base = if is_selected {
                self.highlight_style
            } else {
                self.base_style
            };
            // Fill the row background if a bg is set.
            if base.bg != Color::TRANSPARENT {
                ctx.buffer.fill_rect(Rect::new(x, row, w, 1), base.bg);
            }
            // Render the item content.
            let mut cx = x;
            for span in &item.content.spans {
                let style = span.style.over(base);
                for g in crate::unicode::graphemes(&span.text) {
                    let gw = crate::unicode::grapheme_width(g);
                    if gw == 0 || cx + gw as u16 > x + w {
                        if cx + gw as u16 > x + w {
                            break;
                        }
                        continue;
                    }
                    ctx.buffer.print(cx, row, g, style);
                    cx += gw as u16;
                }
            }
        }
    }
}
