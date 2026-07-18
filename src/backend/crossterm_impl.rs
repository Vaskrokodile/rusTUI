//! Reference backend using `crossterm`.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Color as CColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{
        Clear as TClear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use crate::backend::Backend;
use crate::buffer::Rect;
use crate::cell::Cell;
use crate::color::Color;
use crate::error::{Error, Result};
use crate::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crate::style::Attr;

/// A `crossterm`-backed [`Backend`].
pub struct CrosstermBackend {
    out: std::io::Stdout,
    last_fg: Color,
    last_bg: Color,
    last_attr: Attr,
}

impl CrosstermBackend {
    /// Construct a new backend writing to stdout.
    pub fn new() -> Self {
        Self {
            out: stdout(),
            last_fg: Color::TRANSPARENT,
            last_bg: Color::TRANSPARENT,
            last_attr: Attr::NONE,
        }
    }

    fn apply_style(&mut self, cell: &Cell) -> Result<()> {
        if cell.style.fg != self.last_fg {
            queue!(self.out, SetForegroundColor(to_crossterm(cell.style.fg)))?;
            self.last_fg = cell.style.fg;
        }
        if cell.style.bg != self.last_bg {
            queue!(self.out, SetBackgroundColor(to_crossterm(cell.style.bg)))?;
            self.last_bg = cell.style.bg;
        }
        if cell.style.attr != self.last_attr {
            // Reset attributes by emitting the inverse set, then apply new ones.
            for cmd in attr_commands(cell.style.attr, self.last_attr) {
                queue!(self.out, SetAttribute(cmd))?;
            }
            self.last_attr = cell.style.attr;
        }
        Ok(())
    }
}

impl Default for CrosstermBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn to_crossterm(c: Color) -> CColor {
    if c == Color::TRANSPARENT {
        CColor::Reset
    } else {
        CColor::Rgb {
            r: c.r,
            g: c.g,
            b: c.b,
        }
    }
}

fn attr_commands(new: Attr, old: Attr) -> Vec<crossterm::style::Attribute> {
    use crossterm::style::Attribute as A;
    let mut cmds = Vec::new();
    if new.contains(Attr::BOLD) && !old.contains(Attr::BOLD) {
        cmds.push(A::Bold);
    } else if !new.contains(Attr::BOLD) && old.contains(Attr::BOLD) {
        cmds.push(A::NormalIntensity);
    }
    if new.contains(Attr::ITALIC) && !old.contains(Attr::ITALIC) {
        cmds.push(A::Italic);
    } else if !new.contains(Attr::ITALIC) && old.contains(Attr::ITALIC) {
        cmds.push(A::NoItalic);
    }
    if new.contains(Attr::UNDERLINE) && !old.contains(Attr::UNDERLINE) {
        cmds.push(A::Underlined);
    } else if !new.contains(Attr::UNDERLINE) && old.contains(Attr::UNDERLINE) {
        cmds.push(A::NoUnderline);
    }
    if new.contains(Attr::REVERSE) && !old.contains(Attr::REVERSE) {
        cmds.push(A::Reverse);
    } else if !new.contains(Attr::REVERSE) && old.contains(Attr::REVERSE) {
        cmds.push(A::NoReverse);
    }
    if new.contains(Attr::DIM) && !old.contains(Attr::DIM) {
        cmds.push(A::Dim);
    }
    if new.contains(Attr::STRIKE) && !old.contains(Attr::STRIKE) {
        cmds.push(A::CrossedOut);
    } else if !new.contains(Attr::STRIKE) && old.contains(Attr::STRIKE) {
        cmds.push(A::NotCrossedOut);
    }
    cmds
}

impl Backend for CrosstermBackend {
    fn enter(&mut self) -> Result<()> {
        execute!(self.out, EnterAlternateScreen, DisableLineWrap, Hide)?;
        crossterm::terminal::enable_raw_mode().map_err(|e| Error::Backend(e))?;
        Ok(())
    }

    fn leave(&mut self) -> Result<()> {
        crossterm::terminal::disable_raw_mode().map_err(Error::Backend)?;
        execute!(self.out, Show, EnableLineWrap, LeaveAlternateScreen)?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        queue!(self.out, Hide)?;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        queue!(self.out, Show)?;
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        queue!(self.out, MoveTo(x, y))?;
        Ok(())
    }

    fn size(&self) -> Result<(u16, u16)> {
        let (w, h) = crossterm::terminal::size().map_err(Error::Backend)?;
        Ok((w, h))
    }

    fn poll(&mut self, timeout_ms: u64) -> Result<Option<Event>> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            let remaining = deadline - now;
            if crossterm::event::poll(remaining).map_err(Error::Backend)? {
                let ev = crossterm::event::read().map_err(Error::Backend)?;
                return Ok(convert_event(ev));
            }
        }
    }

    fn begin_frame(&mut self) -> Result<()> {
        // Begin Synchronized Update (DECSET 2026).
        write!(self.out, "\x1B[?2026h")?;
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        write!(self.out, "\x1B[?2026l")?;
        self.out.flush().map_err(Error::Backend)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        queue!(self.out, TClear(ClearType::All))?;
        self.last_fg = Color::TRANSPARENT;
        self.last_bg = Color::TRANSPARENT;
        self.last_attr = Attr::NONE;
        Ok(())
    }

    fn draw_cell(&mut self, x: u16, y: u16, cell: &Cell) -> Result<()> {
        queue!(self.out, MoveTo(x, y))?;
        self.apply_style(cell)?;
        if cell.is_blank() {
            write!(self.out, " ")?;
        } else if cell.wide_start {
            write!(self.out, "{}", cell.grapheme)?;
        }
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect, cell: &Cell) -> Result<()> {
        self.apply_style(cell)?;
        for y in rect.y..rect.bottom() {
            queue!(self.out, MoveTo(rect.x, y))?;
            for _ in rect.x..rect.right() {
                write!(self.out, " ")?;
            }
        }
        Ok(())
    }
}

fn convert_event(ev: crossterm::event::Event) -> Option<Event> {
    use crossterm::event::Event as CE;
    match ev {
        CE::Key(k) => {
            let code = match k.code {
                crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
                crossterm::event::KeyCode::Enter => KeyCode::Enter,
                crossterm::event::KeyCode::Tab => KeyCode::Tab,
                crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
                crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
                crossterm::event::KeyCode::Esc => KeyCode::Esc,
                crossterm::event::KeyCode::Up => KeyCode::Up,
                crossterm::event::KeyCode::Down => KeyCode::Down,
                crossterm::event::KeyCode::Left => KeyCode::Left,
                crossterm::event::KeyCode::Right => KeyCode::Right,
                crossterm::event::KeyCode::Home => KeyCode::Home,
                crossterm::event::KeyCode::End => KeyCode::End,
                crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
                crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
                crossterm::event::KeyCode::Delete => KeyCode::Delete,
                crossterm::event::KeyCode::Insert => KeyCode::Insert,
                crossterm::event::KeyCode::F(n) => KeyCode::F(n),
                crossterm::event::KeyCode::Null => return None,
                _ => return None,
            };
            let mods = convert_modifiers(k.modifiers);
            Some(Event::Key(KeyEvent {
                code,
                modifiers: mods,
            }))
        }
        CE::Mouse(m) => Some(Event::Mouse(MouseEvent {
            kind: match m.kind {
                crossterm::event::MouseEventKind::Down(b) => {
                    MouseEventKind::Down(convert_button(b))
                }
                crossterm::event::MouseEventKind::Up(b) => MouseEventKind::Up(convert_button(b)),
                crossterm::event::MouseEventKind::Drag(b) => {
                    MouseEventKind::Drag(convert_button(b))
                }
                crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
                crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
                crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
                _ => return None,
            },
            x: m.column,
            y: m.row,
            modifiers: convert_modifiers(m.modifiers),
        })),
        CE::Resize(w, h) => Some(Event::Resize(w, h)),
        CE::FocusGained => Some(Event::FocusGained),
        CE::FocusLost => Some(Event::FocusLost),
        CE::Paste(s) => Some(Event::Paste(s)),
    }
}

fn convert_modifiers(m: crossterm::event::KeyModifiers) -> KeyModifiers {
    let mut out = KeyModifiers::NONE;
    if m.contains(crossterm::event::KeyModifiers::SHIFT) {
        out |= KeyModifiers::SHIFT;
    }
    if m.contains(crossterm::event::KeyModifiers::CONTROL) {
        out |= KeyModifiers::CONTROL;
    }
    if m.contains(crossterm::event::KeyModifiers::ALT) {
        out |= KeyModifiers::ALT;
    }
    out
}

fn convert_button(b: crossterm::event::MouseButton) -> MouseButton {
    match b {
        crossterm::event::MouseButton::Left => MouseButton::Left,
        crossterm::event::MouseButton::Right => MouseButton::Right,
        crossterm::event::MouseButton::Middle => MouseButton::Middle,
    }
}
