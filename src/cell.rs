//! A single terminal cell: a grapheme cluster + a style.

use crate::style::Style;

/// A single character position in a [`crate::buffer::Buffer`].
///
/// A cell holds a grapheme cluster (which may be more than one `char`, e.g.
/// `"e\u{0301}"` or an emoji with a VS16 selector) and the style to render it
/// with. The display width is cached so the buffer can advance cursors without
/// re-measuring.
#[derive(Clone, Debug, Default)]
pub struct Cell {
    /// The grapheme cluster (or empty for a blank cell).
    pub grapheme: compact_str::CompactString,
    /// The style to render this cell with.
    pub style: Style,
    /// Cached display width in terminal columns (0, 1, or 2).
    pub width: u8,
    /// True if this cell is the first column of a multi-column grapheme.
    /// Subsequent columns are filled with [`Cell::continuation`] cells so the
    /// buffer stays rectangular.
    pub wide_start: bool,
}

impl Cell {
    /// A blank cell (no grapheme, default style).
    pub const fn blank() -> Self {
        Self {
            grapheme: compact_str::CompactString::const_new(""),
            style: Style::empty(),
            width: 0,
            wide_start: false,
        }
    }

    /// A continuation cell for the trailing column(s) of a wide grapheme.
    pub const fn continuation() -> Self {
        Self {
            grapheme: compact_str::CompactString::const_new(""),
            style: Style::empty(),
            width: 0,
            wide_start: false,
        }
    }

    /// Whether this cell is blank (no grapheme content).
    pub fn is_blank(&self) -> bool {
        self.grapheme.is_empty()
    }

    /// Construct a cell from a grapheme cluster, computing its display width.
    pub fn from_grapheme(grapheme: &str, style: Style) -> Self {
        let width = crate::unicode::grapheme_width(grapheme);
        Self {
            grapheme: compact_str::CompactString::new(grapheme),
            style,
            width: width as u8,
            wide_start: width > 0,
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.grapheme == other.grapheme && self.style == other.style && self.width == other.width
    }
}

impl Eq for Cell {}
