//! Cell-grid frame buffer with scissor clipping and alpha compositing.

use crate::cell::Cell;
use crate::color::Color;
use crate::style::Style;

/// A rectangular region of the screen, in (x, y, width, height) form.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    /// Column index (0-based, from the left).
    pub x: u16,
    /// Row index (0-based, from the top).
    pub y: u16,
    /// Width in columns.
    pub w: u16,
    /// Height in rows.
    pub h: u16,
}

impl Rect {
    /// Construct a rect.
    pub const fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    /// Empty (zero-area) rect.
    pub const fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Whether the rect has any area.
    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// Right edge column (exclusive).
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.w)
    }

    /// Bottom edge row (exclusive).
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.h)
    }

    /// Whether `(x, y)` lies inside this rect.
    pub const fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Intersect with `other`; the result is the largest rect contained in both.
    #[must_use]
    pub fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        let w = right.saturating_sub(x);
        let h = bottom.saturating_sub(y);
        Self::new(x, y, w, h)
    }
}

/// A 2D grid of [`Cell`]s representing one rendered frame.
///
/// The buffer is row-major: `cell(x, y)` is at index `y * width + x`. Wide
/// graphemes occupy their leading cell plus trailing [`Cell::continuation`]
/// cells so the grid stays rectangular.
pub struct Buffer {
    /// Buffer width in columns.
    pub width: u16,
    /// Buffer height in rows.
    pub height: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    /// Construct a blank buffer of the given size.
    pub fn empty(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![Cell::blank(); len],
        }
    }

    /// Resize the buffer, discarding contents if dimensions change.
    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            *self = Self::empty(width, height);
        }
    }

    /// Borrow the cell at `(x, y)`, or `None` if out of bounds.
    pub fn cell(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.width && y < self.height {
            self.cells
                .get(usize::from(y) * usize::from(self.width) + usize::from(x))
        } else {
            None
        }
    }

    /// Mutably borrow the cell at `(x, y)`, or `None` if out of bounds.
    pub fn cell_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if x < self.width && y < self.height {
            let i = usize::from(y) * usize::from(self.width) + usize::from(x);
            self.cells.get_mut(i)
        } else {
            None
        }
    }

    /// Iterator over all cells in row `y`.
    pub fn row(&self, y: u16) -> impl Iterator<Item = &Cell> {
        let start = usize::from(y) * usize::from(self.width);
        self.cells[start..start + usize::from(self.width)].iter()
    }

    /// Fill the entire buffer with blank cells.
    pub fn clear(&mut self) {
        self.cells.fill(Cell::blank());
    }

    /// Fill the entire buffer with a background color (keeping text).
    pub fn fill_bg(&mut self, bg: Color) {
        for c in &mut self.cells {
            if c.is_blank() {
                c.style.bg = bg;
            } else if c.style.bg == Color::TRANSPARENT {
                c.style.bg = bg;
            }
        }
    }

    /// Composite `src` onto `self` within `clip`. Out-of-bounds and
    /// transparent cells in `src` are skipped.
    pub fn composite(&mut self, src: &Buffer, offset_x: u16, offset_y: u16, clip: Rect) {
        let clip = clip.intersect(Rect::new(0, 0, self.width, self.height));
        if clip.is_empty() {
            return;
        }
        for sy in 0..src.height {
            let dy = offset_y.saturating_add(sy);
            if dy < clip.y || dy >= clip.bottom() {
                continue;
            }
            for sx in 0..src.width {
                let dx = offset_x.saturating_add(sx);
                if dx < clip.x || dx >= clip.right() {
                    continue;
                }
                let Some(src_cell) = src.cell(sx, sy) else {
                    continue;
                };
                if src_cell.is_blank() && src_cell.style.bg == Color::TRANSPARENT {
                    continue;
                }
                if let Some(dst) = self.cell_mut(dx, dy) {
                    if !src_cell.is_blank() {
                        dst.grapheme.clone_from(&src_cell.grapheme);
                        dst.width = src_cell.width;
                        dst.wide_start = src_cell.wide_start;
                    }
                    if src_cell.style.bg != Color::TRANSPARENT {
                        dst.style.bg = src_cell.style.bg.over(dst.style.bg);
                    }
                    if src_cell.style.fg != Color::TRANSPARENT {
                        dst.style.fg = src_cell.style.fg.over(dst.style.fg);
                    }
                    if !src_cell.style.attr.is_empty() {
                        dst.style.attr = src_cell.style.attr;
                    }
                }
            }
        }
    }

    /// Print a string at `(x, y)` with the given style, advancing the cursor
    /// by each grapheme's display width. Stops at the right edge.
    pub fn print(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut cx = x;
        for g in crate::unicode::graphemes(text) {
            if cx >= self.width {
                break;
            }
            let w = crate::unicode::grapheme_width(g) as u16;
            if w == 0 {
                continue;
            }
            if let Some(cell) = self.cell_mut(cx, y) {
                cell.grapheme = compact_str::CompactString::new(g);
                cell.style = style;
                cell.width = w as u8;
                cell.wide_start = true;
            }
            // Fill continuation columns for wide graphemes.
            for i in 1..w {
                if let Some(cell) = self.cell_mut(cx + i, y) {
                    *cell = Cell::continuation();
                }
            }
            cx += w;
        }
    }

    /// Fill a rect with a solid background color.
    pub fn fill_rect(&mut self, rect: Rect, bg: Color) {
        let rect = rect.intersect(Rect::new(0, 0, self.width, self.height));
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                if let Some(cell) = self.cell_mut(x, y) {
                    if cell.is_blank() {
                        cell.style.bg = bg;
                    } else if cell.style.bg == Color::TRANSPARENT {
                        cell.style.bg = bg;
                    }
                }
            }
        }
    }

    /// Draw a single-line box border inside `rect`.
    pub fn box_border(&mut self, rect: Rect, style: Style) {
        let Rect { x, y, w, h } = rect.intersect(Rect::new(0, 0, self.width, self.height));
        if w == 0 || h == 0 {
            return;
        }
        let right = x + w - 1;
        let bottom = y + h - 1;
        // Corners
        self.print(x, y, "┌", style);
        self.print(right, y, "┐", style);
        self.print(x, bottom, "└", style);
        self.print(right, bottom, "┘", style);
        // Top and bottom edges
        if w > 2 {
            let top: String = "─".repeat((w - 2) as usize);
            self.print(x + 1, y, &top, style);
            self.print(x + 1, bottom, &top, style);
        }
        // Left and right edges
        if h > 2 {
            for ry in (y + 1)..bottom {
                self.print(x, ry, "│", style);
                self.print(right, ry, "│", style);
            }
        }
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            cells: self.cells.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_intersect() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.intersect(b), Rect::new(5, 5, 5, 5));
    }

    #[test]
    fn print_writes_graphemes() {
        let mut buf = Buffer::empty(10, 1);
        buf.print(0, 0, "hi", Style::empty());
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "h");
        assert_eq!(buf.cell(1, 0).unwrap().grapheme, "i");
    }

    #[test]
    fn print_handles_wide_chars() {
        let mut buf = Buffer::empty(5, 1);
        buf.print(0, 0, "😀x", Style::empty());
        assert_eq!(buf.cell(0, 0).unwrap().grapheme, "😀");
        assert_eq!(buf.cell(0, 0).unwrap().width, 2);
        assert_eq!(buf.cell(1, 0).unwrap().grapheme, ""); // continuation
        assert_eq!(buf.cell(2, 0).unwrap().grapheme, "x");
    }
}
