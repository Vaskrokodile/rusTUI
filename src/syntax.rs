//! Syntax highlighting via `syntect`.
//!
//! This module provides syntax highlighting for code blocks, converting source
//! code into styled [`Spans`] that can be rendered by the TUI.
//!
//! Only available when the `syntax-highlight` feature is enabled.
//!
//! ## Example
//!
//! ```no_run
//! # #[cfg(feature = "syntax-highlight")]
//! # {
//! use rustui::syntax;
//! use rustui::Theme;
//!
//! let spans = syntax::highlight("fn main() { println!(\"hi\"); }", "rust", &Theme::DARK);
//! # }
//! ```

use crate::style::Style;
use crate::text::Spans;
use crate::theme::{SyntaxTokenType, Theme};

/// Highlight a code string using `syntect`, returning styled spans.
///
/// If the language is not recognized, or if syntax highlighting fails for any
/// reason, the function falls back to returning the code as plain spans with
/// the theme's default foreground color.
#[must_use]
pub fn highlight(code: &str, language: &str, theme: &Theme) -> Spans {
    highlight_with_syntect(code, language, theme)
}

/// A simple syntax highlighter that uses basic regex-like rules for common
/// languages. This is a fallback when `syntect` is not available.
#[must_use]
pub fn highlight_simple(code: &str, language: &str, theme: &Theme) -> Spans {
    match language {
        "rust" | "rs" => highlight_rust(code, theme),
        "python" | "py" => highlight_python(code, theme),
        "javascript" | "js" | "typescript" | "ts" => highlight_js(code, theme),
        "json" => highlight_json(code, theme),
        "shell" | "bash" | "sh" => highlight_shell(code, theme),
        _ => Spans::plain(code),
    }
}

/// Highlight Rust code with basic rules.
fn highlight_rust(code: &str, theme: &Theme) -> Spans {
    let keywords = [
        "fn", "let", "mut", "if", "else", "for", "while", "loop", "match", "return", "struct",
        "enum", "impl", "trait", "pub", "use", "mod", "crate", "self", "super", "as", "where",
        "async", "await", "move", "ref", "static", "const", "unsafe", "extern", "type",
    ];
    highlight_with_keywords(code, &keywords, theme)
}

/// Highlight Python code with basic rules.
fn highlight_python(code: &str, theme: &Theme) -> Spans {
    let keywords = [
        "def", "class", "if", "else", "elif", "for", "while", "try", "except", "finally", "return",
        "import", "from", "as", "with", "lambda", "yield", "async", "await", "pass", "break",
        "continue", "raise", "global", "nonlocal", "None", "True", "False", "and", "or", "not",
        "in", "is",
    ];
    highlight_with_keywords(code, &keywords, theme)
}

/// Highlight JavaScript/TypeScript code with basic rules.
fn highlight_js(code: &str, theme: &Theme) -> Spans {
    let keywords = [
        "function",
        "const",
        "let",
        "var",
        "if",
        "else",
        "for",
        "while",
        "do",
        "switch",
        "case",
        "break",
        "continue",
        "return",
        "class",
        "extends",
        "super",
        "this",
        "new",
        "try",
        "catch",
        "finally",
        "throw",
        "typeof",
        "instanceof",
        "in",
        "of",
        "async",
        "await",
        "yield",
        "import",
        "export",
        "default",
        "from",
        "as",
        "null",
        "undefined",
        "true",
        "false",
    ];
    highlight_with_keywords(code, &keywords, theme)
}

/// Highlight JSON with basic rules.
fn highlight_json(code: &str, theme: &Theme) -> Spans {
    let mut spans = Spans::new();
    let string_color = theme.syntax_color(SyntaxTokenType::String);
    let number_color = theme.syntax_color(SyntaxTokenType::Number);
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Strings.
        if ch == '"' {
            let mut content = String::from("\"");
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                content.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                content.push('"');
                i += 1;
            }
            spans = spans.push_styled(content, Style::empty().fg(string_color));
            continue;
        }

        // Numbers.
        if ch.is_ascii_digit() {
            let mut content = String::new();
            while i < chars.len()
                && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '-')
            {
                content.push(chars[i]);
                i += 1;
            }
            spans = spans.push_styled(content, Style::empty().fg(number_color));
            continue;
        }

        spans = spans.push_plain(ch.to_string());
        i += 1;
    }

    spans
}

/// Highlight shell/bash with basic rules.
fn highlight_shell(code: &str, theme: &Theme) -> Spans {
    let keywords = [
        "if", "then", "else", "fi", "for", "do", "done", "while", "case", "esac",
    ];
    highlight_with_keywords(code, &keywords, theme)
}

/// Generic keyword-based highlighting.
fn highlight_with_keywords(code: &str, keywords: &[&str], theme: &Theme) -> Spans {
    let mut spans = Spans::new();
    let keyword_color = theme.syntax_color(SyntaxTokenType::Keyword);
    let string_color = theme.syntax_color(SyntaxTokenType::String);
    let comment_color = theme.syntax_color(SyntaxTokenType::Comment);
    let number_color = theme.syntax_color(SyntaxTokenType::Number);
    let func_color = theme.syntax_color(SyntaxTokenType::Function);

    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Comments.
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let rest: String = chars[i..].iter().take_while(|&&c| c != '\n').collect();
            spans = spans.push_styled(rest, Style::empty().fg(comment_color).italic());
            break;
        }
        if ch == '#' {
            let rest: String = chars[i..].iter().take_while(|&&c| c != '\n').collect();
            i += rest.len();
            spans = spans.push_styled(rest, Style::empty().fg(comment_color).italic());
            continue;
        }

        // Strings.
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let mut content = String::new();
            content.push(quote);
            i += 1;
            while i < chars.len() && chars[i] != quote {
                content.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                content.push(quote);
                i += 1;
            }
            spans = spans.push_styled(content, Style::empty().fg(string_color));
            continue;
        }

        // Numbers.
        if ch.is_ascii_digit() {
            let mut content = String::new();
            while i < chars.len()
                && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_')
            {
                content.push(chars[i]);
                i += 1;
            }
            spans = spans.push_styled(content, Style::empty().fg(number_color));
            continue;
        }

        // Identifiers and keywords.
        if ch.is_alphabetic() || ch == '_' {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                i += 1;
            }

            // Check if it's a keyword.
            if keywords.contains(&word.as_str()) {
                spans = spans.push_styled(word, Style::empty().fg(keyword_color).bold());
            } else if i < chars.len() && chars[i] == '(' {
                // Function call.
                spans = spans.push_styled(word, Style::empty().fg(func_color));
            } else {
                spans = spans.push_plain(word);
            }
            continue;
        }

        // Single character.
        spans = spans.push_plain(ch.to_string());
        i += 1;
    }

    spans
}

#[cfg(feature = "syntax-highlight")]
fn highlight_with_syntect(code: &str, language: &str, theme: &Theme) -> Spans {
    use syntect::highlighting::{Theme as SynTheme, ThemeSet};
    use syntect::parsing::SyntaxSet;
    use syntect::util::as_24_bit_terminal_ansi;

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Find syntax for the language.
    let syntax = ps
        .find_syntax_by_extension(language)
        .or_else(|| ps.find_syntax_by_name(language))
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let syn_theme = ts
        .themes
        .get("base16-ocean.dark")
        .unwrap_or(&ts.themes["base16-eighties.dark"]);

    let h = syntect::easy::HighlightLines::new(syntax, syn_theme);
    let mut spans = Spans::new();

    for line in code.lines() {
        let regions = h.highlight_line(line, &ps).unwrap_or_default();
        for (style, text) in regions {
            let color = Color::rgb(style.foreground.r, style.foreground.g, style.foreground.b);
            let mut s = Style::empty().fg(color);
            if style
                .font_style
                .contains(syntect::highlighting::FontStyle::BOLD)
            {
                s = s.bold();
            }
            if style
                .font_style
                .contains(syntect::highlighting::FontStyle::ITALIC)
            {
                s = s.italic();
            }
            if style
                .font_style
                .contains(syntect::highlighting::FontStyle::UNDERLINE)
            {
                s = s.underline();
            }
            spans = spans.push_styled(text.to_string(), s);
        }
        spans = spans.push_plain("\n");
    }

    // If we got something, return it; otherwise fall back.
    if spans.spans.is_empty() {
        highlight_simple(code, language, theme)
    } else {
        spans
    }
}

#[cfg(not(feature = "syntax-highlight"))]
fn highlight_with_syntect(code: &str, language: &str, theme: &Theme) -> Spans {
    highlight_simple(code, language, theme)
}

/// Map a syntect style to a RusTUI style.
#[cfg(feature = "syntax-highlight")]
fn syntect_style_to_style(style: &syntect::highlighting::Style) -> Style {
    let color = Color::rgb(style.foreground.r, style.foreground.g, style.foreground.b);
    let mut s = Style::empty().fg(color);
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        s = s.bold();
    }
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        s = s.italic();
    }
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::UNDERLINE)
    {
        s = s.underline();
    }
    s
}

/// Detect a language from a file extension.
#[must_use]
pub fn detect_language(filename: &str) -> &str {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "json" => "json",
        "sh" | "bash" => "shell",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "php" => "php",
        "html" | "htm" => "html",
        "css" => "css",
        "md" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "xml" => "xml",
        "sql" => "sql",
        "lua" => "lua",
        "kt" => "kotlin",
        "swift" => "swift",
        "scala" => "scala",
        "dart" => "dart",
        "dockerfile" => "dockerfile",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Attr;

    #[test]
    fn highlight_rust_basic() {
        let spans = highlight_simple("fn main() {}", "rust", &Theme::DARK);
        // Should have at least "fn" as a keyword.
        assert!(!spans.spans.is_empty());
        let has_keyword = spans
            .spans
            .iter()
            .any(|s| s.style.attr.contains(Attr::BOLD) && (s.text == "fn" || s.text == "fn "));
        assert!(has_keyword);
    }

    #[test]
    fn highlight_python_basic() {
        let spans = highlight_simple("def foo(): pass", "python", &Theme::DARK);
        assert!(!spans.spans.is_empty());
    }

    #[test]
    fn highlight_string() {
        let spans = highlight_simple("let x = \"hello\";", "rust", &Theme::DARK);
        let has_string = spans.spans.iter().any(|s| s.text.contains("hello"));
        assert!(has_string);
    }

    #[test]
    fn highlight_comment() {
        let spans = highlight_simple("// this is a comment\nfn main() {}", "rust", &Theme::DARK);
        let has_comment = spans
            .spans
            .iter()
            .any(|s| s.style.attr.contains(Attr::ITALIC) && s.text.contains("comment"));
        assert!(has_comment);
    }

    #[test]
    fn highlight_number() {
        let spans = highlight_simple("let x = 42;", "rust", &Theme::DARK);
        let has_number = spans.spans.iter().any(|s| s.text.contains("42"));
        assert!(has_number);
    }

    #[test]
    fn highlight_unknown_language() {
        let spans = highlight_simple("some code", "unknown_lang", &Theme::DARK);
        assert_eq!(spans.to_plain(), "some code");
    }

    #[test]
    fn detect_language_rust() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("script.py"), "python");
        assert_eq!(detect_language("app.js"), "javascript");
        assert_eq!(detect_language("data.json"), "json");
        assert_eq!(detect_language("run.sh"), "shell");
    }

    #[test]
    fn detect_language_unknown() {
        assert_eq!(detect_language("file.xyz"), "");
        assert_eq!(detect_language("noextension"), "");
    }

    #[test]
    fn highlight_function_call() {
        let spans = highlight_simple("foo()", "rust", &Theme::DARK);
        let has_func = spans.spans.iter().any(|s| s.text == "foo");
        assert!(has_func);
    }
}
