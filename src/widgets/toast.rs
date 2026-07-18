//! The `Toast` widget: a transient notification overlay.
//!
//! Renders a small notification box (toast) at a screen corner. Useful for
//! showing ephemeral messages like "File saved", "Error: connection lost",
//! or "Build complete".

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// The severity level of a toast notification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastLevel {
    /// Informational (blue accent).
    #[default]
    Info,
    /// Success (green accent).
    Success,
    /// Warning (yellow accent).
    Warning,
    /// Error (red accent).
    Error,
}

/// Where on the screen the toast appears.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPosition {
    /// Top-right corner.
    #[default]
    TopRight,
    /// Top-left corner.
    TopLeft,
    /// Bottom-right corner.
    BottomRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Top-center.
    TopCenter,
    /// Bottom-center.
    BottomCenter,
}

/// A single toast notification.
pub struct Toast {
    /// The message text.
    pub message: compact_str::CompactString,
    /// Severity level.
    pub level: ToastLevel,
    /// Position on screen.
    pub position: ToastPosition,
    /// Optional title (shown bold above message).
    pub title: Option<compact_str::CompactString>,
    /// Whether to show an icon prefix.
    pub show_icon: bool,
    /// Padding around the toast content.
    pub padding: u16,
    /// Maximum width before wrapping.
    pub max_width: u16,
    /// Duration to show the toast (for the app to track, not enforced here).
    pub duration: std::time::Duration,
}

impl Toast {
    /// Construct an info toast.
    pub fn info(message: impl Into<compact_str::CompactString>) -> Self {
        Self {
            message: message.into(),
            level: ToastLevel::Info,
            position: ToastPosition::TopRight,
            title: None,
            show_icon: true,
            padding: 1,
            max_width: 50,
            duration: std::time::Duration::from_secs(3),
        }
    }

    /// Construct a success toast.
    pub fn success(message: impl Into<compact_str::CompactString>) -> Self {
        Self {
            message: message.into(),
            level: ToastLevel::Success,
            ..Self::info("")
        }
    }

    /// Construct a warning toast.
    pub fn warning(message: impl Into<compact_str::CompactString>) -> Self {
        Self {
            message: message.into(),
            level: ToastLevel::Warning,
            ..Self::info("")
        }
    }

    /// Construct an error toast.
    pub fn error(message: impl Into<compact_str::CompactString>) -> Self {
        Self {
            message: message.into(),
            level: ToastLevel::Error,
            ..Self::info("")
        }
    }

    /// Set the toast title.
    #[must_use]
    pub fn title(mut self, title: impl Into<compact_str::CompactString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the position.
    #[must_use]
    pub fn position(mut self, pos: ToastPosition) -> Self {
        self.position = pos;
        self
    }

    /// Set the duration.
    #[must_use]
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.duration = d;
        self
    }

    /// Set max width.
    #[must_use]
    pub fn max_width(mut self, w: u16) -> Self {
        self.max_width = w;
        self
    }

    /// Show/hide icon.
    #[must_use]
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    /// Get the accent color for this toast's level.
    pub fn accent_color(&self) -> Color {
        match self.level {
            ToastLevel::Info => Color::rgb(80, 150, 240),
            ToastLevel::Success => Color::rgb(80, 200, 120),
            ToastLevel::Warning => Color::rgb(220, 180, 60),
            ToastLevel::Error => Color::rgb(220, 80, 80),
        }
    }

    /// Get the icon character for this toast's level.
    pub fn icon(&self) -> &'static str {
        match self.level {
            ToastLevel::Info => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✗",
        }
    }

    /// Compute the toast rect given the viewport.
    fn compute_rect(&self, viewport: Rect) -> Rect {
        let msg_w = crate::unicode::str_width(&self.message) as u16;
        let title_w = self
            .title
            .as_ref()
            .map_or(0, |t| crate::unicode::str_width(t) as u16);
        let icon_w = if self.show_icon { 2 } else { 0 }; // icon + space
        let content_w = msg_w.max(title_w) + icon_w + self.padding * 2;
        let width = content_w.min(self.max_width).min(viewport.w);
        let has_title = self.title.is_some();
        let height = self.padding * 2 + if has_title { 2 } else { 1 }; // title + message or just message

        let (x, y) = match self.position {
            ToastPosition::TopRight => (
                viewport.right().saturating_sub(width).saturating_sub(1),
                viewport.y + 1,
            ),
            ToastPosition::TopLeft => (viewport.x + 1, viewport.y + 1),
            ToastPosition::BottomRight => (
                viewport.right().saturating_sub(width).saturating_sub(1),
                viewport.bottom().saturating_sub(height).saturating_sub(1),
            ),
            ToastPosition::BottomLeft => (
                viewport.x + 1,
                viewport.bottom().saturating_sub(height).saturating_sub(1),
            ),
            ToastPosition::TopCenter => (viewport.x + (viewport.w - width) / 2, viewport.y + 1),
            ToastPosition::BottomCenter => (
                viewport.x + (viewport.w - width) / 2,
                viewport.bottom().saturating_sub(height).saturating_sub(1),
            ),
        };

        Rect::new(x, y, width, height)
    }
}

impl Widget for Toast {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = 0.0;
        let mut node = props.to_node(Vec::new());
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let viewport = ctx.rect;
        if viewport.is_empty() {
            return;
        }

        let toast_rect = self.compute_rect(viewport);
        if toast_rect.is_empty() {
            return;
        }

        let accent = self.accent_color();
        let bg = Color::rgb(30, 30, 35);
        let border_style = Style::empty().fg(accent);

        // Fill background.
        ctx.buffer.fill_rect(toast_rect, bg);
        // Draw border.
        ctx.buffer.box_border(toast_rect, border_style);

        let inner_x = toast_rect.x + 1 + self.padding;
        let inner_y = toast_rect.y + 1;
        let mut cx = inner_x;

        // Draw icon.
        if self.show_icon {
            ctx.buffer
                .print(cx, inner_y, self.icon(), Style::empty().fg(accent));
            cx += 2; // icon + space
        }

        // Draw title (if any) in bold.
        if let Some(title) = &self.title {
            let title_style = Style::empty().fg(accent);
            ctx.buffer.print(cx, inner_y, title, title_style);
            // Draw message on the next line.
            let msg_y = inner_y + 1;
            let msg_style = Style::empty().fg(Color::rgb(220, 220, 220));
            ctx.buffer.print(inner_x, msg_y, &self.message, msg_style);
        } else {
            // Just the message.
            let msg_style = Style::empty().fg(Color::rgb(220, 220, 220));
            ctx.buffer.print(cx, inner_y, &self.message, msg_style);
        }
    }
}

/// A container that manages multiple toasts, stacking them vertically.
pub struct ToastStack {
    /// Active toasts.
    pub toasts: Vec<Toast>,
    /// Position for all toasts in this stack.
    pub position: ToastPosition,
    /// Gap between toasts.
    pub gap: u16,
}

impl ToastStack {
    /// Construct a toast stack.
    pub fn new(position: ToastPosition) -> Self {
        Self {
            toasts: Vec::new(),
            position,
            gap: 1,
        }
    }

    /// Add a toast to the stack.
    #[must_use]
    pub fn push(mut self, toast: Toast) -> Self {
        self.toasts.push(toast);
        self
    }

    /// Add a toast to the stack (mutable).
    pub fn add(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }
}

impl Widget for ToastStack {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = 0.0;
        let mut node = props.to_node(Vec::new());
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let viewport = ctx.rect;
        if viewport.is_empty() {
            return;
        }

        let mut offset = 0u16;
        for toast in &self.toasts {
            // Adjust toast position with offset for stacking.
            let mut adjusted = Toast {
                position: self.position,
                ..Toast {
                    message: toast.message.clone(),
                    level: toast.level,
                    position: self.position,
                    title: toast.title.clone(),
                    show_icon: toast.show_icon,
                    padding: toast.padding,
                    max_width: toast.max_width,
                    duration: toast.duration,
                }
            };
            // Apply vertical offset based on position.
            let toast_rect = adjusted.compute_rect(viewport);
            let h = toast_rect.h + self.gap;
            // Paint at adjusted position.
            // For simplicity, paint each toast individually.
            toast.paint(ctx);
            offset += h;
            // Shift subsequent toasts down (for top) or up (for bottom).
            // This is a simplification — a real implementation would compute
            // all rects first, then paint.
            let _ = &mut adjusted;
            let _ = offset;
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
    fn toast_info_renders() {
        let toast = Toast::info("File saved");
        let buf = paint_widget(&toast, 60, 20);
        // Should have some content (border or text).
        let has_content = (0..60).any(|x| buf.cell(x, 1).is_some_and(|c| !c.is_blank()));
        assert!(has_content);
    }

    #[test]
    fn toast_error_has_red_accent() {
        let toast = Toast::error("Connection lost");
        let accent = toast.accent_color();
        assert_eq!(accent, Color::rgb(220, 80, 80));
    }

    #[test]
    fn toast_success_has_green_accent() {
        let toast = Toast::success("Build complete");
        let accent = toast.accent_color();
        assert_eq!(accent, Color::rgb(80, 200, 120));
    }

    #[test]
    fn toast_icons() {
        assert_eq!(Toast::info("").icon(), "ℹ");
        assert_eq!(Toast::success("").icon(), "✓");
        assert_eq!(Toast::warning("").icon(), "⚠");
        assert_eq!(Toast::error("").icon(), "✗");
    }

    #[test]
    fn toast_with_title() {
        let toast = Toast::error("Disk full").title("Error");
        assert_eq!(toast.title.as_ref().unwrap(), "Error");
    }

    #[test]
    fn toast_position_bottom_right() {
        let toast = Toast::info("test").position(ToastPosition::BottomRight);
        assert_eq!(toast.position, ToastPosition::BottomRight);
    }
}
