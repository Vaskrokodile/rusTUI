//! The `DiffViewer` widget: renders a unified diff with syntax coloring.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// Kind of line in a diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    /// Context line (unchanged).
    Context,
    /// Added line.
    Add,
    /// Removed line.
    Remove,
    /// Hunk header (`@@ ... @@`).
    Hunk,
    /// File header (`+++`/`---`).
    File,
}

/// A single line in a diff.
#[derive(Clone, Debug)]
pub struct DiffLine {
    /// Kind of line.
    pub kind: DiffLineKind,
    /// The line content (without the leading `+`/`-`/` ` sigil).
    pub content: String,
}

impl DiffLine {
    /// Construct a diff line.
    pub fn new(kind: DiffLineKind, content: impl Into<String>) -> Self {
        Self {
            kind,
            content: content.into(),
        }
    }
}

/// A hunk: a contiguous block of diff lines.
#[derive(Clone, Debug, Default)]
pub struct DiffHunk {
    /// Lines in this hunk, in order.
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Construct an empty hunk.
    pub fn new() -> Self {
        Self::default()
    }
    /// Push a line.
    pub fn push(&mut self, line: DiffLine) {
        self.lines.push(line);
    }
}

/// A widget that renders a unified diff with color-coded add/remove/context lines.
pub struct DiffViewer {
    /// The hunks to display, in order.
    pub hunks: Vec<DiffHunk>,
    /// Style for added lines.
    pub add_style: Style,
    /// Style for removed lines.
    pub remove_style: Style,
    /// Style for context lines.
    pub context_style: Style,
    /// Style for hunk headers.
    pub hunk_style: Style,
    /// Style for file headers.
    pub file_style: Style,
    /// Flex properties.
    pub flex: FlexProps,
}

impl DiffViewer {
    /// Construct an empty diff viewer.
    pub fn new() -> Self {
        Self {
            hunks: Vec::new(),
            add_style: Style::empty().fg(Color::GREEN),
            remove_style: Style::empty().fg(Color::RED),
            context_style: Style::empty().fg(Color::palette256(7)),
            hunk_style: Style::empty().fg(Color::CYAN).bold(),
            file_style: Style::empty().fg(Color::YELLOW).bold(),
            flex: FlexProps::column(),
        }
    }

    /// Construct a diff viewer from hunks.
    pub fn from_hunks(hunks: impl IntoIterator<Item = DiffHunk>) -> Self {
        Self {
            hunks: hunks.into_iter().collect(),
            ..Self::new()
        }
    }

    /// Parse a unified diff string into hunks.
    ///
    /// File headers (`---`/`+++`) are attached to the hunk that follows them.
    /// If a diff has file headers before any hunk, they become the first lines
    /// of the first hunk.
    pub fn parse(diff: &str) -> Self {
        let mut hunks: Vec<DiffHunk> = Vec::new();
        let mut current: Option<DiffHunk> = None;
        // Buffer file headers that appear before the first hunk.
        let mut pending_file_headers: Vec<DiffLine> = Vec::new();
        for line in diff.lines() {
            if line.starts_with("@@") {
                if let Some(h) = current.take() {
                    hunks.push(h);
                }
                let mut h = DiffHunk::new();
                // Attach any pending file headers to this hunk.
                for fh in pending_file_headers.drain(..) {
                    h.push(fh);
                }
                h.push(DiffLine::new(DiffLineKind::Hunk, line));
                current = Some(h);
            } else if line.starts_with("+++") || line.starts_with("---") {
                let file_line = DiffLine::new(DiffLineKind::File, line);
                if let Some(h) = current.as_mut() {
                    h.push(file_line);
                } else {
                    // Buffer until we see a hunk header.
                    pending_file_headers.push(file_line);
                }
            } else if let Some(h) = current.as_mut() {
                let (kind, content) = if let Some(rest) = line.strip_prefix('+') {
                    (DiffLineKind::Add, rest)
                } else if let Some(rest) = line.strip_prefix('-') {
                    (DiffLineKind::Remove, rest)
                } else if let Some(rest) = line.strip_prefix(' ') {
                    (DiffLineKind::Context, rest)
                } else {
                    (DiffLineKind::Context, line)
                };
                h.push(DiffLine::new(kind, content));
            }
        }
        if let Some(h) = current {
            hunks.push(h);
        }
        Self::from_hunks(hunks)
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Default for DiffViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DiffViewer {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }
        let mut row = y;
        for hunk in &self.hunks {
            for line in &hunk.lines {
                if row >= y + h {
                    return;
                }
                let (style, sigil) = match line.kind {
                    DiffLineKind::Context => (self.context_style, " "),
                    DiffLineKind::Add => (self.add_style, "+"),
                    DiffLineKind::Remove => (self.remove_style, "-"),
                    DiffLineKind::Hunk => (self.hunk_style, ""),
                    DiffLineKind::File => (self.file_style, ""),
                };
                // Background tint for add/remove.
                let bg = match line.kind {
                    DiffLineKind::Add => Color::rgba(40, 80, 50, 60),
                    DiffLineKind::Remove => Color::rgba(90, 30, 40, 60),
                    _ => Color::TRANSPARENT,
                };
                if bg != Color::TRANSPARENT {
                    ctx.buffer.fill_rect(Rect::new(x, row, w, 1), bg);
                }
                ctx.buffer.print(x, row, sigil, style);
                // Truncate content to fit.
                let avail = (w as usize).saturating_sub(1);
                let truncated = truncate_to_width(&line.content, avail);
                ctx.buffer.print(x + 1, row, &truncated, style);
                row += 1;
            }
        }
    }
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut width = 0usize;
    let mut out = String::new();
    for g in crate::unicode::graphemes(s) {
        let gw = crate::unicode::grapheme_width(g);
        if width + gw > max_width {
            out.push('…');
            break;
        }
        out.push_str(g);
        width += gw;
    }
    out
}
