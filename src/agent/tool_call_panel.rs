//! The `ToolCallPanel` widget: shows a list of tool calls with status.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// Status of a single tool call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolCallStatus {
    /// The agent is about to invoke the tool.
    Pending,
    /// The tool is currently running.
    Running,
    /// The tool completed successfully.
    Success,
    /// The tool failed.
    Failed,
    /// The user was asked to approve this call and hasn't responded yet.
    AwaitingApproval,
    /// The user rejected this call.
    Rejected,
}

impl ToolCallStatus {
    fn glyph(self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "◐",
            Self::Success => "✓",
            Self::Failed => "✗",
            Self::AwaitingApproval => "?",
            Self::Rejected => "⊘",
        }
    }
    fn color(self) -> Color {
        match self {
            Self::Pending => Color::palette256(8),
            Self::Running => Color::YELLOW,
            Self::Success => Color::GREEN,
            Self::Failed => Color::RED,
            Self::AwaitingApproval => Color::MAGENTA,
            Self::Rejected => Color::palette256(8),
        }
    }
}

/// A single tool call entry.
#[derive(Clone, Debug)]
pub struct ToolCall {
    /// Tool name, e.g. `read_file` or `bash`.
    pub name: String,
    /// A short summary of the arguments, e.g. the file path or command.
    pub summary: String,
    /// Current status.
    pub status: ToolCallStatus,
    /// Optional result preview (truncated by the widget to fit).
    pub result: Option<String>,
}

impl ToolCall {
    /// Construct a tool call with the given name, summary, and status.
    pub fn new(
        name: impl Into<String>,
        summary: impl Into<String>,
        status: ToolCallStatus,
    ) -> Self {
        Self {
            name: name.into(),
            summary: summary.into(),
            status,
            result: None,
        }
    }

    /// Attach a result preview.
    #[must_use]
    pub fn result(mut self, r: impl Into<String>) -> Self {
        self.result = Some(r.into());
        self
    }
}

/// A panel that displays a list of tool calls.
///
/// Each row shows a status glyph, the tool name, and a summary. If a call has
/// a result, it's shown indented on the next line (truncated to fit).
pub struct ToolCallPanel {
    /// The tool calls, in order.
    pub calls: Vec<ToolCall>,
    /// Title shown at the top of the panel.
    pub title: Option<String>,
    /// Base style.
    pub style: Style,
    /// Flex properties.
    pub flex: FlexProps,
}

impl ToolCallPanel {
    /// Construct an empty panel.
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            title: None,
            style: Style::empty(),
            flex: FlexProps::column(),
        }
    }

    /// Construct a panel from a list of calls.
    pub fn from_calls(calls: impl IntoIterator<Item = ToolCall>) -> Self {
        Self {
            calls: calls.into_iter().collect(),
            ..Self::new()
        }
    }

    /// Set the panel title.
    #[must_use]
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Default for ToolCallPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ToolCallPanel {
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
        if let Some(title) = &self.title {
            let title_style = Style::empty().fg(Color::CYAN).bold();
            ctx.buffer.print(x, row, title, title_style);
            row += 1;
            if row >= y + h {
                return;
            }
        }
        for call in &self.calls {
            if row >= y + h {
                return;
            }
            let glyph_style = Style::empty().fg(call.status.color());
            ctx.buffer.print(x, row, call.status.glyph(), glyph_style);
            let name_style = Style::empty().fg(Color::WHITE).bold();
            ctx.buffer.print(x + 2, row, &call.name, name_style);
            let summary_style = Style::empty().fg(Color::palette256(7));
            // Truncate summary to fit.
            let remaining_cols = (w as usize).saturating_sub(2 + call.name.chars().count() + 1);
            let summary = truncate_to_width(&call.summary, remaining_cols);
            ctx.buffer.print(
                x + 2 + call.name.len() as u16 + 1,
                row,
                &summary,
                summary_style,
            );
            row += 1;
            if let Some(result) = &call.result {
                if row >= y + h {
                    return;
                }
                let result_style = Style::empty().fg(Color::palette256(8));
                let avail = (w as usize).saturating_sub(4);
                let truncated = truncate_to_width(result, avail);
                ctx.buffer.print(x + 4, row, &truncated, result_style);
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
