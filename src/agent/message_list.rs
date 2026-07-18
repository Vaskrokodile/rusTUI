//! The `MessageList` widget: a scrollable transcript of chat messages.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::text::Spans;
use crate::widgets::base::{PaintCtx, Widget};

/// Role of a message author.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageRole {
    /// The user / operator.
    User,
    /// The assistant / agent.
    Assistant,
    /// A system message.
    System,
    /// A tool result.
    Tool,
}

impl MessageRole {
    fn label(self) -> &'static str {
        match self {
            Self::User => "you",
            Self::Assistant => "agent",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }
    fn color(self) -> Color {
        match self {
            Self::User => Color::BLUE,
            Self::Assistant => Color::GREEN,
            Self::System => Color::YELLOW,
            Self::Tool => Color::MAGENTA,
        }
    }
}

/// A single chat message.
#[derive(Clone, Debug)]
pub struct Message {
    /// Author role.
    pub role: MessageRole,
    /// Message body.
    pub content: Spans,
}

impl Message {
    /// Construct a message.
    pub fn new(role: MessageRole, content: impl Into<Spans>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

/// A scrollable list of chat messages.
///
/// The scroll offset lives in your [`crate::app::Context`] state — pass it in
/// via [`MessageList::scroll`] each frame. Set `scroll` to `usize::MAX` (or
/// any large number) to stick to the bottom.
pub struct MessageList {
    /// The messages, oldest first.
    pub messages: Vec<Message>,
    /// Top-visible row offset. Large values pin to the bottom.
    pub scroll: usize,
    /// Flex properties.
    pub flex: FlexProps,
}

impl MessageList {
    /// Construct an empty message list.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll: 0,
            flex: FlexProps::column(),
        }
    }

    /// Construct from messages.
    pub fn from_messages(messages: impl IntoIterator<Item = Message>) -> Self {
        Self {
            messages: messages.into_iter().collect(),
            ..Self::new()
        }
    }

    /// Set the scroll offset.
    #[must_use]
    pub fn scroll(mut self, s: usize) -> Self {
        self.scroll = s;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.flex.grow = g;
        self
    }
}

impl Default for MessageList {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MessageList {
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
        // First pass: compute total height needed (label row + content rows).
        // We render each message as: a label line, then the wrapped content.
        let mut row_heights: Vec<usize> = Vec::with_capacity(self.messages.len());
        for msg in &self.messages {
            let label_h = 1;
            let content_h = wrapped_height(&msg.content, w as usize);
            row_heights.push(label_h + content_h);
        }
        let total: usize = row_heights.iter().sum();
        let scroll = self.scroll.min(total.saturating_sub(h as usize));
        let mut skipped = 0usize;
        let mut row = y;
        for (i, msg) in self.messages.iter().enumerate() {
            let h_i = row_heights[i];
            if skipped + h_i <= scroll {
                skipped += h_i;
                continue;
            }
            // Partially skipped message: render label/content, skipping rows.
            let skip_in_msg = scroll - skipped;
            skipped += h_i;
            // Label row.
            if skip_in_msg == 0 {
                let label_style = Style::empty().fg(msg.role.color()).bold();
                ctx.buffer.print(x, row, msg.role.label(), label_style);
                let dim = Style::empty().fg(Color::palette256(8));
                ctx.buffer
                    .print(x + msg.role.label().len() as u16 + 1, row, "›", dim);
                row += 1;
                if row >= y + h {
                    return;
                }
            }
            // Content rows (wrapped).
            let mut cx = x;
            let mut cy = row;
            let mut local_row = if skip_in_msg > 0 {
                skip_in_msg - 1
            } else {
                0usize
            };
            for span in &msg.content.spans {
                for g in crate::unicode::graphemes(&span.text) {
                    let gw = crate::unicode::grapheme_width(g);
                    if gw == 0 {
                        continue;
                    }
                    if cx + gw as u16 > x + w {
                        cx = x;
                        cy += 1;
                        local_row += 1;
                        if cy >= y + h {
                            return;
                        }
                    }
                    if local_row >= if skip_in_msg > 0 { skip_in_msg - 1 } else { 0 } {
                        if cx + gw as u16 <= x + w {
                            ctx.buffer.print(cx, cy, g, span.style);
                        }
                    }
                    cx += gw as u16;
                }
            }
            row = cy + 1;
            if row >= y + h {
                return;
            }
        }
    }
}

fn wrapped_height(content: &Spans, max_width: usize) -> usize {
    if max_width == 0 {
        return 0;
    }
    let mut lines = 1usize;
    let mut width = 0usize;
    for span in &content.spans {
        for g in crate::unicode::graphemes(&span.text) {
            let gw = crate::unicode::grapheme_width(g);
            if width + gw > max_width {
                lines += 1;
                width = gw;
            } else {
                width += gw;
            }
        }
    }
    lines
}
