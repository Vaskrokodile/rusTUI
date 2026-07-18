//! The `Tree` widget: a hierarchical, collapsible tree view.
//!
//! Useful for file trees, directory structures, JSON outlines, and any other
//! nested data that benefits from expand/collapse interaction. The tree is
//! stateless from the widget's perspective: which nodes are expanded and which
//! path is selected live in your application state and are fed in each frame.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A single node in a [`Tree`].
///
/// Nodes are either *branches* (which may have children and an expanded flag)
/// or *leaves* (which never have children). Use [`TreeNode::new`] for a branch
/// and [`TreeNode::leaf`] for a leaf.
pub struct TreeNode {
    /// Label text for this node.
    pub label: compact_str::CompactString,
    /// Whether this node is expanded (for nodes with children).
    pub expanded: bool,
    /// Whether this node is a leaf (no children).
    pub is_leaf: bool,
    /// Children.
    pub children: Vec<TreeNode>,
    /// Optional icon/character prefix.
    pub icon: Option<compact_str::CompactString>,
    /// Optional style override.
    pub style: Option<Style>,
}

impl TreeNode {
    /// Construct a branch node (may have children) with the given label.
    pub fn new(label: impl Into<compact_str::CompactString>) -> Self {
        Self {
            label: label.into(),
            expanded: false,
            is_leaf: false,
            children: Vec::new(),
            icon: None,
            style: None,
        }
    }

    /// Construct a leaf node (no children) with the given label.
    pub fn leaf(label: impl Into<compact_str::CompactString>) -> Self {
        Self {
            label: label.into(),
            expanded: false,
            is_leaf: true,
            children: Vec::new(),
            icon: None,
            style: None,
        }
    }

    /// Append a child node and return `self` for chaining.
    #[must_use]
    pub fn add_child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }

    /// Set the expanded state and return `self` for chaining.
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set an icon prefix and return `self` for chaining.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<compact_str::CompactString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set a style override and return `self` for chaining.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

/// A hierarchical, collapsible tree view widget.
///
/// The widget walks its [`TreeNode`] roots recursively, only descending into
/// expanded branches. The currently selected node is identified by a path of
/// child indices from a root (e.g. `vec![0, 2]` means "the third child of the
/// first root").
pub struct Tree {
    /// Root nodes.
    pub roots: Vec<TreeNode>,
    /// Currently selected node path (indices from root).
    pub selected: Vec<usize>,
    /// Style for selected node.
    pub selected_style: Style,
    /// Style for expanded folder icons.
    pub expanded_icon: compact_str::CompactString,
    /// Style for collapsed folder icons.
    pub collapsed_icon: compact_str::CompactString,
    /// Leaf icon.
    pub leaf_icon: compact_str::CompactString,
    /// Indent per level.
    pub indent: u16,
    /// Flex grow.
    pub grow: f32,
}

impl Tree {
    /// Construct an empty tree.
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            selected: Vec::new(),
            selected_style: Style::empty().bg(Color::BLUE).fg(Color::WHITE),
            expanded_icon: compact_str::CompactString::const_new("\u{25BE}"),
            collapsed_icon: compact_str::CompactString::const_new("\u{25B8}"),
            leaf_icon: compact_str::CompactString::const_new("\u{2022}"),
            indent: 2,
            grow: 0.0,
        }
    }

    /// Add a root node and return `self` for chaining.
    #[must_use]
    pub fn root(mut self, node: TreeNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Set the selected node path (indices from a root).
    #[must_use]
    pub fn selected(mut self, path: impl IntoIterator<Item = usize>) -> Self {
        self.selected = path.into_iter().collect();
        self
    }

    /// Set the style applied to the selected node.
    #[must_use]
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the icon drawn for expanded branches (default: `▾`).
    #[must_use]
    pub fn expanded_icon(mut self, icon: impl Into<compact_str::CompactString>) -> Self {
        self.expanded_icon = icon.into();
        self
    }

    /// Set the icon drawn for collapsed branches (default: `▸`).
    #[must_use]
    pub fn collapsed_icon(mut self, icon: impl Into<compact_str::CompactString>) -> Self {
        self.collapsed_icon = icon.into();
        self
    }

    /// Set the icon drawn for leaf nodes (default: `•`).
    #[must_use]
    pub fn leaf_icon(mut self, icon: impl Into<compact_str::CompactString>) -> Self {
        self.leaf_icon = icon.into();
        self
    }

    /// Set the indentation (in columns) per nesting level.
    #[must_use]
    pub fn indent(mut self, indent: u16) -> Self {
        self.indent = indent;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Tree {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        let mut node = props.to_node(Vec::new());
        // A tree wants to fill available height.
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        let mut row = y;
        let max_row = y + h;
        for (i, root) in self.roots.iter().enumerate() {
            if row >= max_row {
                break;
            }
            // `Some(path)` means this root is on the selected path; `None`
            // means it is not. An empty path inside `Some` marks the selected
            // node itself.
            let path: Option<&[usize]> = if self.selected.first() == Some(&i) {
                Some(&self.selected[1..])
            } else {
                None
            };
            row = paint_node(ctx, root, x, row, w, max_row, 0, self, path);
        }
    }
}

/// Recursively paint a single node and (if expanded) its children.
///
/// `remaining_path` is `Some(slice)` when this node lies on the selected path;
/// an empty slice inside `Some` means *this* node is the selected one. `None`
/// means the node is not on the selected path.
///
/// Returns the next available row after painting this subtree.
#[allow(clippy::too_many_arguments)]
fn paint_node(
    ctx: &mut PaintCtx,
    node: &TreeNode,
    x: u16,
    row: u16,
    w: u16,
    max_row: u16,
    level: u16,
    tree: &Tree,
    remaining_path: Option<&[usize]>,
) -> u16 {
    if row >= max_row {
        return row;
    }

    let indent = level.saturating_mul(tree.indent);
    let row_start = x.saturating_add(indent);
    let mut cx = row_start;

    // Determine whether this node is the selected one.
    let is_selected = remaining_path == Some(&[]);
    let base = if is_selected {
        tree.selected_style
    } else {
        node.style.unwrap_or(Style::empty())
    };

    // Fill the entire row background (from the indent) when selected.
    if is_selected && base.bg != Color::TRANSPARENT && row_start < x + w {
        ctx.buffer
            .fill_rect(Rect::new(row_start, row, x + w - row_start, 1), base.bg);
    }

    // Draw the icon (expanded/collapsed for branches, leaf icon for leaves).
    let icon_style = base;
    if node.is_leaf {
        let icon = node.icon.as_deref().unwrap_or(tree.leaf_icon.as_str());
        cx = print_clipped(ctx, cx, row, x, w, icon, icon_style);
    } else {
        let icon = if node.expanded {
            node.icon.as_deref().unwrap_or(tree.expanded_icon.as_str())
        } else {
            node.icon.as_deref().unwrap_or(tree.collapsed_icon.as_str())
        };
        cx = print_clipped(ctx, cx, row, x, w, icon, icon_style);
    }

    // Single space between icon and label.
    cx = print_clipped(ctx, cx, row, x, w, " ", base);

    // Draw the label.
    let _ = print_clipped(ctx, cx, row, x, w, node.label.as_str(), base);

    let mut next_row = row + 1;

    // Descend into expanded branches.
    if !node.is_leaf && node.expanded {
        for (i, child) in node.children.iter().enumerate() {
            if next_row >= max_row {
                break;
            }
            // Compute the child's remaining path: only children on the selected
            // path receive `Some`.
            let child_path = match remaining_path {
                Some(path) if path.first() == Some(&i) => Some(&path[1..]),
                _ => None,
            };
            next_row = paint_node(
                ctx,
                child,
                x,
                next_row,
                w,
                max_row,
                level + 1,
                tree,
                child_path,
            );
        }
    }

    next_row
}

/// Print a string clipped to the widget's rect, returning the new column.
fn print_clipped(
    ctx: &mut PaintCtx,
    cx: u16,
    row: u16,
    x: u16,
    w: u16,
    text: &str,
    style: Style,
) -> u16 {
    let right = x.saturating_add(w);
    let mut cx = cx;
    for g in crate::unicode::graphemes(text) {
        let gw = crate::unicode::grapheme_width(g) as u16;
        if gw == 0 {
            continue;
        }
        if cx + gw > right {
            break;
        }
        ctx.buffer.print(cx, row, g, style);
        cx += gw;
    }
    cx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use std::time::Duration;

    fn paint_widget(widget: &impl Widget, w: u16, h: u16) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rects = vec![Rect::new(0, 0, w, h)];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect: rects[0],
            rects: &rects,
            elapsed: Duration::from_secs(0),
        };
        widget.paint(&mut ctx);
        buf
    }

    #[test]
    fn leaf_node_has_no_children() {
        let node = TreeNode::leaf("file.txt");
        assert!(node.is_leaf);
        assert!(node.children.is_empty());
        assert!(!node.expanded);
    }

    #[test]
    fn branch_node_can_have_children() {
        let node = TreeNode::new("dir")
            .expanded(true)
            .add_child(TreeNode::leaf("a.txt"))
            .add_child(TreeNode::leaf("b.txt"));
        assert!(!node.is_leaf);
        assert!(node.expanded);
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn collapsed_branch_hides_children() {
        let tree = Tree::new().root(
            TreeNode::new("dir")
                .expanded(false)
                .add_child(TreeNode::leaf("child.txt")),
        );
        let buf = paint_widget(&tree, 20, 5);
        // Row 0: collapsed icon + "dir"
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "\u{25B8}");
        // Row 1 should be empty (children hidden).
        assert!(buf.cell(0, 1).unwrap().grapheme.is_empty());
    }

    #[test]
    fn expanded_branch_shows_children() {
        let tree = Tree::new().root(
            TreeNode::new("dir")
                .expanded(true)
                .add_child(TreeNode::leaf("child.txt")),
        );
        let buf = paint_widget(&tree, 20, 5);
        // Row 0: expanded icon + "dir"
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "\u{25BE}");
        // Row 1: leaf icon + "child.txt" (indented by 2)
        assert_eq!(buf.cell(2, 1).unwrap().grapheme, "\u{2022}");
        assert_eq!(buf.cell(4, 1).unwrap().grapheme, "c");
    }

    #[test]
    fn selected_node_is_highlighted() {
        let tree = Tree::new()
            .root(TreeNode::leaf("first"))
            .root(TreeNode::leaf("second"))
            .selected([1]);
        let buf = paint_widget(&tree, 20, 5);
        // The second root (row 1) should have the selected background.
        assert_eq!(buf.cell(0, 1).unwrap().style.bg, Color::BLUE);
        // The first root (row 0) should not.
        assert_ne!(buf.cell(0, 0).unwrap().style.bg, Color::BLUE);
    }

    #[test]
    fn nested_selected_node_is_highlighted() {
        let tree = Tree::new().root(
            TreeNode::new("dir")
                .expanded(true)
                .add_child(TreeNode::leaf("a.txt"))
                .add_child(TreeNode::leaf("b.txt")),
        );
        // Select "b.txt": root 0, child 1.
        let buf = paint_widget(&tree.selected([0, 1]), 20, 5);
        // Row 0 is "dir" (not selected).
        assert_ne!(buf.cell(0, 0).unwrap().style.bg, Color::BLUE);
        // Row 2 is "b.txt" (selected): indented leaf icon at col 2.
        assert_eq!(buf.cell(2, 2).unwrap().style.bg, Color::BLUE);
        // Row 1 is "a.txt" (not selected).
        assert_ne!(buf.cell(2, 1).unwrap().style.bg, Color::BLUE);
    }
}
