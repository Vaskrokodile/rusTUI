//! The `Table` widget: a grid of tabular data with an optional header row.
//!
//! [`Table`] renders a header row followed by zero or more data rows. Column
//! widths can be set explicitly via [`Table::widths`]; if left empty the
//! available width is divided equally among the columns. One row may be
//! highlighted as "selected" via [`Table::selected`], and optional vertical
//! separators (`│`) can be drawn between columns via
//! [`Table::show_separators`]. The widget is stateless — the selected index
//! lives in your [`crate::app::Context`] state and is passed in each frame.

use crate::buffer::Rect;
use crate::color::Color;
use crate::layout::{FlexProps, LayoutNode, Length};
use crate::style::{Attr, Style};
use crate::widgets::base::{PaintCtx, Widget};

/// A widget that displays tabular data (rows and columns).
///
/// Useful for tool results, file listings, key/value summaries, and any other
/// data that fits a grid. Construct with [`Table::new`], then add a header via
/// [`Table::header`] and rows via [`Table::row`] / [`Table::rows`].
pub struct Table {
    /// Column widths (in terminal columns). If empty, columns are auto-sized
    /// equally across the widget's width.
    pub widths: Vec<u16>,
    /// Header row (optional). When set, it is rendered on the first line with
    /// [`Table::header_style`] (bold + underline by default).
    pub header: Vec<compact_str::CompactString>,
    /// Data rows. Each inner vector is one row; its length may differ from the
    /// number of columns — missing cells render as blank, extra cells are
    /// ignored.
    pub rows: Vec<Vec<compact_str::CompactString>>,
    /// Header style.
    pub header_style: Style,
    /// Row style (applied to all rows).
    pub row_style: Style,
    /// Selected row style (for highlight).
    pub selected_style: Style,
    /// Index of selected row (`None` = no selection). This indexes into
    /// [`Table::rows`]; the header is not counted.
    pub selected: Option<usize>,
    /// Whether to show vertical column separators (`│`).
    pub show_separators: bool,
    /// Flex grow.
    pub grow: f32,
    /// Background color.
    pub bg: Color,
}

impl Table {
    /// Construct an empty table.
    pub fn new() -> Self {
        Self {
            widths: Vec::new(),
            header: Vec::new(),
            rows: Vec::new(),
            header_style: Style::empty()
                .fg(Color::WHITE)
                .attr(Attr::BOLD | Attr::UNDERLINE),
            row_style: Style::empty(),
            selected_style: Style::empty().bg(Color::BLUE).fg(Color::WHITE),
            selected: None,
            show_separators: false,
            grow: 0.0,
            bg: Color::TRANSPARENT,
        }
    }

    /// Set explicit column widths (in terminal columns).
    ///
    /// If empty, columns are auto-sized equally across the widget's width.
    #[must_use]
    pub fn widths(mut self, widths: impl IntoIterator<Item = u16>) -> Self {
        self.widths = widths.into_iter().collect();
        self
    }

    /// Set the header row.
    ///
    /// Each element is converted to a [`compact_str::CompactString`].
    #[must_use]
    pub fn header(
        mut self,
        header: impl IntoIterator<Item = impl Into<compact_str::CompactString>>,
    ) -> Self {
        self.header = header.into_iter().map(Into::into).collect();
        self
    }

    /// Append a single data row.
    ///
    /// Each element is converted to a [`compact_str::CompactString`].
    #[must_use]
    pub fn row(
        mut self,
        row: impl IntoIterator<Item = impl Into<compact_str::CompactString>>,
    ) -> Self {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }

    /// Replace all data rows.
    ///
    /// Each inner element is converted to a [`compact_str::CompactString`].
    #[must_use]
    pub fn rows(
        mut self,
        rows: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<compact_str::CompactString>>>,
    ) -> Self {
        self.rows = rows
            .into_iter()
            .map(|r| r.into_iter().map(Into::into).collect())
            .collect();
        self
    }

    /// Set the header style.
    #[must_use]
    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Set the row style applied to every (non-selected) row.
    #[must_use]
    pub fn row_style(mut self, style: Style) -> Self {
        self.row_style = style;
        self
    }

    /// Set the style used to highlight the selected row.
    #[must_use]
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the selected row index (`None` for no selection).
    #[must_use]
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Enable or disable vertical column separators.
    #[must_use]
    pub fn show_separators(mut self, show: bool) -> Self {
        self.show_separators = show;
        self
    }

    /// Set flex grow.
    #[must_use]
    pub fn grow(mut self, g: f32) -> Self {
        self.grow = g;
        self
    }

    /// Set the background color.
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    /// Resolve the effective column widths for a widget of `width` columns.
    ///
    /// If [`Table::widths`] is non-empty it is used directly (clamped to the
    /// available width). Otherwise the width is divided as equally as possible
    /// among `num_cols` columns.
    fn column_widths(&self, width: u16, num_cols: usize) -> Vec<u16> {
        if num_cols == 0 {
            return Vec::new();
        }
        if !self.widths.is_empty() {
            return self
                .widths
                .iter()
                .copied()
                .take(num_cols)
                .map(|w| w.min(width))
                .collect();
        }
        let per = width / num_cols as u16;
        let mut remainder = width % num_cols as u16;
        let mut out = vec![per; num_cols];
        // Distribute leftover columns one at a time to the first columns.
        for w in &mut out {
            if remainder == 0 {
                break;
            }
            *w += 1;
            remainder -= 1;
        }
        out
    }

    /// Render a single cell of text into `rect`, truncating content that does
    /// not fit. The `style` is applied to every printed grapheme.
    fn paint_cell(ctx: &mut PaintCtx, rect: Rect, text: &str, style: Style) {
        let Rect { x, y, w, .. } = rect;
        if w == 0 {
            return;
        }
        let mut cx = x;
        for g in crate::unicode::graphemes(text) {
            if cx >= x + w {
                break;
            }
            let gw = crate::unicode::grapheme_width(g) as u16;
            if gw == 0 {
                continue;
            }
            // Truncate a wide grapheme that would overflow the cell.
            if cx + gw > x + w {
                break;
            }
            ctx.buffer.print(cx, y, g, style);
            cx += gw;
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Table {
    fn layout(&self) -> LayoutNode {
        let mut props = FlexProps::column();
        props.grow = self.grow;
        let mut node = props.to_node(Vec::new());
        // A table wants to fill available width and height.
        node.width = Length::Auto;
        node.height = Length::Auto;
        node
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let Rect { x, y, w, h, .. } = ctx.rect;
        if w == 0 || h == 0 {
            return;
        }

        // Fill the widget background if requested.
        if self.bg != Color::TRANSPARENT {
            ctx.buffer.fill_rect(ctx.rect, self.bg);
        }

        // Determine the number of columns from the header (if present) or the
        // widest row.
        let num_cols = if self.header.is_empty() {
            self.rows.iter().map(Vec::len).max().unwrap_or(0)
        } else {
            self.header.len()
        };
        if num_cols == 0 {
            return;
        }

        let col_widths = self.column_widths(w, num_cols);

        // Column x-offsets.
        let mut col_x = Vec::with_capacity(num_cols);
        let mut acc = x;
        for cw in &col_widths {
            col_x.push(acc);
            acc = acc.saturating_add(*cw);
        }

        let mut row_y = y;
        let bottom = y + h;

        // Header row.
        if !self.header.is_empty() && row_y < bottom {
            let style = self.header_style;
            if style.bg != Color::TRANSPARENT {
                ctx.buffer.fill_rect(Rect::new(x, row_y, w, 1), style.bg);
            }
            for (i, cell_text) in self.header.iter().take(num_cols).enumerate() {
                let cell_rect = Rect::new(col_x[i], row_y, col_widths[i], 1);
                Self::paint_cell(ctx, cell_rect, cell_text.as_str(), style);
            }
            if self.show_separators {
                for &sep_x in col_x.iter().take(num_cols).skip(1) {
                    if sep_x < x + w {
                        ctx.buffer.print(sep_x, row_y, "│", style);
                    }
                }
            }
            row_y = row_y.saturating_add(1);
        }

        // Data rows.
        for (i, row) in self.rows.iter().enumerate() {
            if row_y >= bottom {
                break;
            }
            let is_selected = self.selected == Some(i);
            let style = if is_selected {
                self.selected_style
            } else {
                self.row_style
            };
            if style.bg != Color::TRANSPARENT {
                ctx.buffer.fill_rect(Rect::new(x, row_y, w, 1), style.bg);
            }
            for (j, cell_text) in row.iter().take(num_cols).enumerate() {
                let cell_rect = Rect::new(col_x[j], row_y, col_widths[j], 1);
                Self::paint_cell(ctx, cell_rect, cell_text.as_str(), style);
            }
            if self.show_separators {
                for &sep_x in col_x.iter().take(num_cols).skip(1) {
                    if sep_x < x + w {
                        ctx.buffer.print(sep_x, row_y, "│", style);
                    }
                }
            }
            row_y = row_y.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::style::Attr;
    use std::time::Duration;

    /// Paint `table` into a fresh buffer of `w` x `h` and return the buffer.
    fn painted(table: &Table, w: u16, h: u16) -> Buffer {
        let mut buf = Buffer::empty(w, h);
        let rect = Rect::new(0, 0, w, h);
        let rects = [rect];
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect,
            rects: &rects,
            elapsed: Duration::ZERO,
        };
        table.paint(&mut ctx);
        buf
    }

    #[test]
    fn renders_header_and_rows() {
        let table = Table::new()
            .header(["name", "age"])
            .row(["alice", "30"])
            .row(["bob", "25"]);
        let buf = painted(&table, 12, 3);
        // Header on row 0.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "n");
        assert_eq!(buf.cell(6, 0).unwrap().grapheme, "a");
        // First data row on row 1.
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "a");
        assert_eq!(buf.cell(6, 1).unwrap().grapheme, "3");
        // Second data row on row 2.
        assert_eq!(buf.cell(0, 2).unwrap().grapheme, "b");
        assert_eq!(buf.cell(6, 2).unwrap().grapheme, "2");
    }

    #[test]
    fn header_is_bold_and_underlined() {
        let table = Table::new().header(["x"]).row(["1"]);
        let buf = painted(&table, 4, 2);
        let header_style = buf.cell(0, 0).unwrap().style;
        assert!(header_style.attr.contains(Attr::BOLD));
        assert!(header_style.attr.contains(Attr::UNDERLINE));
        // Data row is not bold.
        let row_style = buf.cell(0, 1).unwrap().style;
        assert!(!row_style.attr.contains(Attr::BOLD));
    }

    #[test]
    fn selected_row_uses_selected_style() {
        let table = Table::new()
            .row(["a"])
            .row(["b"])
            .selected(Some(1))
            .selected_style(Style::empty().bg(Color::RED).fg(Color::WHITE));
        let buf = painted(&table, 4, 2);
        // Row 0 is not selected.
        assert_ne!(buf.cell(0, 0).unwrap().style.bg, Color::RED);
        // Row 1 is selected.
        assert_eq!(buf.cell(0, 1).unwrap().style.bg, Color::RED);
        assert_eq!(buf.cell(0, 1).unwrap().style.fg, Color::WHITE);
    }

    #[test]
    fn column_separators_drawn_when_enabled() {
        let table = Table::new()
            .header(["a", "b"])
            .row(["1", "2"])
            .show_separators(true)
            .widths([3, 3]);
        let buf = painted(&table, 6, 2);
        // Separator at column 3 on both rows.
        assert_eq!(buf.cell(3, 0).unwrap().grapheme, "│");
        assert_eq!(buf.cell(3, 1).unwrap().grapheme, "│");
    }

    #[test]
    fn truncates_overflowing_cell_content() {
        let table = Table::new()
            .header(["col"])
            .row(["hello world"])
            .widths([4]);
        let buf = painted(&table, 4, 2);
        // Only "hell" fits in 4 columns.
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "h");
        assert_eq!(buf.cell(3, 1).unwrap().grapheme, "l");
        // Column 4 is out of bounds for a 4-wide buffer.
        assert!(buf.cell(4, 1).is_none());
    }

    #[test]
    fn auto_sizes_columns_equally() {
        let table = Table::new().header(["a", "b"]).row(["1", "2"]);
        let buf = painted(&table, 10, 2);
        // 10 cols / 2 cols = 5 each. "b" header starts at col 5.
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "a");
        assert_eq!(buf.cell(5, 0).unwrap().grapheme, "b");
        assert_eq!(buf.cell(0, 1).unwrap().grapheme, "1");
        assert_eq!(buf.cell(5, 1).unwrap().grapheme, "2");
    }
}
