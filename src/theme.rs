//! Semantic color themes with named presets.
//!
//! A [`Theme`] bundles the colors used across the UI into a single value so
//! that widgets can be recolored without touching their internals. Several
//! named presets (`DARK`, `LIGHT`, `GRUVBOX`, ...) are provided as associated
//! constants; [`Theme::default`] returns [`Theme::DARK`].

use crate::color::Color;

/// A syntax token category, used by [`Theme::syntax_color`].
///
/// Each variant represents a class of source-code token recognized by a syntax
/// highlighter; the theme maps it to a concrete [`Color`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyntaxTokenType {
    /// Language keywords (`fn`, `let`, `if`, ...).
    Keyword,
    /// Function identifiers.
    Function,
    /// String literals.
    String,
    /// Numeric literals.
    Number,
    /// Comments.
    Comment,
    /// Type names / annotations.
    Type,
    /// Plain variable identifiers.
    Variable,
    /// Operators (`+`, `=`, `->`, ...).
    Operator,
    /// Punctuation (`(`, `,`, `;`, ...).
    Punctuation,
    /// Constant values (`UPPER_SNAKE` constants, booleans, ...).
    Constant,
    /// Macro invocations / definitions.
    Macro,
}

/// A bundle of semantic colors used to render the UI.
///
/// All fields are plain [`Color`] values so a theme can be inspected and
/// mutated freely. Use one of the named presets (`DARK`, `LIGHT`, ...) as a
/// starting point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Theme {
    /// Main background color.
    pub bg: Color,
    /// Main foreground (text) color.
    pub fg: Color,
    /// Dimmed / secondary text color.
    pub muted: Color,
    /// Accent / highlight color.
    pub accent: Color,
    /// Border color.
    pub border: Color,
    /// Title text color.
    pub title: Color,
    /// Success / positive color.
    pub success: Color,
    /// Warning color.
    pub warning: Color,
    /// Error color.
    pub error: Color,
    /// Info color.
    pub info: Color,
    /// Diff: added-line foreground.
    pub diff_add: Color,
    /// Diff: removed-line foreground.
    pub diff_remove: Color,
    /// Diff: context-line foreground.
    pub diff_context: Color,
    /// Diff: added-line background.
    pub diff_add_bg: Color,
    /// Diff: removed-line background.
    pub diff_remove_bg: Color,
    /// User message color.
    pub user_msg: Color,
    /// Assistant message color.
    pub assistant_msg: Color,
    /// System message color.
    pub system_msg: Color,
    /// Tool call color.
    pub tool_call: Color,
    /// Selection background color.
    pub selection_bg: Color,
    /// Cursor color.
    pub cursor: Color,
}

impl Theme {
    /// Dark theme: dark background, light foreground, cyan accent.
    pub const DARK: Self = Self {
        bg: Color::rgb(22, 24, 33),
        fg: Color::rgb(220, 224, 232),
        muted: Color::rgb(120, 128, 144),
        accent: Color::rgb(80, 220, 220),
        border: Color::rgb(60, 66, 82),
        title: Color::rgb(180, 190, 210),
        success: Color::rgb(80, 200, 120),
        warning: Color::rgb(220, 200, 80),
        error: Color::rgb(220, 80, 90),
        info: Color::rgb(80, 160, 240),
        diff_add: Color::rgb(80, 200, 120),
        diff_remove: Color::rgb(220, 80, 90),
        diff_context: Color::rgb(160, 168, 184),
        diff_add_bg: Color::rgb(30, 50, 38),
        diff_remove_bg: Color::rgb(50, 30, 36),
        user_msg: Color::rgb(120, 200, 240),
        assistant_msg: Color::rgb(180, 200, 240),
        system_msg: Color::rgb(160, 168, 184),
        tool_call: Color::rgb(220, 180, 120),
        selection_bg: Color::rgb(48, 54, 74),
        cursor: Color::rgb(220, 224, 232),
    };

    /// Light theme: light background, dark foreground, blue accent.
    pub const LIGHT: Self = Self {
        bg: Color::rgb(250, 250, 250),
        fg: Color::rgb(40, 44, 52),
        muted: Color::rgb(140, 146, 160),
        accent: Color::rgb(40, 90, 200),
        border: Color::rgb(210, 214, 222),
        title: Color::rgb(60, 66, 82),
        success: Color::rgb(40, 160, 80),
        warning: Color::rgb(180, 140, 30),
        error: Color::rgb(200, 50, 60),
        info: Color::rgb(40, 120, 220),
        diff_add: Color::rgb(40, 160, 80),
        diff_remove: Color::rgb(200, 50, 60),
        diff_context: Color::rgb(100, 108, 124),
        diff_add_bg: Color::rgb(224, 240, 226),
        diff_remove_bg: Color::rgb(244, 224, 226),
        user_msg: Color::rgb(40, 120, 220),
        assistant_msg: Color::rgb(80, 60, 160),
        system_msg: Color::rgb(120, 126, 140),
        tool_call: Color::rgb(160, 110, 40),
        selection_bg: Color::rgb(206, 212, 224),
        cursor: Color::rgb(40, 44, 52),
    };

    /// Gruvbox-inspired theme.
    pub const GRUVBOX: Self = Self {
        bg: Color::rgb(40, 40, 40),
        fg: Color::rgb(235, 219, 178),
        muted: Color::rgb(146, 131, 116),
        accent: Color::rgb(250, 189, 47),
        border: Color::rgb(80, 73, 69),
        title: Color::rgb(214, 93, 14),
        success: Color::rgb(152, 151, 26),
        warning: Color::rgb(250, 189, 47),
        error: Color::rgb(204, 36, 29),
        info: Color::rgb(69, 133, 136),
        diff_add: Color::rgb(152, 151, 26),
        diff_remove: Color::rgb(204, 36, 29),
        diff_context: Color::rgb(168, 153, 132),
        diff_add_bg: Color::rgb(50, 50, 30),
        diff_remove_bg: Color::rgb(60, 30, 28),
        user_msg: Color::rgb(69, 133, 136),
        assistant_msg: Color::rgb(214, 93, 14),
        system_msg: Color::rgb(146, 131, 116),
        tool_call: Color::rgb(250, 189, 47),
        selection_bg: Color::rgb(80, 73, 69),
        cursor: Color::rgb(250, 189, 47),
    };

    /// Nord-inspired theme.
    pub const NORD: Self = Self {
        bg: Color::rgb(46, 52, 64),
        fg: Color::rgb(236, 239, 244),
        muted: Color::rgb(76, 86, 106),
        accent: Color::rgb(136, 192, 208),
        border: Color::rgb(59, 66, 82),
        title: Color::rgb(143, 188, 187),
        success: Color::rgb(163, 190, 140),
        warning: Color::rgb(235, 203, 139),
        error: Color::rgb(191, 97, 106),
        info: Color::rgb(129, 161, 193),
        diff_add: Color::rgb(163, 190, 140),
        diff_remove: Color::rgb(191, 97, 106),
        diff_context: Color::rgb(216, 222, 233),
        diff_add_bg: Color::rgb(46, 60, 46),
        diff_remove_bg: Color::rgb(64, 46, 52),
        user_msg: Color::rgb(129, 161, 193),
        assistant_msg: Color::rgb(180, 142, 173),
        system_msg: Color::rgb(76, 86, 106),
        tool_call: Color::rgb(136, 192, 208),
        selection_bg: Color::rgb(76, 86, 106),
        cursor: Color::rgb(236, 239, 244),
    };

    /// Dracula-inspired theme.
    pub const DRACULA: Self = Self {
        bg: Color::rgb(40, 42, 54),
        fg: Color::rgb(248, 248, 242),
        muted: Color::rgb(98, 114, 164),
        accent: Color::rgb(189, 147, 249),
        border: Color::rgb(68, 71, 90),
        title: Color::rgb(255, 184, 108),
        success: Color::rgb(80, 250, 123),
        warning: Color::rgb(241, 250, 140),
        error: Color::rgb(255, 85, 85),
        info: Color::rgb(139, 233, 253),
        diff_add: Color::rgb(80, 250, 123),
        diff_remove: Color::rgb(255, 85, 85),
        diff_context: Color::rgb(98, 114, 164),
        diff_add_bg: Color::rgb(40, 60, 48),
        diff_remove_bg: Color::rgb(64, 40, 48),
        user_msg: Color::rgb(139, 233, 253),
        assistant_msg: Color::rgb(189, 147, 249),
        system_msg: Color::rgb(98, 114, 164),
        tool_call: Color::rgb(255, 184, 108),
        selection_bg: Color::rgb(68, 71, 90),
        cursor: Color::rgb(248, 248, 242),
    };

    /// GitHub-inspired theme (GitHub dark default palette).
    pub const GITHUB: Self = Self {
        bg: Color::rgb(13, 17, 23),
        fg: Color::rgb(201, 209, 217),
        muted: Color::rgb(110, 118, 129),
        accent: Color::rgb(88, 166, 255),
        border: Color::rgb(48, 54, 61),
        title: Color::rgb(201, 209, 217),
        success: Color::rgb(63, 185, 80),
        warning: Color::rgb(187, 128, 9),
        error: Color::rgb(248, 81, 73),
        info: Color::rgb(88, 166, 255),
        diff_add: Color::rgb(63, 185, 80),
        diff_remove: Color::rgb(248, 81, 73),
        diff_context: Color::rgb(139, 148, 158),
        diff_add_bg: Color::rgb(18, 44, 29),
        diff_remove_bg: Color::rgb(56, 24, 28),
        user_msg: Color::rgb(88, 166, 255),
        assistant_msg: Color::rgb(163, 113, 247),
        system_msg: Color::rgb(110, 118, 129),
        tool_call: Color::rgb(210, 153, 34),
        selection_bg: Color::rgb(56, 62, 73),
        cursor: Color::rgb(201, 209, 217),
    };

    /// Monokai-inspired theme.
    pub const MONOKAI: Self = Self {
        bg: Color::rgb(39, 40, 34),
        fg: Color::rgb(248, 248, 242),
        muted: Color::rgb(117, 113, 94),
        accent: Color::rgb(166, 226, 46),
        border: Color::rgb(73, 72, 62),
        title: Color::rgb(253, 151, 31),
        success: Color::rgb(166, 226, 46),
        warning: Color::rgb(253, 151, 31),
        error: Color::rgb(249, 38, 114),
        info: Color::rgb(102, 217, 239),
        diff_add: Color::rgb(166, 226, 46),
        diff_remove: Color::rgb(249, 38, 114),
        diff_context: Color::rgb(117, 113, 94),
        diff_add_bg: Color::rgb(44, 56, 30),
        diff_remove_bg: Color::rgb(60, 30, 44),
        user_msg: Color::rgb(102, 217, 239),
        assistant_msg: Color::rgb(174, 129, 255),
        system_msg: Color::rgb(117, 113, 94),
        tool_call: Color::rgb(253, 151, 31),
        selection_bg: Color::rgb(73, 72, 62),
        cursor: Color::rgb(248, 248, 242),
    };

    /// Catppuccin Macchiato-inspired theme.
    pub const CATPPUCCIN: Self = Self {
        bg: Color::rgb(36, 39, 58),
        fg: Color::rgb(202, 211, 245),
        muted: Color::rgb(110, 115, 141),
        accent: Color::rgb(198, 160, 246),
        border: Color::rgb(54, 58, 79),
        title: Color::rgb(245, 169, 127),
        success: Color::rgb(166, 227, 161),
        warning: Color::rgb(238, 212, 159),
        error: Color::rgb(237, 135, 150),
        info: Color::rgb(138, 173, 244),
        diff_add: Color::rgb(166, 227, 161),
        diff_remove: Color::rgb(237, 135, 150),
        diff_context: Color::rgb(184, 192, 224),
        diff_add_bg: Color::rgb(40, 58, 48),
        diff_remove_bg: Color::rgb(60, 40, 52),
        user_msg: Color::rgb(138, 173, 244),
        assistant_msg: Color::rgb(198, 160, 246),
        system_msg: Color::rgb(110, 115, 141),
        tool_call: Color::rgb(245, 169, 127),
        selection_bg: Color::rgb(54, 58, 79),
        cursor: Color::rgb(202, 211, 245),
    };

    /// Returns the color for a syntax token type.
    ///
    /// This maps a [`SyntaxTokenType`] to the appropriate color for the theme,
    /// suitable for driving a syntax highlighter.
    #[must_use]
    pub const fn syntax_color(&self, token_type: SyntaxTokenType) -> Color {
        match token_type {
            SyntaxTokenType::Keyword => self.accent,
            SyntaxTokenType::Function => self.title,
            SyntaxTokenType::String => self.success,
            SyntaxTokenType::Number => self.warning,
            SyntaxTokenType::Comment => self.muted,
            SyntaxTokenType::Type => self.info,
            SyntaxTokenType::Variable => self.fg,
            SyntaxTokenType::Operator => self.accent,
            SyntaxTokenType::Punctuation => self.muted,
            SyntaxTokenType::Constant => self.warning,
            SyntaxTokenType::Macro => self.tool_call,
        }
    }
}

impl Default for Theme {
    /// Returns [`Theme::DARK`].
    fn default() -> Self {
        Self::DARK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collect every theme field into a `Vec` so presets can be compared.
    fn theme_colors(theme: &Theme) -> Vec<Color> {
        vec![
            theme.bg,
            theme.fg,
            theme.muted,
            theme.accent,
            theme.border,
            theme.title,
            theme.success,
            theme.warning,
            theme.error,
            theme.info,
            theme.diff_add,
            theme.diff_remove,
            theme.diff_context,
            theme.diff_add_bg,
            theme.diff_remove_bg,
            theme.user_msg,
            theme.assistant_msg,
            theme.system_msg,
            theme.tool_call,
            theme.selection_bg,
            theme.cursor,
        ]
    }

    #[test]
    fn default_is_dark() {
        assert_eq!(Theme::default(), Theme::DARK);
    }

    #[test]
    fn dark_is_distinct_from_light() {
        assert_ne!(Theme::DARK, Theme::LIGHT);
    }

    #[test]
    fn all_presets_are_distinct() {
        let presets = [
            Theme::DARK,
            Theme::LIGHT,
            Theme::GRUVBOX,
            Theme::NORD,
            Theme::DRACULA,
            Theme::GITHUB,
            Theme::MONOKAI,
            Theme::CATPPUCCIN,
        ];
        for (i, a) in presets.iter().enumerate() {
            for (j, b) in presets.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "presets {i} and {j} are identical");
                }
            }
        }
    }

    #[test]
    fn each_preset_has_distinct_backgrounds() {
        let bgs = [
            Theme::DARK.bg,
            Theme::LIGHT.bg,
            Theme::GRUVBOX.bg,
            Theme::NORD.bg,
            Theme::DRACULA.bg,
            Theme::GITHUB.bg,
            Theme::MONOKAI.bg,
            Theme::CATPPUCCIN.bg,
        ];
        for (i, a) in bgs.iter().enumerate() {
            for (j, b) in bgs.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "preset backgrounds {i} and {j} collide");
                }
            }
        }
    }

    #[test]
    fn each_preset_has_distinct_accents() {
        let accents = [
            Theme::DARK.accent,
            Theme::LIGHT.accent,
            Theme::GRUVBOX.accent,
            Theme::NORD.accent,
            Theme::DRACULA.accent,
            Theme::GITHUB.accent,
            Theme::MONOKAI.accent,
            Theme::CATPPUCCIN.accent,
        ];
        for (i, a) in accents.iter().enumerate() {
            for (j, b) in accents.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "preset accents {i} and {j} collide");
                }
            }
        }
    }

    #[test]
    fn each_preset_has_distinct_color_sets() {
        let presets = [
            Theme::DARK,
            Theme::LIGHT,
            Theme::GRUVBOX,
            Theme::NORD,
            Theme::DRACULA,
            Theme::GITHUB,
            Theme::MONOKAI,
            Theme::CATPPUCCIN,
        ];
        let sets: Vec<Vec<Color>> = presets.iter().map(theme_colors).collect();
        for (i, a) in sets.iter().enumerate() {
            for (j, b) in sets.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "preset color sets {i} and {j} collide");
                }
            }
        }
    }

    #[test]
    fn syntax_color_covers_all_variants() {
        let theme = Theme::DARK;
        let tokens = [
            SyntaxTokenType::Keyword,
            SyntaxTokenType::Function,
            SyntaxTokenType::String,
            SyntaxTokenType::Number,
            SyntaxTokenType::Comment,
            SyntaxTokenType::Type,
            SyntaxTokenType::Variable,
            SyntaxTokenType::Operator,
            SyntaxTokenType::Punctuation,
            SyntaxTokenType::Constant,
            SyntaxTokenType::Macro,
        ];
        for token in tokens {
            let color = theme.syntax_color(token);
            // Every syntax color should be opaque (alpha == 255).
            assert_eq!(color.a, 255, "syntax color for {token:?} is not opaque");
        }
    }

    #[test]
    fn syntax_color_keyword_is_accent() {
        let theme = Theme::DRACULA;
        assert_eq!(theme.syntax_color(SyntaxTokenType::Keyword), theme.accent);
        assert_eq!(theme.syntax_color(SyntaxTokenType::String), theme.success);
        assert_eq!(theme.syntax_color(SyntaxTokenType::Comment), theme.muted);
    }
}
