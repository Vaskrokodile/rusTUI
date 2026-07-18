//! The `Spinner` widget: an animated indicator for in-progress work.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// Visual style for a [`Spinner`].
#[derive(Clone, Copy, Debug)]
pub enum SpinnerStyle {
    /// Braille dots rotating clockwise.
    Braille,
    /// ASCII line spinner.
    Line,
    /// Box-drawing blocks filling clockwise.
    Box,
    /// A simple dot that pulses.
    Pulse,
}

impl SpinnerStyle {
    fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Line => &["|", "/", "-", "\\"],
            Self::Box => &["▖", "▘", "▝", "▗"],
            Self::Pulse => &["●", "◉", "○"],
        }
    }
}

/// An animated spinner. Renders a single glyph that cycles through frames
/// based on `ctx.elapsed`.
pub struct Spinner {
    /// Visual style.
    pub style_kind: SpinnerStyle,
    /// Color.
    pub color: Color,
    /// Whether the spinner is currently spinning. When `false`, renders the
    /// first frame statically.
    pub spinning: bool,
    /// Frame interval (milliseconds per frame).
    pub interval_ms: u64,
    /// Flex properties.
    pub flex: FlexProps,
}

impl Spinner {
    /// Construct a braille spinner.
    pub fn new() -> Self {
        Self {
            style_kind: SpinnerStyle::Braille,
            color: Color::CYAN,
            spinning: true,
            interval_ms: 80,
            flex: FlexProps::column(),
        }
    }

    /// Set the visual style.
    #[must_use]
    pub fn style(mut self, s: SpinnerStyle) -> Self {
        self.style_kind = s;
        self
    }

    /// Set the color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Set whether the spinner is spinning.
    #[must_use]
    pub fn spinning(mut self, s: bool) -> Self {
        self.spinning = s;
        self
    }

    /// Set the frame interval in milliseconds.
    #[must_use]
    pub fn interval(mut self, ms: u64) -> Self {
        self.interval_ms = ms;
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn layout(&self) -> LayoutNode {
        let mut node = self.flex.to_node(Vec::new());
        // A spinner is exactly 1 cell wide.
        node.width = crate::layout::Length::Fixed(1.0);
        node.height = crate::layout::Length::Fixed(1.0);
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, .. } = ctx.rect;
        let frames = self.style_kind.frames();
        let frame = if self.spinning {
            let ms = ctx.elapsed.as_millis() as u64;
            let idx = ((ms / self.interval_ms) % frames.len() as u64) as usize;
            frames[idx]
        } else {
            frames[0]
        };
        let style = Style::empty().fg(self.color);
        ctx.buffer.print(x, y, frame, style);
    }
}
