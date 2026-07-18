//! The `Widget` trait and the widget-tree walker.

use crate::buffer::Rect;
use crate::layout::{LayoutNode, LayoutTree};

/// Stable identifier for a widget within a single frame's tree.
///
/// Indices are assigned during the tree walk and are valid only for the
/// duration of one frame. Widgets that need to keep state across frames should
/// key their state by a user-supplied id (see [`crate::app::Context::state`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WidgetId(pub usize);

/// Passed to [`Widget::paint`].
pub struct PaintCtx<'a> {
    /// The frame buffer to paint into.
    pub buffer: &'a mut crate::buffer::Buffer,
    /// The rect assigned to this widget by the layout pass.
    pub rect: Rect,
    /// The rects for every node in the tree, indexed by [`WidgetId`].
    pub rects: &'a [Rect],
    /// The elapsed time since the app started, for animations (spinners, etc.).
    pub elapsed: std::time::Duration,
}

/// A widget: a unit of UI that knows how to lay itself out and paint itself.
///
/// Widgets are built fresh each frame (immediate-mode style) and dropped after
/// painting. State that needs to persist across frames lives in
/// [`crate::app::Context`], not in the widget.
///
/// Container widgets override [`Widget::take_children`] to yield their children
/// to the tree walker. The walker assigns layout-node indices and fills in the
/// `children` field of the node returned by [`Widget::layout`]; widgets should
/// return a node with an empty `children` vector.
pub trait Widget: Send {
    /// Build the layout node for this widget.
    ///
    /// Return a node with an empty `children` vector — the walker fills in
    /// child indices after collecting children via [`Widget::take_children`].
    fn layout(&self) -> LayoutNode;

    /// Paint this widget into `ctx.buffer` within `ctx.rect`.
    fn paint(&self, ctx: &mut PaintCtx);

    /// Take ownership of this widget's children. The default implementation
    /// returns an empty vector (leaf widget). Container widgets override this
    /// to drain their children into the walker.
    fn take_children(&mut self) -> Vec<Box<dyn Widget>> {
        Vec::new()
    }
}

/// A built widget tree: a flat list of widgets (pre-order, root at index 0)
/// and the corresponding [`LayoutTree`].
pub struct WidgetTree {
    /// All widgets in pre-order; index 0 is the root.
    pub widgets: Vec<Box<dyn Widget>>,
    /// The corresponding layout tree.
    pub layout: LayoutTree,
}

impl WidgetTree {
    /// Build a widget tree from a root widget.
    ///
    /// Walks the tree in pre-order, calling [`Widget::layout`] on each node
    /// and [`Widget::take_children`] to descend. The root ends up at index 0.
    pub fn build(mut root: Box<dyn Widget>) -> Self {
        let mut widgets: Vec<Box<dyn Widget>> = Vec::new();
        let mut layout_nodes: Vec<LayoutNode> = Vec::new();
        build_recursive(&mut root, &mut widgets, &mut layout_nodes);
        debug_assert_eq!(widgets.len(), layout_nodes.len());
        Self {
            widgets,
            layout: LayoutTree {
                nodes: layout_nodes,
            },
        }
    }

    /// Compute layout rects for every widget given a viewport size.
    pub fn compute_rects(&self, width: f32, height: f32) -> Vec<Rect> {
        self.layout.compute(width, height)
    }

    /// Paint every widget into `buffer` using the pre-computed `rects`.
    pub fn paint(
        &self,
        buffer: &mut crate::buffer::Buffer,
        rects: &[Rect],
        elapsed: std::time::Duration,
    ) {
        for (i, w) in self.widgets.iter().enumerate() {
            let rect = rects.get(i).copied().unwrap_or_default();
            if rect.is_empty() {
                continue;
            }
            let mut ctx = PaintCtx {
                buffer,
                rect,
                rects,
                elapsed,
            };
            w.paint(&mut ctx);
        }
    }
}

fn build_recursive(
    w: &mut Box<dyn Widget>,
    widgets: &mut Vec<Box<dyn Widget>>,
    layout_nodes: &mut Vec<LayoutNode>,
) -> usize {
    let my_idx = widgets.len();
    // Take children before moving `w` into the vec.
    let children = w.take_children();
    // Build the layout node (children field will be filled by us).
    let node = w.layout();
    // Move the widget into the flat list.
    // We need to get an owned `Box<dyn Widget>` out of `&mut Box<dyn Widget>`.
    // Replace it with a placeholder; the caller no longer needs `w`.
    let placeholder: Box<dyn Widget> = Box::new(crate::widgets::Text::new(""));
    let owned = std::mem::replace(w, placeholder);
    widgets.push(owned);

    // Reserve our layout slot.
    layout_nodes.push(node);

    // Recurse into children, collecting their indices.
    let mut child_indices = Vec::with_capacity(children.len());
    for mut child in children {
        let ci = build_recursive(&mut child, widgets, layout_nodes);
        child_indices.push(ci);
    }
    layout_nodes[my_idx].children = child_indices;
    my_idx
}
