//! # RusTUI
//!
//! An opinionated TUI toolkit for building **agentic coding harnesses** in Rust.
//!
//! RusTUI sits one layer above a raw rendering engine: it gives you a
//! backend-agnostic renderer, a Flexbox layout system, a tokio-native event
//! loop, and a set of widgets purpose-built for the patterns that show up in
//! agent harnesses — streaming LLM output, tool-call panels, diff viewers,
//! message logs, status lines, and spinners.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                    your agent harness                     │
//! └──────────────────────────────────────────────────────────┘
//!                            │
//! ┌──────────────────────────────────────────────────────────┐
//! │  agent widgets: StreamingText, ToolCallPanel, DiffViewer │
//! │  MessageList, StatusLine, Spinner, ...                   │
//! └──────────────────────────────────────────────────────────┘
//!                            │
//! ┌──────────────────────────────────────────────────────────┐
//! │  base widgets: Text, Box, List, Input, Flex, ...         │
//! └──────────────────────────────────────────────────────────┘
//!                            │
//! ┌──────────────────────┐   ┌──────────────────────────────┐
//! │  layout (taffy)      │   │  renderer (double-buffered)  │
//! └──────────────────────┘   └──────────────────────────────┘
//!                            │
//! ┌──────────────────────────────────────────────────────────┐
//! │  buffer / cell / style / color  (core primitives)        │
//! └──────────────────────────────────────────────────────────┘
//!                            │
//! ┌──────────────────────────────────────────────────────────┐
//! │  Backend trait  ←  crossterm | termion | custom           │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Feature flags
//!
//! - `backend-crossterm` (default): reference backend using `crossterm`.
//! - `agent-full`: pulls in markdown + syntax highlighting for rich agent output.
//!
//! ## Hello world
//!
//! ```no_run
//! use rustui::{App, Flex, Text, Color};
//!
//! # fn main() -> rustui::Result<()> {
//! let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
//! let mut app = App::default();
//! rt.block_on(app.run(|_ctx| {
//!     Flex::column().child(Text::new("hello from RusTUI").fg(Color::CYAN))
//! }))?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
// Pedantic lints that are too noisy for this codebase. Re-enable individually
// if you want to tighten things up in a specific module.
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::needless_doctest_main,
    clippy::redundant_closure,
    clippy::collapsible_if,
    clippy::if_same_then_else,
    clippy::match_same_arms,
    clippy::single_match_else,
    clippy::let_underscore_untyped
)]

pub mod ansi;
pub mod backend;
pub mod buffer;
pub mod cell;
pub mod color;
pub mod error;
pub mod event;
pub mod focus;
pub mod input;
pub mod keybindings;
pub mod layout;
pub mod renderer;
pub mod style;
pub mod syntax;
pub mod text;
pub mod theme;
pub mod unicode;
pub mod wrap;

pub mod app;
pub mod widgets;

/// Widgets purpose-built for agentic coding harnesses.
pub mod agent;

pub use app::{App, AppBuilder, Context, EventSender};
pub use backend::{Backend, HeadlessBackend};
pub use buffer::{Buffer, Rect};
pub use cell::Cell;
pub use color::Color;
pub use error::{Error, Result};
pub use event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
pub use focus::{FocusManager, FocusState};
pub use keybindings::{KeyBinding, KeyBindings};
pub use layout::{Align, FlexDirection, FlexProps, Justify, LayoutNode, LayoutTree, Length};
pub use renderer::Renderer;
pub use style::Style;
pub use text::{Span, Spans};
pub use theme::{SyntaxTokenType, Theme};
pub use unicode::{grapheme_width, graphemes};

pub use widgets::{
    scrollbar_thumb, Block, BorderType, Box as WidgetBox, Command, CommandPalette, CursorPos,
    Dialog, Flex, Gauge, Input, LineGauge, List, ListItem, Markdown, Modal, Paragraph, Scrollable,
    Spinner, SpinnerStyle, Table, Tabs, Text, TextArea, Toast, ToastLevel, ToastPosition,
    ToastStack, Tree, TreeNode,
};

pub use agent::{
    DiffHunk, DiffLine, DiffLineKind, DiffViewer, Message, MessageList, MessageRole, StatusLine,
    StreamingText, ToolCall, ToolCallPanel, ToolCallStatus,
};
