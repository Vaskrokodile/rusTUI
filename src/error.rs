//! Error types for RusTUI.

use thiserror::Error;

/// A RusTUI error.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error from the terminal backend.
    #[error("backend io: {0}")]
    Backend(#[from] std::io::Error),

    /// The terminal is too small for the requested layout.
    #[error("terminal too small: need {needed}x{need_h}, got {got_w}x{got_h}")]
    TerminalTooSmall {
        /// Required width.
        needed: u16,
        /// Required height.
        need_h: u16,
        /// Actual width.
        got_w: u16,
        /// Actual height.
        got_h: u16,
    },

    /// The application was asked to stop (e.g. user pressed Ctrl-C / `q`).
    #[error("application exited")]
    Exit,

    /// A backend-specific error reported as a boxed dynamic error.
    #[error("backend: {0}")]
    BackendDyn(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// A miscellaneous error.
    #[error("{0}")]
    Other(String),
}

/// Convenience `Result` alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;
