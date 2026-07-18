//! The `TextArea` widget: a multi-line text input with cursor movement.
//!
//! Supports multiple lines, cursor movement (up/down/left/right), word
//! wrapping, and optional line numbers. The text state and cursor position
//! are managed by the caller (stored in `Context::state`); the widget is a
//! stateless view.

use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::Style;
use crate::widgets::base::{PaintCtx, Widget};

/// A cursor position in a multi-line text buffer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorPos {
    /// Line index (0-based).
    pub line: usize,
    /// Column index (grapheme-based, 0-based).
    pub col: usize,
}

impl CursorPos {
    /// Construct a cursor position.
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// A multi-line text input with cursor movement.
pub struct TextArea {
    /// The text content, split into lines.
    pub lines: Vec<String>,
    /// Cursor position.
    pub cursor: CursorPos,
    /// Base style.
    pub style: Style,
    /// Cursor style.
    pub cursor_style: Style,
    /// Placeholder text shown when all lines are empty.
    pub placeholder: Option<String>,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
    /// Line number style.
    pub line_number_style: Style,
    /// Whether to enable word wrapping.
    pub word_wrap: bool,
    /// Scroll offset (lines from top).
    pub scroll: u16,
    /// Flex grow.
    pub grow: f32,
    /// Background color.
    pub bg: Color,
}

impl TextArea {
    /// Construct an empty text area.
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: CursorPos::default(),
            style: Style::empty(),
            cursor_style: Style::empty().bg(Color::WHITE).fg(Color::BLACK),
            placeholder: None,
            show_line_numbers: false,
            line_number_style: Style::empty().fg(Color::rgb(100, 100, 100)),
            word_wrap: false,
            scroll: 0,
            grow: 1.0,
            bg: Color::TRANSPARENT,
        }
    }

    /// Construct a text area from a multi-line string.
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = text.split('\n').map(String::from).collect();
        Self::new().lines(lines)
    }

    /// Set the lines.
    #[must_use]
    pub fn lines(mut self, lines: Vec<String>) -> Self {
        if lines.is_empty() {
            self.lines = vec![String::new()];
        } else {
            self.lines = lines;
        }
        self
    }

    /// Set the cursor position.
    #[must_use]
    pub fn cursor(mut self, line: usize, col: usize) -> Self {
        self.cursor = CursorPos::new(line, col);
        self
    }

    /// Set the base style.
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set the cursor style.
    #[must_use]
    pub fn cursor_style(mut self, s: Style) -> Self {
        self.cursor_style = s;
        self
    }

    /// Set placeholder text.
    #[must_use]
    pub fn placeholder(mut self, s: impl Into<String>) -> Self {
        self.placeholder = Some(s.into());
        self
    }

    /// Show or hide line numbers.
    #[must_use]
    pub fn line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Enable or disable word wrapping.
    #[must_use]
    pub fn word_wrap(mut self, enable: bool) -> Self {
        self.word_wrap = enable;
        self
    }

    /// Set scroll offset.
    #[must_use]
    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }

    /// Set background color.
    #[must_use]
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    /// Get the full text as a single string (lines joined by \n).
    #[must_use]
    pub fn to_plain(&self) -> String {
        self.lines.join("\n")
    }

    /// Get the number of lines.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Compute the line number gutter width.
    fn gutter_width(&self) -> u16 {
        if !self.show_line_numbers {
            return 0;
        }
        let digits = self.lines.len().to_string().len();
        digits as u16 + 1 // +1 for padding
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextArea {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
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

        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }

        let gutter_w = self.gutter_width();
        let text_x = x + gutter_w;
        let text_w = w.saturating_sub(gutter_w);
        if text_w == 0 {
            return;
        }

        // Check if all lines are empty (for placeholder).
        let all_empty = self.lines.iter().all(String::is_empty);

        if all_empty {
            let Some(ph) = &self.placeholder else { return };
            let ph_style = self.style.fg(Color::rgb(100, 100, 100));
            let mut cx = text_x;
            let mut cy = y;
            for g in crate::unicode::graphemes(ph) {
                let gw = crate::unicode::grapheme_width(g) as u16;
                if g == "\n" {
                    cx = text_x;
                    cy += 1;
                    if cy >= y + h {
                        break;
                    }
                    continue;
                }
                if gw == 0 || cx + gw > text_x + text_w {
                    if self.word_wrap && cx + gw > text_x + text_w {
                        cx = text_x;
                        cy += 1;
                        if cy >= y + h {
                            break;
                        }
                    } else {
                        continue;
                    }
                }
                if cy < y + h && cx + gw <= text_x + text_w {
                    ctx.buffer.print(cx, cy, g, ph_style);
                }
                cx += gw;
            }
            // Draw cursor at start.
            ctx.buffer.print(text_x, y, " ", self.cursor_style);
            return;
        }

        // Render each visible line.
        let start_line = usize::from(self.scroll).min(self.lines.len());
        let visible_lines = &self.lines[start_line..];
        let max_visible = usize::from(h);

        for (row, line_text) in visible_lines.iter().take(max_visible).enumerate() {
            let line_idx = start_line + row;
            let mut cy = y + row as u16;

            // Draw line number.
            if self.show_line_numbers {
                let num = format!("{:>width$}", line_idx + 1, width = gutter_w as usize - 1);
                ctx.buffer.print(x, cy, &num, self.line_number_style);
                // Draw separator.
                if gutter_w > 0 && x + gutter_w - 1 < x + w {
                    ctx.buffer
                        .print(x + gutter_w - 1, cy, " ", self.line_number_style);
                }
            }

            // Render the line text.
            let mut cx = text_x;
            let mut grapheme_col = 0usize;
            for g in crate::unicode::graphemes(line_text) {
                let gw = crate::unicode::grapheme_width(g) as u16;
                if gw == 0 {
                    continue;
                }

                // Check if cursor is at this position.
                let is_cursor = line_idx == self.cursor.line && grapheme_col == self.cursor.col;

                if self.word_wrap && cx + gw > text_x + text_w {
                    cx = text_x;
                    cy += 1;
                    if cy >= y + h {
                        break;
                    }
                }

                if cx + gw <= text_x + text_w && cy < y + h {
                    let style = if is_cursor {
                        self.cursor_style
                    } else {
                        self.style
                    };
                    ctx.buffer.print(cx, cy, g, style);
                }
                cx += gw;
                grapheme_col += 1;
            }

            // Handle cursor at end of line.
            if line_idx == self.cursor.line && grapheme_col == self.cursor.col {
                let cursor_x = text_x
                    + crate::unicode::str_width(
                        &line_text
                            .graphemes(true)
                            .take(self.cursor.col)
                            .collect::<String>(),
                    ) as u16;
                if cursor_x < text_x + text_w && cy < y + h {
                    ctx.buffer.print(cursor_x, cy, " ", self.cursor_style);
                }
            }
        }
    }
}

/// Helper functions for manipulating a multi-line text buffer.
#[allow(dead_code)]
pub mod ops {
    use super::CursorPos;
    use unicode_segmentation::UnicodeSegmentation;

    /// Insert a character at the cursor position, returning the new cursor.
    pub fn insert_char(lines: &mut Vec<String>, cursor: CursorPos, ch: char) -> CursorPos {
        if cursor.line >= lines.len() {
            lines.push(String::new());
        }
        let line = &mut lines[cursor.line];
        // Find byte offset for the grapheme column.
        let byte_offset = grapheme_col_to_byte(line, cursor.col);
        line.insert(byte_offset, ch);
        CursorPos::new(cursor.line, cursor.col + 1)
    }

    /// Insert a newline at the cursor position, splitting the current line.
    pub fn insert_newline(lines: &mut Vec<String>, cursor: CursorPos) -> CursorPos {
        if cursor.line >= lines.len() {
            lines.push(String::new());
            return CursorPos::new(cursor.line + 1, 0);
        }
        let line = &mut lines[cursor.line];
        let byte_offset = grapheme_col_to_byte(line, cursor.col);
        let rest: String = line[byte_offset..].graphemes(true).collect();
        line.truncate(byte_offset);
        lines.insert(cursor.line + 1, rest);
        CursorPos::new(cursor.line + 1, 0)
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(lines: &mut Vec<String>, cursor: CursorPos) -> CursorPos {
        if cursor.col > 0 {
            let line = &mut lines[cursor.line];
            let byte_offset = grapheme_col_to_byte(line, cursor.col);
            // Find the start of the previous grapheme.
            let prev_byte = prev_grapheme_boundary(line, byte_offset);
            line.replace_range(prev_byte..byte_offset, "");
            CursorPos::new(cursor.line, cursor.col - 1)
        } else if cursor.line > 0 {
            // Merge with previous line.
            let current_line = lines.remove(cursor.line);
            let prev_line = &mut lines[cursor.line - 1];
            let new_col = prev_line.graphemes(true).count();
            prev_line.push_str(&current_line);
            CursorPos::new(cursor.line - 1, new_col)
        } else {
            cursor
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete_char(lines: &mut Vec<String>, cursor: CursorPos) -> CursorPos {
        let line = &mut lines[cursor.line];
        let byte_offset = grapheme_col_to_byte(line, cursor.col);
        if byte_offset < line.len() {
            let next_byte = next_grapheme_boundary(line, byte_offset);
            line.replace_range(byte_offset..next_byte, "");
        } else if cursor.line + 1 < lines.len() {
            // Merge with next line.
            let next_line = lines.remove(cursor.line + 1);
            lines[cursor.line].push_str(&next_line);
        }
        cursor
    }

    /// Move cursor left by one grapheme.
    pub fn move_left(lines: &[String], cursor: CursorPos) -> CursorPos {
        if cursor.col > 0 {
            CursorPos::new(cursor.line, cursor.col - 1)
        } else if cursor.line > 0 {
            let prev_line_len = lines[cursor.line - 1].graphemes(true).count();
            CursorPos::new(cursor.line - 1, prev_line_len)
        } else {
            cursor
        }
    }

    /// Move cursor right by one grapheme.
    pub fn move_right(lines: &[String], cursor: CursorPos) -> CursorPos {
        let current_line_len = lines
            .get(cursor.line)
            .map_or(0, |l| l.graphemes(true).count());
        if cursor.col < current_line_len {
            CursorPos::new(cursor.line, cursor.col + 1)
        } else if cursor.line + 1 < lines.len() {
            CursorPos::new(cursor.line + 1, 0)
        } else {
            cursor
        }
    }

    /// Move cursor up one line.
    pub fn move_up(lines: &[String], cursor: CursorPos) -> CursorPos {
        if cursor.line > 0 {
            let prev_line_len = lines[cursor.line - 1].graphemes(true).count();
            CursorPos::new(cursor.line - 1, cursor.col.min(prev_line_len))
        } else {
            cursor
        }
    }

    /// Move cursor down one line.
    pub fn move_down(lines: &[String], cursor: CursorPos) -> CursorPos {
        if cursor.line + 1 < lines.len() {
            let next_line_len = lines[cursor.line + 1].graphemes(true).count();
            CursorPos::new(cursor.line + 1, cursor.col.min(next_line_len))
        } else {
            cursor
        }
    }

    /// Move cursor to start of line.
    pub fn move_line_start(cursor: CursorPos) -> CursorPos {
        CursorPos::new(cursor.line, 0)
    }

    /// Move cursor to end of line.
    pub fn move_line_end(lines: &[String], cursor: CursorPos) -> CursorPos {
        let len = lines
            .get(cursor.line)
            .map_or(0, |l| l.graphemes(true).count());
        CursorPos::new(cursor.line, len)
    }

    /// Convert a grapheme column to a byte offset within a string.
    fn grapheme_col_to_byte(s: &str, col: usize) -> usize {
        s.graphemes(true).take(col).map(str::len).sum()
    }

    /// Find the previous grapheme boundary before `byte_offset`.
    fn prev_grapheme_boundary(s: &str, byte_offset: usize) -> usize {
        if byte_offset == 0 {
            return 0;
        }
        let mut prev = 0;
        for (i, _) in s.char_indices() {
            if i >= byte_offset {
                break;
            }
            prev = i;
        }
        prev
    }

    /// Find the next grapheme boundary after `byte_offset`.
    fn next_grapheme_boundary(s: &str, byte_offset: usize) -> usize {
        for (i, _) in s.char_indices() {
            if i > byte_offset {
                return i;
            }
        }
        s.len()
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
    fn text_area_renders_multiple_lines() {
        let ta = TextArea::from_text("hello\nworld\nfoo");
        let buf = paint_widget(&ta, 20, 3);
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "h");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "w");
        assert_eq!(buf.cell(0, 2).unwrap().grapheme, "f");
    }

    #[test]
    fn text_area_cursor_visible() {
        let ta = TextArea::from_text("hello").cursor(0, 2);
        let buf = paint_widget(&ta, 20, 1);
        // Cursor at position 2 should have inverted style.
        let cell = buf.cell(2, 0).unwrap();
        assert_eq!(cell.grapheme, "l");
        // Cursor style has white bg.
        assert_eq!(cell.style.bg, Color::WHITE);
    }

    #[test]
    fn text_area_placeholder() {
        let ta = TextArea::new().placeholder("Type here...");
        let buf = paint_widget(&ta, 20, 1);
        // Placeholder should be visible (cursor is drawn at position 0,
        // so the first cell is the cursor, but "T" should be at position 1
        // since the cursor is a space that overwrites position 0).
        // Actually the cursor draws a space at position 0, so we check
        // that the second character is "y" (from "Type").
        // The cursor block is at position 0, so check position 1.
        assert_eq!(buf.cell(1, 0).unwrap().grapheme, "y");
    }

    #[test]
    fn text_area_line_numbers() {
        let ta = TextArea::from_text("hello\nworld").line_numbers(true);
        let buf = paint_widget(&ta, 20, 2);
        // Line number "1" at position 0.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "1");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "2");
    }

    #[test]
    fn ops_insert_char() {
        let mut lines = vec!["hello".to_string()];
        let cursor = ops::insert_char(&mut lines, CursorPos::new(0, 2), 'X');
        assert_eq!(lines[0], "heXllo");
        assert_eq!(cursor, CursorPos::new(0, 3));
    }

    #[test]
    fn ops_insert_newline() {
        let mut lines = vec!["hello".to_string()];
        let cursor = ops::insert_newline(&mut lines, CursorPos::new(0, 2));
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "he");
        assert_eq!(lines[1], "llo");
        assert_eq!(cursor, CursorPos::new(1, 0));
    }

    #[test]
    fn ops_backspace() {
        let mut lines = vec!["hello".to_string()];
        let cursor = ops::backspace(&mut lines, CursorPos::new(0, 5));
        assert_eq!(lines[0], "hell");
        assert_eq!(cursor, CursorPos::new(0, 4));
    }

    #[test]
    fn ops_backspace_at_line_start_merges() {
        let mut lines = vec!["hello".to_string(), "world".to_string()];
        let cursor = ops::backspace(&mut lines, CursorPos::new(1, 0));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "helloworld");
        assert_eq!(cursor, CursorPos::new(0, 5));
    }

    #[test]
    fn ops_move_left_right() {
        let lines = vec!["hello".to_string()];
        let c = ops::move_right(&lines, CursorPos::new(0, 0));
        assert_eq!(c, CursorPos::new(0, 1));
        let c = ops::move_left(&lines, c);
        assert_eq!(c, CursorPos::new(0, 0));
    }

    #[test]
    fn ops_move_up_down() {
        let lines = vec!["hello".to_string(), "world".to_string()];
        let c = ops::move_down(&lines, CursorPos::new(0, 2));
        assert_eq!(c, CursorPos::new(1, 2));
        let c = ops::move_up(&lines, c);
        assert_eq!(c, CursorPos::new(0, 2));
    }

    #[test]
    fn ops_move_up_clamps_col() {
        let lines = vec!["hello world".to_string(), "hi".to_string()];
        let c = ops::move_up(&lines, CursorPos::new(1, 8));
        // Column 8 should clamp to line 0's length (11).
        assert_eq!(c, CursorPos::new(0, 8));
    }
}
