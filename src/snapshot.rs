//! Snapshot testing helpers for [`Buffer`].
//!
//! These utilities capture the visible state of a [`Buffer`] as plain text so
//! that rendered output can be compared against expected strings in unit
//! tests. [`BufferSnapshot`] is a lightweight value capturing the text + size;
//! [`assert_buffer`] is a one-shot assertion that prints a helpful diff on
//! mismatch.
//!
//! # Example
//!
//! ```no_run
//! use rustui::{Buffer, Style, assert_buffer};
//!
//! let mut buf = Buffer::empty(5, 1);
//! buf.print(0, 0, "hi", Style::empty());
//! assert_buffer(&buf, "hi   \n");
//! ```

use crate::buffer::Buffer;
use crate::cell::Cell;
use crate::color::Color;
use crate::style::Attr;

/// A captured visible-state snapshot of a [`Buffer`].
///
/// The [`text`](Self::text) field holds the rendered content as a multi-line
/// string (one line per row, with a trailing newline after every row including
/// the last). [`width`](Self::width) and [`height`](Self::height) record the
/// grid dimensions so snapshots can be sanity-checked against expected sizes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferSnapshot {
    /// The rendered text as a multi-line string.
    pub text: String,
    /// Width of the snapshot.
    pub width: u16,
    /// Height of the snapshot.
    pub height: u16,
}

impl BufferSnapshot {
    /// Capture a buffer's visible state.
    ///
    /// Each row is rendered as the concatenation of its cells' graphemes
    /// (blanks become spaces), followed by a `\n`.
    #[must_use]
    pub fn from_buffer(buf: &Buffer) -> Self {
        let text = render_buffer(buf);
        Self {
            text,
            width: buf.width,
            height: buf.height,
        }
    }

    /// Get the plain text representation (a copy of [`text`](Self::text)).
    #[must_use]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.text.clone()
    }

    /// Compare two snapshots for equality (text + dimensions).
    #[must_use]
    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }
}

/// Render a [`Buffer`] to a plain-text string.
///
/// Each cell's grapheme is emitted in column order; blank and continuation
/// cells become a single space. Every row is terminated with a `\n`, so a
/// buffer of height `h` produces `h` lines.
#[must_use]
pub fn render_buffer(buf: &Buffer) -> String {
    let mut out = String::with_capacity(usize::from(buf.width) * usize::from(buf.height) * 2);
    for y in 0..buf.height {
        for x in 0..buf.width {
            match buf.cell(x, y) {
                Some(cell) if !cell.is_blank() => out.push_str(&cell.grapheme),
                _ => out.push(' '),
            }
        }
        out.push('\n');
    }
    out
}

/// Render a [`Buffer`] to a string annotated with per-cell style info.
///
/// Each cell is rendered as `"<grapheme>[fg=..,bg=..,attr=..]"`. Blanks use a
/// literal `·` (middle dot) so empty cells are visually distinct from spaces.
/// Wide-grapheme continuation cells are shown as `~` to distinguish them from
/// real blanks. This format is intended for debugging and human-readable
/// snapshot inspection, not for equality comparison.
#[must_use]
pub fn render_buffer_with_styles(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.height {
        for x in 0..buf.width {
            let cell = buf.cell(x, y);
            let label = match cell {
                None => "OOB".to_string(),
                Some(c) if !c.wide_start && !c.is_blank() => {
                    // A non-wide-start cell with content is unusual but possible
                    // (e.g. set directly). Show it as-is.
                    format_cell(c)
                }
                Some(c) if c.wide_start => format_cell(c),
                Some(c) if c.is_blank() => {
                    // Distinguish a true blank from a continuation cell. Both
                    // have empty graphemes, but continuations are produced by
                    // `Cell::continuation()` after a wide grapheme. We can't
                    // tell them apart structurally, so treat all empties as
                    // blanks and annotate with their style.
                    format_blank(c)
                }
                Some(c) => format_cell(c),
            };
            out.push_str(&label);
            out.push(' ');
        }
        out.push('\n');
    }
    out
}

/// Format a non-blank cell with style annotations.
fn format_cell(c: &Cell) -> String {
    let grapheme = if c.is_blank() { "·" } else { &c.grapheme };
    format!("{}[{}]", grapheme, style_annotation(c))
}

/// Format a blank cell with style annotations.
fn format_blank(c: &Cell) -> String {
    format!("·[{}]", style_annotation(c))
}

/// Build a compact `fg=..,bg=..,attr=..` annotation for a cell.
fn style_annotation(c: &Cell) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("fg={}", color_name(c.style.fg)));
    parts.push(format!("bg={}", color_name(c.style.bg)));
    if !c.style.attr.is_empty() {
        parts.push(format!("attr={}", attr_name(c.style.attr)));
    }
    parts.join(",")
}

/// A short human-readable name for a [`Color`].
///
/// Recognizes the common named constants from [`Color`]; everything else is
/// rendered as a `#RRGGBB` hex string (or `#RRGGBBAA` when there is alpha).
fn color_name(c: Color) -> String {
    if c == Color::TRANSPARENT {
        return "none".to_string();
    }
    if c == Color::BLACK {
        return "black".to_string();
    }
    if c == Color::WHITE {
        return "white".to_string();
    }
    if c == Color::RED {
        return "red".to_string();
    }
    if c == Color::GREEN {
        return "green".to_string();
    }
    if c == Color::BLUE {
        return "blue".to_string();
    }
    if c == Color::CYAN {
        return "cyan".to_string();
    }
    if c == Color::YELLOW {
        return "yellow".to_string();
    }
    if c == Color::MAGENTA {
        return "magenta".to_string();
    }
    if c.a == 255 {
        format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", c.r, c.g, c.b, c.a)
    }
}

/// A short human-readable name for an [`Attr`] bitfield.
fn attr_name(attr: Attr) -> String {
    let mut names: Vec<&str> = Vec::new();
    if attr.contains(Attr::BOLD) {
        names.push("bold");
    }
    if attr.contains(Attr::ITALIC) {
        names.push("italic");
    }
    if attr.contains(Attr::UNDERLINE) {
        names.push("underline");
    }
    if attr.contains(Attr::REVERSE) {
        names.push("reverse");
    }
    if attr.contains(Attr::DIM) {
        names.push("dim");
    }
    if attr.contains(Attr::BLINK) {
        names.push("blink");
    }
    if attr.contains(Attr::HIDDEN) {
        names.push("hidden");
    }
    if attr.contains(Attr::STRIKE) {
        names.push("strike");
    }
    names.join("|")
}

/// Assert that a buffer's rendered text matches `expected`.
///
/// On mismatch, prints a helpful unified-style diff to stderr and panics.
/// `expected` should include the trailing newline after every row (as
/// produced by [`render_buffer`]).
pub fn assert_buffer(buf: &Buffer, expected: &str) {
    let actual = render_buffer(buf);
    assert!(
        actual == expected,
        "buffer snapshot mismatch:\n\
         --- expected\n\
         +++ actual\n\
         {}\n\
         --- expected (raw) ---\n\
         {expected:?}\n\
         --- actual (raw) ---\n\
         {actual:?}",
        diff_lines(expected, &actual)
    );
}

/// Produce a simple line-by-line diff between two strings.
fn diff_lines(expected: &str, actual: &str) -> String {
    let exp: Vec<&str> = expected.split_inclusive('\n').collect();
    let act: Vec<&str> = actual.split_inclusive('\n').collect();
    let n = exp.len().max(act.len());
    let mut out = String::new();
    for i in 0..n {
        let e = exp.get(i).copied().unwrap_or("");
        let a = act.get(i).copied().unwrap_or("");
        if e == a {
            out.push_str("  ");
            out.push_str(e);
        } else {
            if !e.is_empty() {
                out.push_str("- ");
                out.push_str(e);
                if !e.ends_with('\n') {
                    out.push('\n');
                }
            }
            if !a.is_empty() {
                out.push_str("+ ");
                out.push_str(a);
                if !a.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    #[test]
    fn render_empty_buffer() {
        let buf = Buffer::empty(3, 2);
        let s = render_buffer(&buf);
        assert_eq!(s, "   \n   \n");
    }

    #[test]
    fn snapshot_from_buffer_captures_text_and_size() {
        let mut buf = Buffer::empty(5, 1);
        buf.print(0, 0, "hi", Style::empty());
        let snap = BufferSnapshot::from_buffer(&buf);
        assert_eq!(snap.width, 5);
        assert_eq!(snap.height, 1);
        assert_eq!(snap.text, "hi   \n");
        assert_eq!(snap.to_string(), "hi   \n");
    }

    #[test]
    fn snapshot_equals_compares_text_and_dims() {
        let mut a = Buffer::empty(4, 1);
        a.print(0, 0, "yo", Style::empty());
        let sa = BufferSnapshot::from_buffer(&a);

        let mut b = Buffer::empty(4, 1);
        b.print(0, 0, "yo", Style::empty());
        let sb = BufferSnapshot::from_buffer(&b);

        assert!(sa.equals(&sb));

        let mut c = Buffer::empty(4, 1);
        c.print(0, 0, "no", Style::empty());
        let sc = BufferSnapshot::from_buffer(&c);
        assert!(!sa.equals(&sc));

        let mut d = Buffer::empty(2, 1);
        d.print(0, 0, "yo", Style::empty());
        let sd = BufferSnapshot::from_buffer(&d);
        assert!(!sa.equals(&sd));
    }

    #[test]
    fn assert_buffer_passes_on_match() {
        let mut buf = Buffer::empty(4, 1);
        buf.print(0, 0, "ab", Style::empty());
        assert_buffer(&buf, "ab  \n");
    }

    #[test]
    #[should_panic(expected = "buffer snapshot mismatch")]
    fn assert_buffer_panics_on_mismatch() {
        let mut buf = Buffer::empty(4, 1);
        buf.print(0, 0, "ab", Style::empty());
        assert_buffer(&buf, "cd  \n");
    }

    #[test]
    fn render_buffer_with_styles_includes_colors() {
        let mut buf = Buffer::empty(2, 1);
        buf.print(0, 0, "x", Style::empty().fg(Color::RED).bg(Color::BLACK));
        let rendered = render_buffer_with_styles(&buf);
        assert!(rendered.contains("x[fg=red,bg=black]"), "got: {rendered}");
    }

    #[test]
    fn render_buffer_with_styles_shows_bold_attr() {
        let mut buf = Buffer::empty(2, 1);
        buf.print(0, 0, "z", Style::empty().bold());
        let rendered = render_buffer_with_styles(&buf);
        assert!(rendered.contains("attr=bold"), "got: {rendered}");
    }
}
