//! Text styling: foreground/background color plus attributes.

use crate::color::Color;

/// Text attributes (bold, italic, underline, ...).
///
/// Stored as a `u16` bitfield. Use the associated constants to construct and
/// combine: `Attr::BOLD | Attr::UNDERLINE`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Attr(pub u16);

impl Attr {
    /// No attributes.
    pub const NONE: Self = Self(0);
    /// Bold / increased intensity.
    pub const BOLD: Self = Self(1 << 0);
    /// Italic.
    pub const ITALIC: Self = Self(1 << 1);
    /// Underlined.
    pub const UNDERLINE: Self = Self(1 << 2);
    /// Reverse video (swap fg/bg).
    pub const REVERSE: Self = Self(1 << 3);
    /// Dim / decreased intensity.
    pub const DIM: Self = Self(1 << 4);
    /// Blinking text (rarely supported, use sparingly).
    pub const BLINK: Self = Self(1 << 5);
    /// Hidden (foreground == background).
    pub const HIDDEN: Self = Self(1 << 6);
    /// Strikethrough.
    pub const STRIKE: Self = Self(1 << 7);

    /// Whether `self` contains all bits in `other`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether any attributes are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for Attr {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Attr {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Attr {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for Attr {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for Attr {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// A complete text style: foreground color, background color, and attributes.
///
/// `Color::TRANSPARENT` is used as a sentinel meaning "inherit from parent /
/// default". This lets widgets compose styles without forcing every layer to
/// restate every property.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Style {
    /// Foreground (text) color. `Color::TRANSPARENT` means inherit.
    pub fg: Color,
    /// Background color. `Color::TRANSPARENT` means inherit.
    pub bg: Color,
    /// Text attributes.
    pub attr: Attr,
}

impl Style {
    /// An empty style — inherits everything.
    pub const fn empty() -> Self {
        Self {
            fg: Color::TRANSPARENT,
            bg: Color::TRANSPARENT,
            attr: Attr::NONE,
        }
    }

    /// Set the foreground color.
    #[must_use]
    pub const fn fg(mut self, c: Color) -> Self {
        self.fg = c;
        self
    }

    /// Set the background color.
    #[must_use]
    pub const fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    /// Set the text attributes.
    #[must_use]
    pub const fn attr(mut self, a: Attr) -> Self {
        self.attr = Attr(self.attr.0 | a.0);
        self
    }

    /// Add bold.
    #[must_use]
    pub const fn bold(self) -> Self {
        self.attr(Attr::BOLD)
    }

    /// Add italic.
    #[must_use]
    pub const fn italic(self) -> Self {
        self.attr(Attr::ITALIC)
    }

    /// Add underline.
    #[must_use]
    pub const fn underline(self) -> Self {
        self.attr(Attr::UNDERLINE)
    }

    /// Add dim.
    #[must_use]
    pub const fn dim(self) -> Self {
        self.attr(Attr::DIM)
    }

    /// Compose `self` over `parent`: any property of `self` that is the
    /// sentinel (transparent / none) inherits from `parent`.
    #[must_use]
    pub fn over(self, parent: Style) -> Style {
        Style {
            fg: if self.fg == Color::TRANSPARENT {
                parent.fg
            } else {
                self.fg
            },
            bg: if self.bg == Color::TRANSPARENT {
                parent.bg
            } else {
                self.bg
            },
            attr: if self.attr == Attr::NONE {
                parent.attr
            } else {
                self.attr | parent.attr
            },
        }
    }
}

impl From<Color> for Style {
    fn from(c: Color) -> Self {
        Style::empty().fg(c)
    }
}
