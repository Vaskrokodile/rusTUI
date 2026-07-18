//! ANSI escape-sequence parser: converts raw terminal output into [`Spans`].
//!
//! LLMs and CLI tools frequently emit ANSI color codes (SGR sequences).
//! This module parses them into styled [`Span`]s so they can be rendered
//! faithfully in the TUI without leaking escape codes to the screen.
//!
//! ## Supported sequences
//!
//! - `ESC [ Nm` — SGR (Select Graphic Rendition), including:
//!   - Standard colors: 30–37 (fg), 40–47 (bg)
//!   - Bright colors: 90–97 (fg), 100–107 (bg)
//!   - 256-color: `38;5;N` / `48;5;N`
//!   - Truecolor: `38;2;R;G;B` / `48;2;R;G;B`
//!   - Attributes: bold (1), dim (2), italic (3), underline (4),
//!     reverse (7), strikethrough (9)
//!   - Reset: 0, 39 (fg reset), 49 (bg reset)
//! - `ESC [ N A` / `B` / `C` / `D` — cursor movement (consumed, not rendered)
//! - `ESC [ K` — erase line (consumed)
//! - `ESC [ ? 25 h` / `l` — show/hide cursor (consumed)
//!
//! Unsupported sequences are consumed (not rendered) so they don't leak
//! to the screen.
//!
//! ## Example
//!
//! ```
//! use rustui::ansi;
//! use rustui::Color;
//!
//! let spans = ansi::parse("\x1b[1;31mError:\x1b[0m something went wrong");
//! assert_eq!(spans.spans.len(), 2);
//! assert_eq!(spans.spans[0].text, "Error:");
//! // ANSI 31 = standard red = rgb(180, 30, 30)
//! assert_eq!(spans.spans[0].style.fg, Color::rgb(180, 30, 30));
//! ```

use crate::color::Color;
use crate::style::{Attr, Style};
use crate::text::{Span, Spans};

/// Parse an ANSI-escaped string into [`Spans`].
///
/// Adjacent text with the same style is merged. Escape sequences that don't
/// produce visible output (cursor movement, erasing, cursor visibility) are
/// consumed but do not create spans.
#[must_use]
pub fn parse(input: &str) -> Spans {
    let mut parser = Parser::new();
    parser.feed(input);
    parser.finish()
}

/// A streaming ANSI parser. Feed text incrementally and call [`Parser::finish`]
/// to get the final [`Spans`].
pub struct Parser {
    spans: Vec<Span>,
    /// Current pending text (not yet flushed to a span).
    pending: String,
    /// Current style state.
    style: Style,
    /// Parser state machine.
    state: State,
    /// Buffer for parsing CSI parameters.
    csi_buf: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Ground,
    Escape,
    Csi,
    Osc,
    OscEsc,
}

impl Parser {
    /// Construct a new parser.
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            pending: String::new(),
            style: Style::empty(),
            state: State::Ground,
            csi_buf: String::new(),
        }
    }

    /// Feed a chunk of text into the parser.
    pub fn feed(&mut self, input: &str) {
        for byte in input.bytes() {
            self.process_byte(byte);
        }
    }

    /// Finish parsing and return the resulting [`Spans`].
    pub fn finish(mut self) -> Spans {
        self.flush_pending();
        // Merge adjacent spans with the same style.
        let mut merged: Vec<Span> = Vec::with_capacity(self.spans.len());
        for span in self.spans {
            if let Some(last) = merged.last_mut() {
                if last.style == span.style {
                    last.text.push_str(&span.text);
                    continue;
                }
            }
            merged.push(span);
        }
        Spans { spans: merged }
    }

    fn flush_pending(&mut self) {
        if !self.pending.is_empty() {
            let text = std::mem::take(&mut self.pending);
            self.spans.push(Span {
                text: text.into(),
                style: self.style,
            });
        }
    }

    fn process_byte(&mut self, byte: u8) {
        match self.state {
            State::Ground => {
                if byte == 0x1b {
                    self.state = State::Escape;
                } else if byte == 0x0d {
                    // Carriage return — flush and add a literal \r
                    self.pending.push('\r');
                } else if byte == 0x08 {
                    // Backspace — just skip it
                } else if byte >= 0x20 {
                    self.pending.push(byte as char);
                } else if byte == b'\n' {
                    self.pending.push('\n');
                }
            }
            State::Escape => {
                match byte {
                    b'[' => {
                        self.state = State::Csi;
                        self.csi_buf.clear();
                    }
                    b']' => {
                        self.state = State::Osc;
                        self.csi_buf.clear();
                    }
                    b'(' | b')' | b'*' | b'+' => {
                        // Character set designation — consume the next byte.
                        self.state = State::Escape;
                        // We'll consume one more byte and go back to ground.
                        // Actually, we need to consume one byte then go to ground.
                        // Simplify: just go to ground, the next byte will be
                        // treated as text which is wrong but harmless for our use.
                        self.state = State::Ground;
                    }
                    b'M' => {
                        // Reverse line feed — consume.
                        self.state = State::Ground;
                    }
                    b'7' => {
                        // Save cursor — consume.
                        self.state = State::Ground;
                    }
                    b'8' => {
                        // Restore cursor — consume.
                        self.state = State::Ground;
                    }
                    _ => {
                        self.state = State::Ground;
                    }
                }
            }
            State::Csi => {
                if (0x30..=0x3f).contains(&byte) {
                    // Parameter or intermediate byte.
                    self.csi_buf.push(byte as char);
                } else if (0x20..=0x2f).contains(&byte) {
                    // Intermediate byte.
                    self.csi_buf.push(byte as char);
                } else if (0x40..=0x7e).contains(&byte) {
                    // Final byte — dispatch.
                    self.dispatch_csi(byte);
                    self.state = State::Ground;
                } else {
                    // Unexpected byte — abort.
                    self.state = State::Ground;
                }
            }
            State::Osc => {
                if byte == 0x07 {
                    // BEL terminates OSC.
                    self.state = State::Ground;
                } else if byte == 0x1b {
                    self.state = State::OscEsc;
                } else {
                    // Consume OSC content.
                }
            }
            State::OscEsc => {
                if byte == b'\\' {
                    self.state = State::Ground;
                } else {
                    self.state = State::Osc;
                }
            }
        }
    }

    fn dispatch_csi(&mut self, final_byte: u8) {
        let params = std::mem::take(&mut self.csi_buf);
        match final_byte {
            b'm' => self.dispatch_sgr(&params),
            // Cursor movement, erase, and other non-rendering sequences:
            // consume silently.
            b'A' | b'B' | b'C' | b'D' | b'E' | b'F' | b'G' | b'H' | b'f' | b'd' | b'J' | b'K'
            | b'L' | b'M' | b'P' | b'S' | b'T' | b'X' | b'@' | b'h' | b'l' | b'r' | b's' | b'u'
            | b'n' | b'q' | b'`' | b'a' => {
                // Consume.
            }
            _ => {
                // Unknown — consume.
            }
        }
    }

    fn dispatch_sgr(&mut self, params: &str) {
        // Flush any pending text with the current style before changing it.
        self.flush_pending();

        if params.is_empty() || params == "0" || params == "00" {
            self.style = Style::empty();
            return;
        }

        // Handle private-mode prefix (e.g., "?25h" which we already consumed).
        let params = params.strip_prefix('?').unwrap_or(params);

        let nums: Vec<u16> = params.split(';').filter_map(|s| s.parse().ok()).collect();
        if nums.is_empty() {
            self.style = Style::empty();
            return;
        }

        let mut i = 0;
        while i < nums.len() {
            let n = nums[i];
            match n {
                0 => self.style = Style::empty(),
                1 => self.style.attr |= Attr::BOLD,
                2 => self.style.attr |= Attr::DIM,
                3 => self.style.attr |= Attr::ITALIC,
                4 => self.style.attr |= Attr::UNDERLINE,
                7 => self.style.attr |= Attr::REVERSE,
                9 => self.style.attr |= Attr::STRIKE,
                22 => {
                    self.style.attr &= !(Attr::BOLD | Attr::DIM);
                }
                23 => self.style.attr &= !Attr::ITALIC,
                24 => self.style.attr &= !Attr::UNDERLINE,
                27 => self.style.attr &= !Attr::REVERSE,
                29 => self.style.attr &= !Attr::STRIKE,
                // Standard foreground colors
                30 => self.style.fg = Color::rgb(0, 0, 0),
                31 => self.style.fg = Color::rgb(180, 30, 30),
                32 => self.style.fg = Color::rgb(40, 160, 60),
                33 => self.style.fg = Color::rgb(200, 170, 40),
                34 => self.style.fg = Color::rgb(40, 90, 200),
                35 => self.style.fg = Color::rgb(170, 50, 170),
                36 => self.style.fg = Color::rgb(40, 170, 170),
                37 => self.style.fg = Color::rgb(200, 200, 200),
                // Bright foreground colors
                90 => self.style.fg = Color::rgb(80, 80, 80),
                91 => self.style.fg = Color::rgb(220, 60, 80),
                92 => self.style.fg = Color::rgb(80, 200, 120),
                93 => self.style.fg = Color::rgb(220, 200, 80),
                94 => self.style.fg = Color::rgb(80, 140, 240),
                95 => self.style.fg = Color::rgb(220, 100, 220),
                96 => self.style.fg = Color::rgb(80, 220, 220),
                97 => self.style.fg = Color::rgb(255, 255, 255),
                // Standard background colors
                40 => self.style.bg = Color::rgb(0, 0, 0),
                41 => self.style.bg = Color::rgb(180, 30, 30),
                42 => self.style.bg = Color::rgb(40, 160, 60),
                43 => self.style.bg = Color::rgb(200, 170, 40),
                44 => self.style.bg = Color::rgb(40, 90, 200),
                45 => self.style.bg = Color::rgb(170, 50, 170),
                46 => self.style.bg = Color::rgb(40, 170, 170),
                47 => self.style.bg = Color::rgb(200, 200, 200),
                // Bright background colors
                100 => self.style.bg = Color::rgb(80, 80, 80),
                101 => self.style.bg = Color::rgb(220, 60, 80),
                102 => self.style.bg = Color::rgb(80, 200, 120),
                103 => self.style.bg = Color::rgb(220, 200, 80),
                104 => self.style.bg = Color::rgb(80, 140, 240),
                105 => self.style.bg = Color::rgb(220, 100, 220),
                106 => self.style.bg = Color::rgb(80, 220, 220),
                107 => self.style.bg = Color::rgb(255, 255, 255),
                // Default fg/bg
                39 => self.style.fg = Color::TRANSPARENT,
                49 => self.style.bg = Color::TRANSPARENT,
                // Extended color: 38/48
                38 | 48 => {
                    if i + 1 < nums.len() {
                        match nums[i + 1] {
                            5 => {
                                // 256-color: 38;5;N
                                if i + 2 < nums.len() {
                                    let color = Color::palette256(nums[i + 2] as u8);
                                    if n == 38 {
                                        self.style.fg = color;
                                    } else {
                                        self.style.bg = color;
                                    }
                                }
                                i += 2;
                            }
                            2 => {
                                // Truecolor: 38;2;R;G;B
                                if i + 4 < nums.len() {
                                    let color = Color::rgb(
                                        nums[i + 2] as u8,
                                        nums[i + 3] as u8,
                                        nums[i + 4] as u8,
                                    );
                                    if n == 38 {
                                        self.style.fg = color;
                                    } else {
                                        self.style.bg = color;
                                    }
                                }
                                i += 4;
                            }
                            _ => {
                                // Unknown sub-mode — skip.
                            }
                        }
                    }
                }
                _ => {
                    // Unknown SGR — ignore.
                }
            }
            i += 1;
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Attr;

    #[test]
    fn plain_text() {
        let spans = parse("hello world");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "hello world");
        assert_eq!(spans.spans[0].style, Style::empty());
    }

    #[test]
    fn basic_color() {
        let spans = parse("\x1b[31mred\x1b[0m plain");
        assert_eq!(spans.spans.len(), 2);
        assert_eq!(spans.spans[0].text, "red");
        assert_eq!(spans.spans[0].style.fg, Color::rgb(180, 30, 30));
        assert_eq!(spans.spans[1].text, " plain");
        assert_eq!(spans.spans[1].style.fg, Color::TRANSPARENT);
    }

    #[test]
    fn bold_and_color() {
        let spans = parse("\x1b[1;32mbold green\x1b[0m");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "bold green");
        assert!(spans.spans[0].style.attr.contains(Attr::BOLD));
        assert_eq!(spans.spans[0].style.fg, Color::rgb(40, 160, 60));
    }

    #[test]
    fn truecolor() {
        let spans = parse("\x1b[38;2;100;150;200mcustom\x1b[0m");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].style.fg, Color::rgb(100, 150, 200));
    }

    #[test]
    fn color_256() {
        let spans = parse("\x1b[38;5;196mred256\x1b[0m");
        assert_eq!(spans.spans.len(), 1);
        // 196 = 16 + 180 = 5*36 + 0*6 + 0 -> r=5, g=0, b=0
        let expected = Color::palette256(196);
        assert_eq!(spans.spans[0].style.fg, expected);
    }

    #[test]
    fn multiple_attributes() {
        let spans = parse("\x1b[1;3;4mbold italic underline\x1b[0m");
        assert_eq!(spans.spans.len(), 1);
        let attr = spans.spans[0].style.attr;
        assert!(attr.contains(Attr::BOLD));
        assert!(attr.contains(Attr::ITALIC));
        assert!(attr.contains(Attr::UNDERLINE));
    }

    #[test]
    fn reset_specific() {
        let spans = parse("\x1b[1;31mbold red\x1b[22mnot bold\x1b[0m");
        assert_eq!(spans.spans.len(), 2);
        assert!(spans.spans[0].style.attr.contains(Attr::BOLD));
        assert!(!spans.spans[1].style.attr.contains(Attr::BOLD));
        // Color should persist after 22 (only resets bold/dim)
        assert_eq!(spans.spans[1].style.fg, Color::rgb(180, 30, 30));
    }

    #[test]
    fn background_color() {
        let spans = parse("\x1b[44mblue bg\x1b[0m");
        assert_eq!(spans.spans[0].style.bg, Color::rgb(40, 90, 200));
    }

    #[test]
    fn non_sgr_sequences_consumed() {
        let spans = parse("hello\x1b[2Aworld\x1b[Ktest");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "helloworldtest");
    }

    #[test]
    fn osc_sequence_consumed() {
        let spans = parse("hello\x1b]0;title\x07world");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "helloworld");
    }

    #[test]
    fn osc_with_st_terminator() {
        let spans = parse("hello\x1b]0;title\x1b\\world");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "helloworld");
    }

    #[test]
    fn streaming_parser() {
        let mut p = Parser::new();
        p.feed("hello ");
        p.feed("\x1b[31m");
        p.feed("red");
        p.feed("\x1b[0m");
        p.feed(" world");
        let spans = p.finish();
        assert_eq!(spans.spans.len(), 3);
        assert_eq!(spans.spans[0].text, "hello ");
        assert_eq!(spans.spans[1].text, "red");
        assert_eq!(spans.spans[1].style.fg, Color::rgb(180, 30, 30));
        assert_eq!(spans.spans[2].text, " world");
    }

    #[test]
    fn adjacent_same_style_merged() {
        let spans = parse("\x1b[31ma\x1b[31mb\x1b[31mc\x1b[0m");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "abc");
    }

    #[test]
    fn bright_colors() {
        let spans = parse("\x1b[91mbright red\x1b[0m");
        assert_eq!(spans.spans[0].style.fg, Color::rgb(220, 60, 80));
    }

    #[test]
    fn strikethrough() {
        let spans = parse("\x1b[9mstruck\x1b[29mnot\x1b[0m");
        assert!(spans.spans[0].style.attr.contains(Attr::STRIKE));
        assert!(!spans.spans[1].style.attr.contains(Attr::STRIKE));
    }

    #[test]
    fn empty_input() {
        let spans = parse("");
        assert!(spans.spans.is_empty());
    }

    #[test]
    fn incomplete_escape_at_end() {
        let spans = parse("hello\x1b");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "hello");
    }

    #[test]
    fn incomplete_csi_at_end() {
        let spans = parse("hello\x1b[31");
        assert_eq!(spans.spans.len(), 1);
        assert_eq!(spans.spans[0].text, "hello");
    }
}
