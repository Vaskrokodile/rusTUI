//! Flexbox layout via `taffy`.
//!
//! Widgets declare a [`Style`] (flex direction, grow, shrink, basis, padding,
//! margin, alignment) and a content size hint; the layout pass produces a
//! [`Rect`] for every node. This is the same model OpenTUI uses (Yoga), just
//! backed by `taffy` instead of the Zig Yoga bindings.

use crate::buffer::Rect;

/// Flex direction (which axis is the main axis).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexDirection {
    /// Children laid out top-to-bottom. Main axis = vertical.
    #[default]
    Column,
    /// Children laid out left-to-right. Main axis = horizontal.
    Row,
    /// Children laid out bottom-to-top.
    ColumnReverse,
    /// Children laid out right-to-left.
    RowReverse,
}

impl From<FlexDirection> for taffy::FlexDirection {
    fn from(d: FlexDirection) -> Self {
        match d {
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
        }
    }
}

/// Cross-axis alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Stretch children to fill the cross axis.
    #[default]
    Stretch,
    /// Align children to the start of the cross axis.
    Start,
    /// Align children to the center of the cross axis.
    Center,
    /// Align children to the end of the cross axis.
    End,
}

impl From<Align> for taffy::AlignItems {
    fn from(a: Align) -> Self {
        match a {
            Align::Stretch => taffy::AlignItems::Stretch,
            Align::Start => taffy::AlignItems::FlexStart,
            Align::Center => taffy::AlignItems::Center,
            Align::End => taffy::AlignItems::FlexEnd,
        }
    }
}

/// Main-axis distribution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    /// Pack children at the start of the main axis.
    #[default]
    Start,
    /// Pack children at the center of the main axis.
    Center,
    /// Pack children at the end of the main axis.
    End,
    /// Spread children evenly; first at start, last at end.
    SpaceBetween,
    /// Spread children evenly with space around each.
    SpaceAround,
    /// Spread children evenly with equal space between.
    SpaceEvenly,
}

impl From<Justify> for taffy::JustifyContent {
    fn from(j: Justify) -> Self {
        match j {
            Justify::Start => taffy::JustifyContent::FlexStart,
            Justify::Center => taffy::JustifyContent::Center,
            Justify::End => taffy::JustifyContent::FlexEnd,
            Justify::SpaceBetween => taffy::JustifyContent::SpaceBetween,
            Justify::SpaceAround => taffy::JustifyContent::SpaceAround,
            Justify::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
        }
    }
}

/// A length value: fixed pixels, percentage of parent, or "auto".
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Length {
    /// Auto — let the layout engine decide.
    #[default]
    Auto,
    /// Fixed number of columns/rows.
    Fixed(f32),
    /// Percentage of the parent's main-axis size (`0.0..=1.0`).
    Percent(f32),
    /// Flex grow factor.
    Grow(f32),
}

impl From<Length> for taffy::Dimension {
    fn from(l: Length) -> Self {
        match l {
            Length::Auto => taffy::Dimension::Auto,
            Length::Fixed(v) => taffy::Dimension::Length(v),
            Length::Percent(v) => taffy::Dimension::Percent(v),
            Length::Grow(_) => taffy::Dimension::Auto,
        }
    }
}

/// A node in the layout tree.
///
/// Each widget produces one of these; the layout pass turns them into rects.
pub struct LayoutNode {
    /// Flex direction for this node's children.
    pub direction: FlexDirection,
    /// Cross-axis alignment.
    pub align: Align,
    /// Main-axis justification.
    pub justify: Justify,
    /// Main-axis size.
    pub width: Length,
    /// Cross-axis size.
    pub height: Length,
    /// Flex grow factor.
    pub grow: f32,
    /// Flex shrink factor.
    pub shrink: f32,
    /// Flex basis.
    pub basis: Length,
    /// Padding (top, right, bottom, left) in cells.
    pub padding: [f32; 4],
    /// Margin (top, right, bottom, left) in cells.
    pub margin: [f32; 4],
    /// Child node indices in the global node list.
    pub children: Vec<usize>,
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Column,
            align: Align::Stretch,
            justify: Justify::Start,
            width: Length::Auto,
            height: Length::Auto,
            grow: 0.0,
            shrink: 1.0,
            basis: Length::Auto,
            padding: [0.0; 4],
            margin: [0.0; 4],
            children: Vec::new(),
        }
    }
}

impl LayoutNode {
    fn to_taffy(&self) -> taffy::Style {
        taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: self.direction.into(),
            align_items: Some(self.align.into()),
            justify_content: Some(self.justify.into()),
            size: taffy::Size {
                width: self.width.into(),
                height: self.height.into(),
            },
            flex_grow: self.grow,
            flex_shrink: self.shrink,
            flex_basis: match self.basis {
                Length::Auto => taffy::Dimension::Auto,
                Length::Fixed(v) => taffy::Dimension::Length(v),
                Length::Percent(v) => taffy::Dimension::Percent(v),
                Length::Grow(_) => taffy::Dimension::Auto,
            },
            padding: taffy::Rect {
                top: taffy::LengthPercentage::Length(self.padding[0]),
                right: taffy::LengthPercentage::Length(self.padding[1]),
                bottom: taffy::LengthPercentage::Length(self.padding[2]),
                left: taffy::LengthPercentage::Length(self.padding[3]),
            },
            margin: taffy::Rect {
                top: taffy::LengthPercentageAuto::Length(self.margin[0]),
                right: taffy::LengthPercentageAuto::Length(self.margin[1]),
                bottom: taffy::LengthPercentageAuto::Length(self.margin[2]),
                left: taffy::LengthPercentageAuto::Length(self.margin[3]),
            },
            ..taffy::Style::default()
        }
    }
}

/// A layout tree: a flat list of nodes (root is index 0).
pub struct LayoutTree {
    /// All nodes; index 0 is the root.
    pub nodes: Vec<LayoutNode>,
}

impl LayoutTree {
    /// Construct an empty tree with just a root.
    pub fn new(root: LayoutNode) -> Self {
        Self { nodes: vec![root] }
    }

    /// Append a child to `parent_idx` and return its index.
    pub fn add_child(&mut self, parent_idx: usize, node: LayoutNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.nodes[parent_idx].children.push(idx);
        idx
    }

    /// Compute layout for a viewport of `width` x `height` cells. Returns one
    /// [`Rect`] per node in `self.nodes`, in the same order.
    pub fn compute(&self, width: f32, height: f32) -> Vec<Rect> {
        let mut taffy: taffy::TaffyTree<()> = taffy::TaffyTree::new();
        let mut ids = Vec::with_capacity(self.nodes.len());
        for n in &self.nodes {
            let style = n.to_taffy();
            let id = taffy.new_leaf(style).expect("taffy new_leaf");
            ids.push(id);
        }
        // Build hierarchy.
        for (i, n) in self.nodes.iter().enumerate() {
            if !n.children.is_empty() {
                let children: Vec<_> = n.children.iter().map(|&c| ids[c]).collect();
                taffy
                    .set_children(ids[i], &children)
                    .expect("taffy set_children");
            }
        }
        taffy
            .compute_layout(
                ids[0],
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(width),
                    height: taffy::AvailableSpace::Definite(height),
                },
            )
            .expect("taffy compute_layout");

        self.nodes
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let l = taffy.layout(ids[i]).expect("taffy layout");
                Rect::new(
                    l.location.x.round().max(0.0) as u16,
                    l.location.y.round().max(0.0) as u16,
                    l.size.width.round().max(0.0) as u16,
                    l.size.height.round().max(0.0) as u16,
                )
            })
            .collect()
    }
}

/// A reusable set of flex properties. Widgets that participate in flex
/// layout embed one of these and call [`FlexProps::to_node`] in their
/// [`crate::widgets::Widget::layout`] implementation.
#[derive(Clone, Debug, Default)]
pub struct FlexProps {
    /// Direction.
    pub direction: FlexDirection,
    /// Cross-axis alignment.
    pub align: Align,
    /// Main-axis justification.
    pub justify: Justify,
    /// Width.
    pub width: Length,
    /// Height.
    pub height: Length,
    /// Flex grow.
    pub grow: f32,
    /// Padding (top, right, bottom, left).
    pub padding: [f32; 4],
    /// Margin (top, right, bottom, left).
    pub margin: [f32; 4],
}

impl FlexProps {
    /// Default column flex props.
    pub fn column() -> Self {
        Self::default()
    }
    /// Default row flex props.
    pub fn row() -> Self {
        Self {
            direction: FlexDirection::Row,
            ..Self::default()
        }
    }
    /// Set flex direction.
    #[must_use]
    pub fn direction(mut self, d: FlexDirection) -> Self {
        self.direction = d;
        self
    }
    /// Set cross-axis alignment.
    #[must_use]
    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }
    /// Set main-axis justification.
    #[must_use]
    pub fn justify(mut self, j: Justify) -> Self {
        self.justify = j;
        self
    }
    /// Set width.
    #[must_use]
    pub fn width(mut self, w: Length) -> Self {
        self.width = w;
        self
    }
    /// Set height.
    #[must_use]
    pub fn height(mut self, h: Length) -> Self {
        self.height = h;
        self
    }
    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
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
    /// Set margin (top, right, bottom, left).
    #[must_use]
    pub fn margin(mut self, trbl: [f32; 4]) -> Self {
        self.margin = trbl;
        self
    }
    /// Convert to a [`LayoutNode`] with the given child indices.
    pub fn to_node(&self, children: Vec<usize>) -> LayoutNode {
        LayoutNode {
            direction: self.direction,
            align: self.align,
            justify: self.justify,
            width: self.width,
            height: self.height,
            grow: self.grow,
            shrink: 1.0,
            basis: Length::Auto,
            padding: self.padding,
            margin: self.margin,
            children,
        }
    }
}
