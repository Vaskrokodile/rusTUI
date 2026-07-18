//! The `CommandPalette` widget: a fuzzy-searchable command menu overlay.
//!
//! Renders a centered popup with a search input and a filtered list of
//! commands. The user types to filter, and arrow keys navigate the results.
//! This is inspired by VS Code's Ctrl+P / Ctrl+Shift+P command palette.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A command entry in the palette.
#[derive(Clone)]
pub struct Command {
    /// Display label.
    pub label: compact_str::CompactString,
    /// Optional description/tooltip.
    pub description: Option<compact_str::CompactString>,
    /// Optional keyboard shortcut display.
    pub shortcut: Option<compact_str::CompactString>,
    /// Optional category for grouping.
    pub category: Option<compact_str::CompactString>,
}

impl Command {
    /// Construct a command with a label.
    pub fn new(label: impl Into<compact_str::CompactString>) -> Self {
        Self {
            label: label.into(),
            description: None,
            shortcut: None,
            category: None,
        }
    }

    /// Set description.
    #[must_use]
    pub fn description(mut self, d: impl Into<compact_str::CompactString>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// Set keyboard shortcut display.
    #[must_use]
    pub fn shortcut(mut self, s: impl Into<compact_str::CompactString>) -> Self {
        self.shortcut = Some(s.into());
        self
    }

    /// Set category.
    #[must_use]
    pub fn category(mut self, c: impl Into<compact_str::CompactString>) -> Self {
        self.category = Some(c.into());
        self
    }
}

/// A command palette overlay with fuzzy search.
pub struct CommandPalette {
    /// All available commands.
    pub commands: Vec<Command>,
    /// Current search query.
    pub query: compact_str::CompactString,
    /// Index of the highlighted result.
    pub highlighted: usize,
    /// Maximum number of results to show.
    pub max_results: usize,
    /// Style for the palette border.
    pub border_style: Style,
    /// Style for the search prompt.
    pub prompt_style: Style,
    /// Style for highlighted result.
    pub highlighted_style: Style,
    /// Style for normal results.
    pub result_style: Style,
    /// Style for shortcut display.
    pub shortcut_style: Style,
    /// Background color.
    pub bg: Color,
    /// Width as percentage of screen (0.0 to 1.0).
    pub width_pct: f32,
}

impl CommandPalette {
    /// Construct a command palette with the given commands.
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            commands,
            query: compact_str::CompactString::new(""),
            highlighted: 0,
            max_results: 10,
            border_style: Style::empty().fg(Color::rgb(100, 180, 255)),
            prompt_style: Style::empty().fg(Color::rgb(100, 180, 255)),
            highlighted_style: Style::empty().bg(Color::rgb(40, 80, 120)).fg(Color::WHITE),
            result_style: Style::empty().fg(Color::rgb(200, 200, 200)),
            shortcut_style: Style::empty().fg(Color::rgb(120, 120, 120)),
            bg: Color::rgb(30, 30, 40),
            width_pct: 0.6,
        }
    }

    /// Set the search query.
    #[must_use]
    pub fn query(mut self, q: impl Into<compact_str::CompactString>) -> Self {
        self.query = q.into();
        self
    }

    /// Set the highlighted index.
    #[must_use]
    pub fn highlighted(mut self, i: usize) -> Self {
        self.highlighted = i;
        self
    }

    /// Set max results.
    #[must_use]
    pub fn max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    /// Set width percentage.
    #[must_use]
    pub fn width_pct(mut self, pct: f32) -> Self {
        self.width_pct = pct.clamp(0.1, 1.0);
        self
    }

    /// Get the filtered commands based on the current query.
    #[must_use]
    pub fn filtered(&self) -> Vec<&Command> {
        if self.query.is_empty() {
            return self.commands.iter().take(self.max_results).collect();
        }
        let query_lower = self.query.to_lowercase();
        let mut results: Vec<(i32, usize, &Command)> = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                let label_lower = cmd.label.to_lowercase();
                if fuzzy_match(&query_lower, &label_lower) {
                    let score = fuzzy_score(&query_lower, &label_lower);
                    Some((score, i, cmd))
                } else {
                    None
                }
            })
            .collect();
        // Sort by score descending, then by original index.
        results.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        results
            .into_iter()
            .map(|(_, _, cmd)| cmd)
            .take(self.max_results)
            .collect()
    }
}

impl Widget for CommandPalette {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = 1.0;
        let mut node = props.to_node(Vec::new());
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        // Compute palette width.
        let palette_w = ((f32::from(w) * self.width_pct) as u16).max(20).min(w);
        let palette_x = x + (w - palette_w) / 2;

        // Compute palette height based on results.
        let filtered = self.filtered();
        let result_count = filtered.len();
        let palette_h = (2 + result_count as u16 + 1).min(h); // border + prompt + results + border
        let palette_y = y;

        // Draw border.
        let border_rect = Rect::new(palette_x, palette_y, palette_w, palette_h);
        ctx.buffer.fill_rect(border_rect, self.bg);
        ctx.buffer.box_border(border_rect, self.border_style);

        // Draw search prompt.
        let prompt_y = palette_y + 1;
        let prompt_x = palette_x + 2;
        let prompt_text = format!("> {}", self.query);
        ctx.buffer
            .print(prompt_x, prompt_y, &prompt_text, self.prompt_style);

        // Draw cursor at end of query.
        let cursor_x = prompt_x + prompt_text.len() as u16;
        if cursor_x < palette_x + palette_w - 1 {
            ctx.buffer.print(cursor_x, prompt_y, "▏", self.prompt_style);
        }

        // Draw results.
        for (i, cmd) in filtered.iter().enumerate() {
            let row_y = prompt_y + 1 + i as u16;
            if row_y >= palette_y + palette_h - 1 {
                break;
            }
            let style = if i == self.highlighted {
                self.highlighted_style
            } else {
                self.result_style
            };

            // Highlight background for selected row.
            if i == self.highlighted {
                for cx in palette_x + 1..palette_x + palette_w - 1 {
                    if let Some(cell) = ctx.buffer.cell_mut(cx, row_y) {
                        cell.style.bg = self.highlighted_style.bg;
                    }
                }
            }

            // Draw label.
            let label = &cmd.label;
            let max_label_w = palette_w.saturating_sub(4);
            let label_w = crate::unicode::str_width(label) as u16;
            if label_w <= max_label_w {
                ctx.buffer.print(palette_x + 2, row_y, label, style);
            } else {
                // Truncate.
                let truncated = crate::wrap::truncate(
                    &crate::text::Spans::plain(label.as_str()),
                    usize::from(max_label_w),
                    "…",
                );
                let plain = truncated.to_plain();
                ctx.buffer.print(palette_x + 2, row_y, &plain, style);
            }

            // Draw shortcut on the right.
            if let Some(shortcut) = &cmd.shortcut {
                let sc_w = crate::unicode::str_width(shortcut) as u16;
                let sc_x = palette_x + palette_w - sc_w - 2;
                if sc_x > prompt_x + label_w {
                    ctx.buffer.print(sc_x, row_y, shortcut, self.shortcut_style);
                }
            }
        }
    }
}

/// Simple fuzzy match: checks if all characters of `query` appear in `target`
/// in order (subsequence match).
fn fuzzy_match(query: &str, target: &str) -> bool {
    let mut query_chars = query.chars().peekable();
    for target_ch in target.chars() {
        if let Some(&query_ch) = query_chars.peek() {
            if query_ch == target_ch {
                query_chars.next();
            }
        }
    }
    query_chars.peek().is_none()
}

/// Fuzzy match score: higher is better. Prefers consecutive matches and
/// matches at word boundaries.
fn fuzzy_score(query: &str, target: &str) -> i32 {
    let mut score = 0i32;
    let mut query_chars = query.chars().peekable();
    let mut prev_matched = false;
    let mut pos = 0u32;

    for target_ch in target.chars() {
        if let Some(&query_ch) = query_chars.peek() {
            if query_ch == target_ch {
                score += 10;
                // Bonus for consecutive matches.
                if prev_matched {
                    score += 15;
                }
                // Bonus for word boundary matches.
                if pos == 0 || target.chars().nth(pos as usize - 1) == Some(' ') {
                    score += 20;
                }
                prev_matched = true;
                query_chars.next();
            } else {
                prev_matched = false;
            }
        }
        pos += 1;
    }

    // Penalty for length difference.
    score -= (target.len().saturating_sub(query.len())) as i32;
    score
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
    fn fuzzy_match_subsequence() {
        assert!(fuzzy_match("abc", "aXXbYYc"));
        assert!(fuzzy_match("abc", "abcdefg"));
        assert!(!fuzzy_match("abc", "acb"));
        assert!(fuzzy_match("", "anything"));
    }

    #[test]
    fn fuzzy_score_consecutive_better() {
        let s1 = fuzzy_score("abc", "abc");
        let s2 = fuzzy_score("abc", "aXbYc");
        assert!(s1 > s2);
    }

    #[test]
    fn command_palette_renders() {
        let cmds = vec![
            Command::new("Open File").shortcut("Ctrl+P"),
            Command::new("Save File").shortcut("Ctrl+S"),
            Command::new("Quit").shortcut("Ctrl+Q"),
        ];
        let palette = CommandPalette::new(cmds);
        let buf = paint_widget(&palette, 60, 20);
        // Should have a border.
        assert_eq!(buf.cell(12, 0).unwrap().grapheme, "┌");
        // Should show all 3 commands (empty query = show all).
        assert_eq!(buf.cell(14, 2).unwrap().grapheme, "O");
    }

    #[test]
    fn command_palette_filters_by_query() {
        let cmds = vec![
            Command::new("Open File"),
            Command::new("Save File"),
            Command::new("Quit"),
        ];
        let palette = CommandPalette::new(cmds).query("sa");
        let filtered = palette.filtered();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].label, "Save File");
    }

    #[test]
    fn command_palette_empty_query_shows_all() {
        let cmds = vec![
            Command::new("Open File"),
            Command::new("Save File"),
            Command::new("Quit"),
        ];
        let palette = CommandPalette::new(cmds);
        let filtered = palette.filtered();
        assert_eq!(filtered.len(), 3);
    }
}
