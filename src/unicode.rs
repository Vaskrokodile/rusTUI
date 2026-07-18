//! Unicode helpers: grapheme iteration and display-width calculation.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Iterate over the grapheme clusters of `s`.
pub fn graphemes(s: &str) -> impl Iterator<Item = &str> {
    s.graphemes(true)
}

/// Display width of a grapheme cluster in terminal columns (0, 1, or 2).
///
/// We take the width of the first non-zero-width code point; combining marks
/// and zero-width joiners contribute 0. This matches what most terminals do.
pub fn grapheme_width(g: &str) -> usize {
    for c in g.chars() {
        let w = c.width().unwrap_or(0);
        if w > 0 {
            return w;
        }
    }
    // Fallback: use the width of the whole string (handles some emoji ZWJ
    // sequences that `UnicodeWidthChar` underestimates).
    g.width()
}

/// Display width of an entire string.
pub fn str_width(s: &str) -> usize {
    s.graphemes(true).map(|g| grapheme_width(g)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(str_width("hello"), 5);
    }

    #[test]
    fn emoji_width() {
        assert_eq!(grapheme_width("😀"), 2);
    }

    #[test]
    fn combining_mark_zero_width() {
        assert_eq!(grapheme_width("e\u{0301}"), 1);
    }

    #[test]
    fn zero_width_chars() {
        assert_eq!(str_width("\u{200B}"), 0);
    }
}
