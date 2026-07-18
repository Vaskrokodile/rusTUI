//! Input and lifecycle events.

/// Keyboard modifier flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeyModifiers(pub u8);

impl KeyModifiers {
    /// No modifiers.
    pub const NONE: Self = Self(0);
    /// Shift held.
    pub const SHIFT: Self = Self(1 << 0);
    /// Control held.
    pub const CONTROL: Self = Self(1 << 1);
    /// Alt / Option held.
    pub const ALT: Self = Self(1 << 2);

    /// Whether `self` contains all bits in `other`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether no modifiers are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A keyboard key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// A printable character.
    Char(char),
    /// Enter / Return.
    Enter,
    /// Tab.
    Tab,
    /// Shift-Tab.
    BackTab,
    /// Backspace.
    Backspace,
    /// Escape.
    Esc,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Delete (forward).
    Delete,
    /// Insert.
    Insert,
    /// A function key (F1..F12).
    F(u8),
}

/// A key event: key code + active modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    /// Which key was pressed.
    pub code: KeyCode,
    /// Active modifiers.
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Whether this is a plain (no-modifier) press of `code`.
    pub fn is(self, code: KeyCode) -> bool {
        self.modifiers.is_empty() && self.code == code
    }
}

/// Mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle / wheel button.
    Middle,
}

/// Kind of mouse event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    /// Button pressed.
    Down(MouseButton),
    /// Button released.
    Up(MouseButton),
    /// Button held and moved.
    Drag(MouseButton),
    /// Pointer moved with no button held.
    Moved,
    /// Wheel scrolled up.
    ScrollUp,
    /// Wheel scrolled down.
    ScrollDown,
}

/// A mouse event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MouseEvent {
    /// What happened.
    pub kind: MouseEventKind,
    /// Column (0-based).
    pub x: u16,
    /// Row (0-based).
    pub y: u16,
    /// Active modifiers.
    pub modifiers: KeyModifiers,
}

/// An input or lifecycle event.
#[derive(Debug)]
pub enum Event {
    /// A key was pressed (or released, on terminals that report it).
    Key(KeyEvent),
    /// A mouse event.
    Mouse(MouseEvent),
    /// The terminal was resized to `(width, height)`.
    Resize(u16, u16),
    /// The terminal gained focus.
    FocusGained,
    /// The terminal lost focus.
    FocusLost,
    /// A bracketed-paste payload.
    Paste(String),
    /// A wake-up requested via [`crate::app::Context::request_wakeup`].
    Wakeup,
    /// A custom user event (e.g. an LLM token chunk).
    User(Box<dyn std::any::Any + Send>),
}

impl Event {
    /// If this is a key event, return it.
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(k) => Some(k),
            _ => None,
        }
    }
}
