//! Focus management for keyboard-driven widget traversal.
//!
//! RusTUI widgets are stateless — the application owns the state and passes it
//! in each frame. Focus is no different: a [`FocusManager`] (or the lighter
//! [`FocusState`]) lives in your [`crate::app::Context`] state and records which
//! widget currently receives keyboard input.
//!
//! [`FocusManager`] is the full-featured manager: it owns the ordered list of
//! focusable widget IDs, the current index, and a wrap flag. [`FocusState`] is a
//! smaller, [`Clone`]able snapshot that is convenient to store directly in
//! `Context::state` and persist across frames.

/// A full focus manager: owns the ordered list of focusable widget IDs, the
/// current index, and a wrap flag.
///
/// Construct with [`FocusManager::new`] (empty) or [`FocusManager::with_ids`],
/// then [`FocusManager::add`] / [`FocusManager::remove`] widgets as they mount.
/// Use [`FocusManager::focus_next`] / [`FocusManager::focus_prev`] to cycle
/// focus in response to `Tab` / `Shift-Tab` key events.
#[derive(Clone, Debug)]
pub struct FocusManager {
    /// Ordered list of focusable widget IDs.
    pub focusable: Vec<compact_str::CompactString>,
    /// Index of the currently focused widget.
    pub current: usize,
    /// Whether focus cycling wraps around.
    pub wrap: bool,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create an empty focus manager (no focusable widgets, wrap disabled).
    #[must_use]
    pub fn new() -> Self {
        Self {
            focusable: Vec::new(),
            current: 0,
            wrap: false,
        }
    }

    /// Create a focus manager pre-populated with an ordered list of focusable
    /// IDs. The first ID is focused. Duplicates are ignored.
    #[must_use]
    pub fn with_ids(ids: impl IntoIterator<Item = impl Into<compact_str::CompactString>>) -> Self {
        let mut focusable: Vec<compact_str::CompactString> = Vec::new();
        for id in ids {
            let id = id.into();
            if !focusable.contains(&id) {
                focusable.push(id);
            }
        }
        Self {
            focusable,
            current: 0,
            wrap: false,
        }
    }

    /// Add a focusable widget ID. If the ID is already registered this is a
    /// no-op. Adding a widget never changes the current focus.
    pub fn add(&mut self, id: impl Into<compact_str::CompactString>) {
        let id = id.into();
        if !self.focusable.contains(&id) {
            self.focusable.push(id);
        }
    }

    /// Remove a focusable widget ID. If it was the focused widget, focus moves
    /// to the previous remaining widget (clamped to the new list bounds).
    pub fn remove(&mut self, id: &str) {
        if let Some(idx) = self.focusable.iter().position(|f| f == id) {
            self.focusable.remove(idx);
            // Clamp the current index to the new bounds. If we removed the
            // focused widget (or one before it), step back so we land on a
            // valid widget.
            if self.current >= self.focusable.len() {
                self.current = self.focusable.len().saturating_sub(1);
            } else if idx < self.current {
                self.current = self.current.saturating_sub(1);
            }
        }
    }

    /// Returns the ID of the currently focused widget, or `None` if there are
    /// no focusable widgets.
    #[must_use]
    pub fn current_id(&self) -> Option<&str> {
        self.focusable
            .get(self.current)
            .map(compact_str::CompactString::as_str)
    }

    /// Move focus to the next focusable widget.
    ///
    /// If [`FocusManager::wrap`] is enabled, focus cycles from the last widget
    /// back to the first; otherwise it clamps at the last widget. Does nothing
    /// if there are no focusable widgets.
    pub fn focus_next(&mut self) {
        if self.focusable.is_empty() {
            return;
        }
        if self.current + 1 < self.focusable.len() {
            self.current += 1;
        } else if self.wrap {
            self.current = 0;
        }
    }

    /// Move focus to the previous focusable widget.
    ///
    /// If [`FocusManager::wrap`] is enabled, focus cycles from the first widget
    /// back to the last; otherwise it clamps at the first widget. Does nothing
    /// if there are no focusable widgets.
    pub fn focus_prev(&mut self) {
        if self.focusable.is_empty() {
            return;
        }
        if self.current > 0 {
            self.current -= 1;
        } else if self.wrap {
            self.current = self.focusable.len() - 1;
        }
    }

    /// Focus a specific widget by ID. Returns `true` if the widget was found
    /// and focused, `false` otherwise (in which case focus is unchanged).
    pub fn focus(&mut self, id: &str) -> bool {
        if let Some(idx) = self.focusable.iter().position(|f| f == id) {
            self.current = idx;
            true
        } else {
            false
        }
    }

    /// Returns `true` if the given widget ID is currently focused.
    #[must_use]
    pub fn is_focused(&self, id: &str) -> bool {
        self.current_id().is_some_and(|current| current == id)
    }

    /// Enable or disable wrap-around when cycling past the first/last widget.
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }
}

/// A compact, [`Clone`]able snapshot of focus state suitable for storing in
/// [`crate::app::Context::state`].
///
/// Unlike [`FocusManager`], [`FocusState`] tracks the focused ID directly
/// (rather than an index) and exposes a smaller API oriented around
/// registering widgets as they mount and cycling focus. This is the type you
/// typically persist across frames.
#[derive(Clone, Debug, Default)]
pub struct FocusState {
    /// The currently focused widget ID, if any.
    pub current: Option<compact_str::CompactString>,
    /// The traversal order of focusable widget IDs.
    pub order: Vec<compact_str::CompactString>,
}

impl FocusState {
    /// Create an empty focus state (no registered widgets, no focus).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a widget ID in the focus traversal order.
    ///
    /// If the ID is already registered this is a no-op. The first widget
    /// registered becomes the focused widget automatically.
    pub fn register(&mut self, id: impl Into<compact_str::CompactString>) {
        let id = id.into();
        if !self.order.contains(&id) {
            self.order.push(id);
        }
        if self.current.is_none() {
            self.current = self.order.first().cloned();
        }
    }

    /// Returns the currently focused widget ID, or `None` if nothing is
    /// focused.
    #[must_use]
    pub fn focused(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// Set focus to a specific widget ID. Returns `true` if the widget is
    /// registered and was focused, `false` otherwise.
    pub fn set_focus(&mut self, id: &str) -> bool {
        if self.order.iter().any(|o| o == id) {
            self.current = Some(compact_str::CompactString::from(id));
            true
        } else {
            false
        }
    }

    /// Cycle focus to the next widget in traversal order, wrapping around.
    /// Does nothing if no widgets are registered.
    pub fn next(&mut self) {
        let Some(cur) = &self.current else {
            // Nothing focused yet: focus the first widget, if any.
            self.current = self.order.first().cloned();
            return;
        };
        let Some(idx) = self.order.iter().position(|o| o == cur) else {
            // Stale focus: fall back to the first widget.
            self.current = self.order.first().cloned();
            return;
        };
        if self.order.is_empty() {
            return;
        }
        let next = (idx + 1) % self.order.len();
        self.current = self.order.get(next).cloned();
    }

    /// Cycle focus to the previous widget in traversal order, wrapping around.
    /// Does nothing if no widgets are registered.
    pub fn prev(&mut self) {
        let Some(cur) = &self.current else {
            // Nothing focused yet: focus the last widget, if any.
            self.current = self.order.last().cloned();
            return;
        };
        let Some(idx) = self.order.iter().position(|o| o == cur) else {
            self.current = self.order.last().cloned();
            return;
        };
        if self.order.is_empty() {
            return;
        }
        let prev = if idx == 0 {
            self.order.len() - 1
        } else {
            idx - 1
        };
        self.current = self.order.get(prev).cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusManager, FocusState};

    #[test]
    fn add_and_remove_widgets() {
        let mut fm = FocusManager::new();
        fm.add("a");
        fm.add("b");
        fm.add("c");
        assert_eq!(fm.focusable.len(), 3);
        assert_eq!(fm.current_id(), Some("a"));

        // Duplicate adds are ignored.
        fm.add("b");
        assert_eq!(fm.focusable.len(), 3);

        // Removing a non-focused widget keeps focus stable.
        fm.remove("c");
        assert_eq!(fm.current_id(), Some("a"));
        assert_eq!(fm.focusable.len(), 2);
    }

    #[test]
    fn focus_next_without_wrap_clamps() {
        let mut fm = FocusManager::with_ids(["a", "b", "c"]);
        fm.focus_next();
        assert_eq!(fm.current_id(), Some("b"));
        fm.focus_next();
        assert_eq!(fm.current_id(), Some("c"));
        // No wrap: stays on the last widget.
        fm.focus_next();
        assert_eq!(fm.current_id(), Some("c"));
    }

    #[test]
    fn focus_prev_without_wrap_clamps() {
        let mut fm = FocusManager::with_ids(["a", "b", "c"]);
        // Move to the middle first.
        fm.focus_next();
        assert_eq!(fm.current_id(), Some("b"));
        fm.focus_prev();
        assert_eq!(fm.current_id(), Some("a"));
        // No wrap: stays on the first widget.
        fm.focus_prev();
        assert_eq!(fm.current_id(), Some("a"));
    }

    #[test]
    fn focus_next_prev_with_wrap_cycles() {
        let mut fm = FocusManager::with_ids(["a", "b", "c"]);
        fm.set_wrap(true);
        fm.focus_next();
        fm.focus_next();
        assert_eq!(fm.current_id(), Some("c"));
        // Wrap to the first.
        fm.focus_next();
        assert_eq!(fm.current_id(), Some("a"));
        // Wrap backwards to the last.
        fm.focus_prev();
        assert_eq!(fm.current_id(), Some("c"));
    }

    #[test]
    fn focus_specific_id_and_is_focused() {
        let mut fm = FocusManager::with_ids(["a", "b", "c"]);
        assert!(fm.focus("b"));
        assert_eq!(fm.current_id(), Some("b"));
        assert!(fm.is_focused("b"));
        assert!(!fm.is_focused("a"));
        // Focusing an unknown ID fails and leaves focus unchanged.
        assert!(!fm.focus("zzz"));
        assert_eq!(fm.current_id(), Some("b"));
    }

    #[test]
    fn empty_manager_has_no_focus() {
        let mut fm = FocusManager::new();
        assert_eq!(fm.current_id(), None);
        assert!(!fm.is_focused("a"));
        // Cycling an empty manager is a no-op.
        fm.focus_next();
        fm.focus_prev();
        assert_eq!(fm.current_id(), None);
    }

    #[test]
    fn focus_state_register_and_cycle() {
        let mut state = FocusState::new();
        assert_eq!(state.focused(), None);
        // First registered widget is auto-focused.
        state.register("a");
        state.register("b");
        state.register("c");
        assert_eq!(state.focused(), Some("a"));

        state.next();
        assert_eq!(state.focused(), Some("b"));
        state.next();
        assert_eq!(state.focused(), Some("c"));
        // Wraps back to the first.
        state.next();
        assert_eq!(state.focused(), Some("a"));
        // Prev wraps to the last.
        state.prev();
        assert_eq!(state.focused(), Some("c"));

        // set_focus to a registered widget succeeds; to an unknown one fails.
        assert!(state.set_focus("a"));
        assert_eq!(state.focused(), Some("a"));
        assert!(!state.set_focus("zzz"));
        assert_eq!(state.focused(), Some("a"));
    }
}
