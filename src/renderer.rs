//! Double-buffered renderer with diff detection.
//!
//! The renderer keeps two [`Buffer`]s: the last frame that was sent to the
//! backend (`prev`) and the frame currently being built (`curr`). When
//! [`Renderer::present`] is called, only cells where `curr` differs from
//! `prev` are re-emitted through the [`Backend`].

use crate::backend::Backend;
use crate::buffer::Buffer;
use crate::cell::Cell;
use crate::color::Color;
use crate::error::Result;

/// The renderer.
pub struct Renderer {
    /// The buffer for the frame currently being built.
    pub curr: Buffer,
    /// The buffer for the last presented frame.
    pub prev: Buffer,
}

impl Renderer {
    /// Construct a renderer for a viewport of `width` x `height`.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            curr: Buffer::empty(width, height),
            prev: Buffer::empty(width, height),
        }
    }

    /// Resize both buffers.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.curr.resize(width, height);
        self.prev.resize(width, height);
        // Force a full repaint on the next present.
        self.prev.clear();
    }

    /// Begin a new frame: clear the current buffer.
    pub fn begin(&mut self) {
        self.curr.clear();
    }

    /// Borrow the in-progress frame buffer for widgets to write into.
    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.curr
    }

    /// Present the current frame to `backend`, emitting only the cells that
    /// changed since the previous frame.
    pub fn present(&mut self, backend: &mut dyn Backend) -> Result<()> {
        backend.begin_frame()?;
        if self.curr.width != self.prev.width || self.curr.height != self.prev.height {
            // Size changed: full repaint.
            backend.clear()?;
            for y in 0..self.curr.height {
                for x in 0..self.curr.width {
                    if let Some(cell) = self.curr.cell(x, y) {
                        backend.draw_cell(x, y, cell)?;
                    }
                }
            }
        } else {
            // Diff: only changed cells.
            for y in 0..self.curr.height {
                for x in 0..self.curr.width {
                    let curr = self.curr.cell(x, y).expect("in-bounds");
                    let prev = self.prev.cell(x, y).expect("in-bounds");
                    if curr != prev {
                        backend.draw_cell(x, y, curr)?;
                    }
                }
            }
        }
        backend.end_frame()?;
        std::mem::swap(&mut self.curr, &mut self.prev);
        self.curr.clear();
        Ok(())
    }

    /// Force a full repaint on the next [`Renderer::present`].
    pub fn invalidate(&mut self) {
        self.prev.clear();
    }

    /// The default cell used to fill blank regions: a space with the default
    /// background. Backends may override via [`Backend::fill_rect`].
    pub fn blank_cell(bg: Color) -> Cell {
        let mut c = Cell::blank();
        c.style.bg = bg;
        c
    }
}
