//! A headless backend for testing — no real terminal required.
//!
//! Records drawn cells into an internal buffer and lets tests feed
//! pre-scripted events. Useful for snapshot testing widgets and layouts
//! without spawning a real terminal.

use std::collections::VecDeque;

use crate::backend::Backend;
use crate::buffer::Rect;
use crate::cell::Cell;
use crate::error::Result;
use crate::event::Event;

/// A headless backend for testing.
///
/// Construct with [`HeadlessBackend::new`] giving a terminal size, then
/// pre-load events with [`HeadlessBackend::push_event`]. After running a
/// frame, inspect the drawn output via [`HeadlessBackend::cell`],
/// [`HeadlessBackend::row_str`], or [`HeadlessBackend::buffer`].
pub struct HeadlessBackend {
    width: u16,
    height: u16,
    events: VecDeque<Event>,
    drawn: Vec<Cell>,
    cursor: Option<(u16, u16)>,
}

impl HeadlessBackend {
    /// Construct a headless backend with the given terminal size.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            events: VecDeque::new(),
            drawn: vec![Cell::blank(); usize::from(width) * usize::from(height)],
            cursor: None,
        }
    }

    /// Pre-load an event to be returned by the next [`Backend::poll`] call.
    pub fn push_event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    /// Pre-load multiple events.
    pub fn push_events(&mut self, events: impl IntoIterator<Item = Event>) {
        for e in events {
            self.events.push_back(e);
        }
    }

    /// The cell drawn at `(x, y)`, or `None` if out of bounds.
    pub fn cell(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.width && y < self.height {
            self.drawn
                .get(usize::from(y) * usize::from(self.width) + usize::from(x))
        } else {
            None
        }
    }

    /// A row of drawn cells as a plain string (graphemes concatenated,
    /// blanks become spaces).
    pub fn row_str(&self, y: u16) -> Option<String> {
        if y >= self.height {
            return None;
        }
        let mut s = String::new();
        for x in 0..self.width {
            match self.cell(x, y) {
                Some(c) if !c.is_blank() => s.push_str(&c.grapheme),
                _ => s.push(' '),
            }
        }
        Some(s)
    }

    /// All drawn rows as strings.
    pub fn rows(&self) -> Vec<String> {
        (0..self.height)
            .map(|y| self.row_str(y).unwrap_or_default())
            .collect()
    }

    /// The full drawn buffer as a single string with newlines between rows.
    pub fn buffer(&self) -> String {
        self.rows().join("\n")
    }

    /// The last cursor position set by the backend, if any.
    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }

    /// Clear the drawn buffer (between test frames).
    pub fn clear_drawn(&mut self) {
        self.drawn.fill(Cell::blank());
        self.cursor = None;
    }
}

impl Backend for HeadlessBackend {
    fn enter(&mut self) -> Result<()> {
        Ok(())
    }

    fn leave(&mut self) -> Result<()> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.cursor = None;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        self.cursor = Some((x, y));
        Ok(())
    }

    fn size(&self) -> Result<(u16, u16)> {
        Ok((self.width, self.height))
    }

    fn poll(&mut self, _timeout_ms: u64) -> Result<Option<Event>> {
        Ok(self.events.pop_front())
    }

    fn begin_frame(&mut self) -> Result<()> {
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.clear_drawn();
        Ok(())
    }

    fn draw_cell(&mut self, x: u16, y: u16, cell: &Cell) -> Result<()> {
        if let Some(dst) = self
            .drawn
            .get_mut(usize::from(y) * usize::from(self.width) + usize::from(x))
        {
            *dst = cell.clone();
        }
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect, cell: &Cell) -> Result<()> {
        for y in rect.y..rect.bottom().min(self.height) {
            for x in rect.x..rect.right().min(self.width) {
                if let Some(dst) = self
                    .drawn
                    .get_mut(usize::from(y) * usize::from(self.width) + usize::from(x))
                {
                    *dst = cell.clone();
                    dst.grapheme = compact_str::CompactString::new("");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::style::Style;

    #[test]
    fn draws_and_reads_back() {
        let mut backend = HeadlessBackend::new(10, 1);
        let cell = Cell::from_grapheme("x", Style::empty());
        backend.draw_cell(0, 0, &cell).unwrap();
        backend.draw_cell(9, 0, &cell).unwrap();
        assert_eq!(backend.row_str(0).unwrap(), "x        x");
    }

    #[test]
    fn events_dequeue_in_order() {
        let mut backend = HeadlessBackend::new(10, 1);
        backend.push_event(Event::Wakeup);
        backend.push_event(Event::FocusGained);
        assert!(matches!(backend.poll(0).unwrap(), Some(Event::Wakeup)));
        assert!(matches!(backend.poll(0).unwrap(), Some(Event::FocusGained)));
        assert!(backend.poll(0).unwrap().is_none());
    }

    #[test]
    fn fill_rect_blanks_graphemes() {
        let mut backend = HeadlessBackend::new(5, 1);
        backend
            .draw_cell(0, 0, &Cell::from_grapheme("x", Style::empty()))
            .unwrap();
        let bg_cell = {
            let mut c = Cell::blank();
            c.style.bg = Color::RED;
            c
        };
        backend.fill_rect(Rect::new(0, 0, 5, 1), &bg_cell).unwrap();
        assert_eq!(backend.row_str(0).unwrap(), "     ");
        assert_eq!(backend.cell(0, 0).unwrap().style.bg, Color::RED);
    }
}
