//! The `Gauge` widget: a horizontal progress bar.
//!
//! Displays a filled portion of a bar to represent progress (0.0 to 1.0).
//! Useful for showing task completion, file download progress, token usage,
//! etc.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A horizontal progress bar.
pub struct Gauge {
    /// Progress value in `0.0..=1.0`.
    pub ratio: f32,
    /// Style for the filled portion.
    pub filled_style: Style,
    /// Style for the unfilled portion.
    pub unfilled_style: Style,
    /// Optional label rendered centered on the bar.
    pub label: Option<compact_str::CompactString>,
    /// Label style (defaults to filled_style for contrast).
    pub label_style: Option<Style>,
    /// Whether to use block characters (█) or thin lines (━).
    pub use_blocks: bool,
    /// Flex grow.
    pub grow: f32,
    /// Fixed height (default: 1).
    pub height: u16,
}

impl Gauge {
    /// Construct a gauge with the given ratio (0.0 to 1.0).
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            filled_style: Style::empty().fg(Color::GREEN).bg(Color::rgb(40, 40, 40)),
            unfilled_style: Style::empty().bg(Color::rgb(40, 40, 40)),
            label: None,
            label_style: None,
            use_blocks: true,
            grow: 0.0,
            height: 1,
        }
    }

    /// Set the progress ratio (clamped to 0.0..=1.0).
    #[must_use]
    pub fn ratio(mut self, r: f32) -> Self {
        self.ratio = r.clamp(0.0, 1.0);
        self
    }

    /// Set the filled portion style.
    #[must_use]
    pub fn filled_style(mut self, s: Style) -> Self {
        self.filled_style = s;
        self
    }

    /// Set the unfilled portion style.
    #[must_use]
    pub fn unfilled_style(mut self, s: Style) -> Self {
        self.unfilled_style = s;
        self
    }

    /// Set the bar color (shorthand for filled fg + unfilled bg).
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.filled_style = self.filled_style.fg(c);
        self
    }

    /// Set a centered label on the bar.
    #[must_use]
    pub fn label(mut self, label: impl Into<compact_str::CompactString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the label style.
    #[must_use]
    pub fn label_style(mut self, s: Style) -> Self {
        self.label_style = Some(s);
        self
    }

    /// Use block characters (█) instead of thin lines (━).
    #[must_use]
    pub fn use_blocks(mut self, blocks: bool) -> Self {
        self.use_blocks = blocks;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }

    /// Set the bar height.
    #[must_use]
    pub fn height(mut self, h: u16) -> Self {
        self.height = h;
        self
    }
}

impl Widget for Gauge {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        let mut node = props.to_node(Vec::new());
        node.height = Length::Fixed(f32::from(self.height));
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        let filled_w = (f32::from(w) * self.ratio).round() as u16;
        let filled_w = filled_w.min(w);

        let fill_char = if self.use_blocks { "█" } else { "━" };
        let empty_char = if self.use_blocks { " " } else { "─" };

        // Draw the bar across all rows.
        for row in 0..h {
            let cy = y + row;
            // Filled portion.
            for cx in 0..filled_w {
                ctx.buffer.print(x + cx, cy, fill_char, self.filled_style);
            }
            // Unfilled portion.
            for cx in filled_w..w {
                ctx.buffer
                    .print(x + cx, cy, empty_char, self.unfilled_style);
            }
        }

        // Draw label if set.
        if let Some(label) = &self.label {
            let label_w = crate::unicode::str_width(label) as u16;
            if label_w < w {
                let label_x = x + (w - label_w) / 2;
                let label_y = y + h / 2;
                let style = self.label_style.unwrap_or(self.filled_style);
                // Render label, clipping to bar width.
                let mut cx = label_x;
                for g in crate::unicode::graphemes(label) {
                    let gw = crate::unicode::grapheme_width(g) as u16;
                    if cx + gw > x + w {
                        break;
                    }
                    // Choose style based on whether we're on the filled or unfilled part.
                    let on_filled = cx < x + filled_w;
                    let s = if on_filled {
                        style
                    } else {
                        // On unfilled part, use unfilled fg if label style doesn't set fg.
                        Style::empty()
                            .fg(self.unfilled_style.fg)
                            .bg(self.unfilled_style.bg)
                    };
                    ctx.buffer.print(cx, label_y, g, s);
                    cx += gw;
                }
            }
        }
    }
}

/// A line gauge that shows progress as a percentage on a single line.
pub struct LineGauge {
    /// Progress value in `0.0..=1.0`.
    pub ratio: f32,
    /// Label shown before the bar.
    pub label: Option<compact_str::CompactString>,
    /// Filled color.
    pub filled_color: Color,
    /// Unfilled color.
    pub unfilled_color: Color,
    /// Flex grow.
    pub grow: f32,
}

impl LineGauge {
    /// Construct a line gauge.
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            label: None,
            filled_color: Color::GREEN,
            unfilled_color: Color::rgb(60, 60, 60),
            grow: 0.0,
        }
    }

    /// Set the progress ratio.
    #[must_use]
    pub fn ratio(mut self, r: f32) -> Self {
        self.ratio = r.clamp(0.0, 1.0);
        self
    }

    /// Set the label.
    #[must_use]
    pub fn label(mut self, label: impl Into<compact_str::CompactString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set filled color.
    #[must_use]
    pub fn filled_color(mut self, c: Color) -> Self {
        self.filled_color = c;
        self
    }

    /// Set unfilled color.
    #[must_use]
    pub fn unfilled_color(mut self, c: Color) -> Self {
        self.unfilled_color = c;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }
}

impl Widget for LineGauge {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        let mut node = props.to_node(Vec::new());
        node.height = Length::Fixed(1.0);
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, .. } = ctx.rect;
        if w == 0 {
            return;
        }

        let mut cx = x;

        // Draw label if present.
        if let Some(label) = &self.label {
            let label_w = crate::unicode::str_width(label) as u16;
            if label_w + 1 < w {
                ctx.buffer
                    .print(cx, y, label, Style::empty().fg(self.filled_color));
                cx += label_w;
                ctx.buffer.print(cx, y, " ", Style::empty());
                cx += 1;
            }
        }

        let bar_w = w.saturating_sub(cx - x);
        if bar_w == 0 {
            return;
        }

        // Draw percentage text.
        let pct = (self.ratio * 100.0) as u16;
        let pct_text = format!("{pct:>3}% ");
        let pct_w = pct_text.len() as u16;

        // Draw percentage on the right.
        if pct_w + 1 < bar_w {
            let bar_actual_w = bar_w - pct_w - 1;
            let filled_w = (f32::from(bar_actual_w) * self.ratio).round() as u16;

            for i in 0..bar_actual_w {
                let c = if i < filled_w { "━" } else { "─" };
                let color = if i < filled_w {
                    self.filled_color
                } else {
                    self.unfilled_color
                };
                ctx.buffer.print(cx + i, y, c, Style::empty().fg(color));
            }
            ctx.buffer.print(cx + bar_actual_w, y, " ", Style::empty());
            ctx.buffer.print(
                cx + bar_actual_w + 1,
                y,
                &pct_text,
                Style::empty().fg(self.filled_color),
            );
        } else {
            // Not enough room — just show the bar.
            let filled_w = (f32::from(bar_w) * self.ratio).round() as u16;
            for i in 0..bar_w {
                let c = if i < filled_w { "━" } else { "─" };
                let color = if i < filled_w {
                    self.filled_color
                } else {
                    self.unfilled_color
                };
                ctx.buffer.print(cx + i, y, c, Style::empty().fg(color));
            }
        }
    }
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
    fn gauge_half_filled() {
        let g = Gauge::new(0.5);
        let buf = paint_widget(&g, 10, 1);
        // First 5 cells should be filled (█), last 5 empty.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "█");
        assert_eq!(buf.cell(4, 0).unwrap().grapheme, "█");
        assert_eq!(buf.cell(5, 0).unwrap().grapheme, " ");
    }

    #[test]
    fn gauge_zero() {
        let g = Gauge::new(0.0);
        let buf = paint_widget(&g, 10, 1);
        for i in 0..10 {
            assert_eq!(buf.cell(i, 0).unwrap().grapheme, " ");
        }
    }

    #[test]
    fn gauge_full() {
        let g = Gauge::new(1.0);
        let buf = paint_widget(&g, 10, 1);
        for i in 0..10 {
            assert_eq!(buf.cell(i, 0).unwrap().grapheme, "█");
        }
    }

    #[test]
    fn gauge_clamps_ratio() {
        let g = Gauge::new(1.5);
        assert!((g.ratio - 1.0).abs() < f32::EPSILON);
        let g2 = Gauge::new(-0.5);
        assert!(g2.ratio.abs() < f32::EPSILON);
    }

    #[test]
    fn gauge_with_label() {
        let g = Gauge::new(0.5).label("50%");
        let buf = paint_widget(&g, 20, 1);
        // Label "50%" should be centered at position 8-10.
        assert_eq!(buf.cell(8, 0).unwrap().grapheme, "5");
        assert_eq!(buf.cell(9, 0).unwrap().grapheme, "0");
        assert_eq!(buf.cell(10, 0).unwrap().grapheme, "%");
    }

    #[test]
    fn line_gauge_basic() {
        let g = LineGauge::new(0.5).label("Progress");
        let buf = paint_widget(&g, 30, 1);
        // Should have "Progress" at the start.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "P");
    }
}
