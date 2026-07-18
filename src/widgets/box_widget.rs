//! The `Flex` widget (a flex container) and the `Box` decorator.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A flex container: lays its children out along a main axis using Flexbox.
///
/// This is the primary layout primitive in RusTUI. Use [`Flex::column`] and
/// [`Flex::row`] to start a new container, then `.child(...)` to add children.
pub struct Flex {
    /// Flex properties (direction, align, justify, padding, etc.).
    pub props: FlexProps,
    /// Children.
    pub children: Vec<std::boxed::Box<dyn Widget>>,
    /// Optional background color for this container.
    pub bg: Color,
    /// Optional border style. If `Some`, draws a single-line border.
    pub border: Option<Style>,
}

impl Flex {
    /// Column layout (children stack vertically).
    pub fn column() -> Self {
        Self {
            props: FlexProps::column(),
            children: Vec::new(),
            bg: Color::TRANSPARENT,
            border: None,
        }
    }
    /// Row layout (children stack horizontally).
    pub fn row() -> Self {
        Self {
            props: FlexProps::row(),
            children: Vec::new(),
            bg: Color::TRANSPARENT,
            border: None,
        }
    }
    /// Set flex direction.
    #[must_use]
    pub fn direction(mut self, d: crate::layout::FlexDirection) -> Self {
        self.props.direction = d;
        self
    }
    /// Set cross-axis alignment.
    #[must_use]
    pub fn align(mut self, a: crate::layout::Align) -> Self {
        self.props.align = a;
        self
    }
    /// Set main-axis justification.
    #[must_use]
    pub fn justify(mut self, j: crate::layout::Justify) -> Self {
        self.props.justify = j;
        self
    }
    /// Set width.
    #[must_use]
    pub fn width(mut self, w: crate::layout::Length) -> Self {
        self.props.width = w;
        self
    }
    /// Set height.
    #[must_use]
    pub fn height(mut self, h: crate::layout::Length) -> Self {
        self.props.height = h;
        self
    }
    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.props.grow = g;
        self
    }
    /// Set padding (all sides).
    #[must_use]
    pub fn padding_all(mut self, p: f32) -> Self {
        self.props.padding = [p; 4];
        self
    }
    /// Set padding (top, right, bottom, left).
    #[must_use]
    pub fn padding(mut self, trbl: [f32; 4]) -> Self {
        self.props.padding = trbl;
        self
    }
    /// Set margin (top, right, bottom, left).
    #[must_use]
    pub fn margin(mut self, trbl: [f32; 4]) -> Self {
        self.props.margin = trbl;
        self
    }
    /// Set background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }
    /// Draw a single-line border with the given style.
    #[must_use]
    pub fn border(mut self, style: Style) -> Self {
        self.border = Some(style);
        self
    }
    /// Append a child widget.
    #[must_use]
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.children.push(std::boxed::Box::new(w));
        self
    }
    /// Append a boxed child widget.
    #[must_use]
    pub fn child_boxed(mut self, w: std::boxed::Box<dyn Widget>) -> Self {
        self.children.push(w);
        self
    }
}

impl Widget for Flex {
    fn layout(&self) -> LayoutNode {
        // Children indices are filled in by the tree walker.
        self.props.to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }
        if let Some(border_style) = self.border {
            ctx.buffer.box_border(ctx.rect, border_style);
        }
    }

    fn take_children(&mut self) -> Vec<std::boxed::Box<dyn Widget>> {
        std::mem::take(&mut self.children)
    }
}

/// A simple decorated box: background + optional border + a single child.
///
/// Unlike [`Flex`], `Box` always sizes to its child (it does not stretch
/// children along a main axis). Use it for padding/border wrappers.
pub struct Box {
    /// The single child.
    pub child: Option<std::boxed::Box<dyn Widget>>,
    /// Background color.
    pub bg: Color,
    /// Optional border style.
    pub border: Option<Style>,
    /// Padding (top, right, bottom, left).
    pub padding: [f32; 4],
    /// Flex grow.
    pub grow: f32,
}

impl Box {
    /// Construct an empty box.
    pub fn new() -> Self {
        Self {
            child: None,
            bg: Color::TRANSPARENT,
            border: None,
            padding: [0.0; 4],
            grow: 0.0,
        }
    }
    /// Set the child.
    #[must_use]
    pub fn child(mut self, w: impl Widget + 'static) -> Self {
        self.child = Some(std::boxed::Box::new(w));
        self
    }
    /// Set background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }
    /// Draw a border.
    #[must_use]
    pub fn border(mut self, s: Style) -> Self {
        self.border = Some(s);
        self
    }
    /// Set padding (all sides).
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
}

impl Default for Box {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Box {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        props.padding = self.padding;
        props.to_node(Vec::new())
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        if ctx.rect.is_empty() {
            return;
        }
        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }
        if let Some(border_style) = self.border {
            ctx.buffer.box_border(ctx.rect, border_style);
        }
    }

    fn take_children(&mut self) -> Vec<std::boxed::Box<dyn Widget>> {
        self.child.take().into_iter().collect()
    }
}
