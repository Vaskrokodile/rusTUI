//! The terminal backend abstraction.
//!
//! RusTUI never writes bytes to the terminal directly. Everything goes through
//! a [`Backend`], which knows how to enter/leave raw mode, query the terminal
//! size, poll for input events, and emit styled cells. The crate ships a
//! reference `crossterm` backend behind the `backend-crossterm` feature; users
//! can implement their own for `termion`, raw file-descriptor I/O, or a
//! headless test harness.

use crate::buffer::Rect;
use crate::cell::Cell;
use crate::error::Result;
use crate::event::Event;

/// A terminal backend.
///
/// All methods are synchronous and may block; the async event loop in
/// [`crate::app::App`] wraps input polling in `tokio::task::spawn_blocking` so
/// backends don't need to be async-aware.
pub trait Backend: Send {
    /// Enter raw mode and prepare the terminal for full-screen rendering.
    fn enter(&mut self) -> Result<()>;

    /// Leave raw mode and restore the terminal.
    fn leave(&mut self) -> Result<()>;

    /// Hide the cursor.
    fn hide_cursor(&mut self) -> Result<()>;

    /// Show the cursor.
    fn show_cursor(&mut self) -> Result<()>;

    /// Move the cursor to `(x, y)`.
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()>;

    /// Current terminal size in (width, height) columns/rows.
    fn size(&self) -> Result<(u16, u16)>;

    /// Poll for an input event, blocking up to `timeout_ms` milliseconds.
    /// `None` means "no event within the timeout".
    fn poll(&mut self, timeout_ms: u64) -> Result<Option<Event>>;

    /// Begin a frame. Backends that support synchronized output (e.g. the
    /// `Begin Synchronized Update` DECSET 2026 escape) should emit the start
    /// sequence here.
    fn begin_frame(&mut self) -> Result<()>;

    /// End a frame. Backends should flush any pending output and emit the
    /// synchronized-update end sequence if they started one.
    fn end_frame(&mut self) -> Result<()>;

    /// Clear the screen.
    fn clear(&mut self) -> Result<()>;

    /// Write a single cell at `(x, y)`. Called by the renderer only for cells
    /// that changed since the previous frame.
    fn draw_cell(&mut self, x: u16, y: u16, cell: &Cell) -> Result<()>;

    /// Fill `rect` with the background color of `cell` (used for blank regions
    /// and padding). Default implementation calls [`Backend::draw_cell`] for
    /// every position.
    fn fill_rect(&mut self, rect: Rect, cell: &Cell) -> Result<()> {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                self.draw_cell(x, y, cell)?;
            }
        }
        Ok(())
    }
}

impl Backend for Box<dyn Backend> {
    fn enter(&mut self) -> Result<()> {
        (**self).enter()
    }
    fn leave(&mut self) -> Result<()> {
        (**self).leave()
    }
    fn hide_cursor(&mut self) -> Result<()> {
        (**self).hide_cursor()
    }
    fn show_cursor(&mut self) -> Result<()> {
        (**self).show_cursor()
    }
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        (**self).move_cursor(x, y)
    }
    fn size(&self) -> Result<(u16, u16)> {
        (**self).size()
    }
    fn poll(&mut self, timeout_ms: u64) -> Result<Option<Event>> {
        (**self).poll(timeout_ms)
    }
    fn begin_frame(&mut self) -> Result<()> {
        (**self).begin_frame()
    }
    fn end_frame(&mut self) -> Result<()> {
        (**self).end_frame()
    }
    fn clear(&mut self) -> Result<()> {
        (**self).clear()
    }
    fn draw_cell(&mut self, x: u16, y: u16, cell: &Cell) -> Result<()> {
        (**self).draw_cell(x, y, cell)
    }
}

#[cfg(feature = "backend-crossterm")]
pub mod crossterm_impl;

#[cfg(feature = "backend-crossterm")]
pub use crossterm_impl::CrosstermBackend;

/// Construct the default backend for the current feature set.
#[cfg(feature = "backend-crossterm")]
pub fn default_backend() -> CrosstermBackend {
    CrosstermBackend::new()
}

#[cfg(not(feature = "backend-crossterm"))]
pub fn default_backend() -> Box<dyn Backend> {
    panic!("no default backend compiled in; enable `backend-crossterm` or supply your own");
}
