//! Keybinding management for RusTUI.
//!
//! This module provides a small, dependency-light system for mapping
//! [`KeyBinding`]s (a key plus optional modifiers) to action names. Action
//! names are stored as [`compact_str::CompactString`]s so that short strings
//! like `"quit"` or `"cursor_left"` don't heap-allocate.
//!
//! Two pre-defined binding sets are shipped:
//!
//! - [`KeyBindings::emacs`] — standard emacs/readline bindings.
//! - [`KeyBindings::vim_normal`] — a small sample of vim normal-mode bindings.
//!
//! ```
//! use rustui::keybindings::KeyBindings;
//! use rustui::event::{KeyCode, KeyEvent, KeyModifiers};
//!
//! let mut kb = KeyBindings::emacs();
//! let event = KeyEvent {
//!     code: KeyCode::Char('a'),
//!     modifiers: KeyModifiers::CONTROL,
//! };
//! assert_eq!(kb.lookup_event(&event), Some("move_beginning_of_line"));
//! ```

use std::collections::HashMap;
use std::fmt;

use compact_str::CompactString;
use thiserror::Error;

use crate::event::{KeyCode, KeyEvent, KeyModifiers};

/// A key combination: a [`KeyCode`] plus any active [`KeyModifiers`].
///
/// This is the unit used as a map key in [`KeyBindings`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    /// The key that was pressed.
    pub key: KeyCode,
    /// The modifiers held when the key was pressed.
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Create a new key binding from a key code and modifiers.
    #[must_use]
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }
}

impl From<KeyEvent> for KeyBinding {
    fn from(event: KeyEvent) -> Self {
        Self {
            key: event.code,
            modifiers: event.modifiers,
        }
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Modifiers are rendered in a stable order: Ctrl, Alt, Shift.
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            f.write_str("Ctrl+")?;
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            f.write_str("Alt+")?;
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            f.write_str("Shift+")?;
        }
        write_key_code(f, self.key)
    }
}

/// Render a [`KeyCode`] without its modifiers.
fn write_key_code(f: &mut fmt::Formatter<'_>, key: KeyCode) -> fmt::Result {
    match key {
        KeyCode::Char(c) => {
            // Render the literal character; callers see modifiers separately
            // (e.g. "Ctrl+x", "Shift+A").
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            f.write_str(s)
        }
        KeyCode::Enter => f.write_str("Enter"),
        KeyCode::Tab => f.write_str("Tab"),
        KeyCode::BackTab => f.write_str("BackTab"),
        KeyCode::Backspace => f.write_str("Backspace"),
        KeyCode::Esc => f.write_str("Esc"),
        KeyCode::Up => f.write_str("Up"),
        KeyCode::Down => f.write_str("Down"),
        KeyCode::Left => f.write_str("Left"),
        KeyCode::Right => f.write_str("Right"),
        KeyCode::Home => f.write_str("Home"),
        KeyCode::End => f.write_str("End"),
        KeyCode::PageUp => f.write_str("PageUp"),
        KeyCode::PageDown => f.write_str("PageDown"),
        KeyCode::Delete => f.write_str("Delete"),
        KeyCode::Insert => f.write_str("Insert"),
        KeyCode::F(n) => write!(f, "F{n}"),
    }
}

/// Error returned when two distinct actions are bound to the same [`KeyBinding`].
#[derive(Debug, Error)]
#[error("keybinding conflict on {binding}: already bound to {existing}, tried to bind {attempted}")]
pub struct KeyBindingConflict {
    /// The binding that collided.
    pub binding: KeyBinding,
    /// The action already bound to `binding`.
    pub existing: CompactString,
    /// The action that was attempted to be bound.
    pub attempted: CompactString,
}

/// A collection of [`KeyBinding`] → action-name mappings.
///
/// Action names are stored as [`CompactString`]s. Lookups return `&str` so
/// callers don't need to care about the internal representation.
pub struct KeyBindings {
    bindings: HashMap<KeyBinding, CompactString>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBindings {
    /// Create an empty set of key bindings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Bind `key` to `action`.
    ///
    /// If `key` is already bound, the previous binding is overwritten and
    /// returned. Use [`KeyBindings::try_bind`] if you'd prefer an error on
    /// conflict.
    pub fn bind(&mut self, key: KeyBinding, action: &str) -> Option<CompactString> {
        self.bindings.insert(key, CompactString::from(action))
    }

    /// Bind `key` to `action`, returning [`KeyBindingConflict`] if `key` is
    /// already bound to a *different* action.
    ///
    /// Re-binding the same key to the same action is a no-op and succeeds.
    pub fn try_bind(
        &mut self,
        key: KeyBinding,
        action: &str,
    ) -> std::result::Result<(), KeyBindingConflict> {
        if let Some(existing) = self.bindings.get(&key) {
            if existing.as_str() != action {
                return Err(KeyBindingConflict {
                    binding: key,
                    existing: existing.clone(),
                    attempted: CompactString::from(action),
                });
            }
            return Ok(());
        }
        self.bindings.insert(key, CompactString::from(action));
        Ok(())
    }

    /// Remove the binding for `key`, returning the previously-bound action if
    /// any.
    pub fn unbind(&mut self, key: &KeyBinding) -> Option<CompactString> {
        self.bindings.remove(key)
    }

    /// Look up the action name bound to `key`, if any.
    #[must_use]
    pub fn lookup(&self, key: &KeyBinding) -> Option<&str> {
        self.bindings.get(key).map(CompactString::as_str)
    }

    /// Convenience: look up the action bound to the key described by `event`.
    #[must_use]
    pub fn lookup_event(&self, event: &KeyEvent) -> Option<&str> {
        self.lookup(&KeyBinding::from(*event))
    }

    /// Return all bindings as `(key, action)` pairs, in arbitrary order.
    pub fn actions(&self) -> Vec<(&KeyBinding, &str)> {
        self.bindings.iter().map(|(k, v)| (k, v.as_str())).collect()
    }

    /// Merge another set of bindings into this one.
    ///
    /// On conflicts, `other` wins — its bindings overwrite the corresponding
    /// entries in `self`.
    pub fn merge(&mut self, other: KeyBindings) {
        for (k, v) in other.bindings {
            self.bindings.insert(k, v);
        }
    }

    /// The number of bindings currently registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether there are no bindings registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Standard emacs / readline editing bindings.
    ///
    /// These cover cursor movement, deletion, and the usual Ctrl-A / Ctrl-E
    /// line anchors.
    #[must_use]
    pub fn emacs() -> Self {
        let mut kb = Self::new();
        // Movement.
        kb.bind(ctrl('a'), "move_beginning_of_line");
        kb.bind(ctrl('e'), "move_end_of_line");
        kb.bind(ctrl('b'), "cursor_left");
        kb.bind(ctrl('f'), "cursor_right");
        kb.bind(ctrl('p'), "cursor_up");
        kb.bind(ctrl('n'), "cursor_down");
        // Deletion.
        kb.bind(ctrl('d'), "delete_forward");
        kb.bind(ctrl('h'), "delete_backward");
        kb.bind(ctrl('k'), "kill_to_end_of_line");
        kb.bind(ctrl('u'), "kill_to_beginning_of_line");
        kb.bind(ctrl('w'), "delete_word_backward");
        // Misc.
        kb.bind(ctrl('m'), "submit");
        kb.bind(ctrl('i'), "complete");
        kb.bind(ctrl('g'), "cancel");
        kb
    }

    /// A small sample of vim *normal-mode* bindings.
    ///
    /// This is intentionally not a complete vim implementation — it's a
    /// starting point you can extend with [`KeyBindings::bind`].
    #[must_use]
    pub fn vim_normal() -> Self {
        let mut kb = Self::new();
        // Movement.
        kb.bind(plain('h'), "cursor_left");
        kb.bind(plain('j'), "cursor_down");
        kb.bind(plain('k'), "cursor_up");
        kb.bind(plain('l'), "cursor_right");
        kb.bind(plain('w'), "forward_word");
        kb.bind(plain('b'), "backward_word");
        kb.bind(plain('0'), "line_start");
        kb.bind(plain('$'), "line_end");
        kb.bind(plain('G'), "buffer_end");
        // Mode switches.
        kb.bind(plain('i'), "enter_insert");
        kb.bind(plain('a'), "append");
        kb.bind(plain('o'), "open_below");
        kb.bind(plain('O'), "open_above");
        // Edits.
        kb.bind(plain('x'), "delete_char");
        kb.bind(plain('d'), "delete");
        kb.bind(plain('y'), "yank");
        kb.bind(plain('p'), "paste_after");
        kb.bind(plain('u'), "undo");
        kb.bind(plain('r'), "redo");
        // Ex commands.
        kb.bind(plain(':'), "enter_command");
        kb.bind(plain('/'), "search_forward");
        kb.bind(plain('n'), "search_next");
        kb.bind(plain('N'), "search_prev");
        kb
    }
}

/// Build a `KeyBinding` for a plain (no-modifier) character.
const fn plain(c: char) -> KeyBinding {
    KeyBinding {
        key: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
    }
}

/// Build a `KeyBinding` for a Ctrl-modified character.
const fn ctrl(c: char) -> KeyBinding {
    KeyBinding {
        key: KeyCode::Char(c),
        modifiers: KeyModifiers::CONTROL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb_char(c: char, mods: KeyModifiers) -> KeyBinding {
        KeyBinding::new(KeyCode::Char(c), mods)
    }

    #[test]
    fn display_formats_modifiers_in_stable_order() {
        let ctrl_x = kb_char('x', KeyModifiers::CONTROL);
        assert_eq!(ctrl_x.to_string(), "Ctrl+x");

        let shift_a = kb_char('A', KeyModifiers::SHIFT);
        assert_eq!(shift_a.to_string(), "Shift+A");

        let alt_tab = KeyBinding::new(KeyCode::Tab, KeyModifiers::ALT);
        assert_eq!(alt_tab.to_string(), "Alt+Tab");

        let combo = kb_char(
            'x',
            KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
        );
        assert_eq!(combo.to_string(), "Ctrl+Alt+Shift+x");

        let plain = kb_char('q', KeyModifiers::NONE);
        assert_eq!(plain.to_string(), "q");
    }

    #[test]
    fn from_key_event_preserves_code_and_modifiers() {
        let event = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        };
        let binding = KeyBinding::from(event);
        assert_eq!(
            binding,
            KeyBinding {
                key: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }
        );
    }

    #[test]
    fn bind_and_lookup_roundtrip() {
        let mut kb = KeyBindings::new();
        let key = kb_char('q', KeyModifiers::NONE);
        assert_eq!(kb.lookup(&key), None);

        kb.bind(key.clone(), "quit");
        assert_eq!(kb.lookup(&key), Some("quit"));

        // Overwriting returns the previous action.
        let prev = kb.bind(key.clone(), "force_quit");
        assert_eq!(prev.as_deref(), Some("quit"));
        assert_eq!(kb.lookup(&key), Some("force_quit"));
    }

    #[test]
    fn lookup_event_matches_event() {
        let mut kb = KeyBindings::new();
        kb.bind(ctrl('a'), "move_beginning_of_line");

        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
        };
        assert_eq!(kb.lookup_event(&event), Some("move_beginning_of_line"));

        let other = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(kb.lookup_event(&other), None);
    }

    #[test]
    fn unbind_removes_binding() {
        let mut kb = KeyBindings::new();
        let key = kb_char('x', KeyModifiers::NONE);
        kb.bind(key.clone(), "delete");
        assert_eq!(kb.unbind(&key).as_deref(), Some("delete"));
        assert_eq!(kb.lookup(&key), None);
        assert_eq!(kb.unbind(&key), None);
    }

    #[test]
    fn merge_other_wins_on_conflict() {
        let mut a = KeyBindings::new();
        a.bind(plain('q'), "quit_a");
        a.bind(plain('j'), "down_a");

        let mut b = KeyBindings::new();
        b.bind(plain('q'), "quit_b");

        a.merge(b);
        assert_eq!(a.lookup(&plain('q')), Some("quit_b"));
        assert_eq!(a.lookup(&plain('j')), Some("down_a"));
    }

    #[test]
    fn try_bind_conflicts_on_different_action() {
        let mut kb = KeyBindings::new();
        kb.bind(plain('q'), "quit");

        // Same action is fine.
        assert!(kb.try_bind(plain('q'), "quit").is_ok());

        // Different action errors.
        let err = kb.try_bind(plain('q'), "force_quit").unwrap_err();
        assert_eq!(err.binding, plain('q'));
        assert_eq!(err.existing.as_str(), "quit");
        assert_eq!(err.attempted.as_str(), "force_quit");

        // Unbound key succeeds.
        assert!(kb.try_bind(plain('w'), "up").is_ok());
        assert_eq!(kb.lookup(&plain('w')), Some("up"));
    }

    #[test]
    fn emacs_set_has_expected_entries() {
        let kb = KeyBindings::emacs();
        assert_eq!(kb.lookup(&ctrl('a')), Some("move_beginning_of_line"));
        assert_eq!(kb.lookup(&ctrl('e')), Some("move_end_of_line"));
        assert_eq!(kb.lookup(&ctrl('k')), Some("kill_to_end_of_line"));
        assert!(!kb.is_empty());
    }

    #[test]
    fn vim_normal_set_has_expected_entries() {
        let kb = KeyBindings::vim_normal();
        assert_eq!(kb.lookup(&plain('h')), Some("cursor_left"));
        assert_eq!(kb.lookup(&plain('j')), Some("cursor_down"));
        assert_eq!(kb.lookup(&plain('i')), Some("enter_insert"));
        assert!(kb.len() > 10);
    }

    #[test]
    fn actions_lists_all_bindings() {
        let mut kb = KeyBindings::new();
        kb.bind(plain('q'), "quit");
        kb.bind(ctrl('c'), "cancel");

        let mut actions: Vec<(&KeyBinding, &str)> = kb.actions();
        actions.sort_by_key(|(_, a)| *a);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].1, "cancel");
        assert_eq!(actions[1].1, "quit");
    }
}
