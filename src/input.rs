//! Input parsing utilities.
//!
//! Most users don't need this — the [`crate::backend::Backend`] already
//! returns parsed [`Event`]s. This module exists for backends that want to
//! share a common parser, and for tests.

use crate::event::{KeyCode, KeyEvent, KeyModifiers};

/// Parse a single printable-character byte as a key event with no modifiers.
pub fn from_byte(b: u8) -> Option<KeyEvent> {
    let c = b as char;
    if c.is_control() {
        None
    } else {
        Some(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        })
    }
}
