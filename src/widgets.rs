//! Base widgets: the building blocks every TUI uses.
//!
//! These are deliberately small and composable. Agent-specific widgets live
//! in [`crate::agent`].

pub mod base;
pub(crate) mod box_widget;
pub(crate) mod input;
pub(crate) mod list;
pub(crate) mod spinner;
pub(crate) mod text_widget;

pub use base::{PaintCtx, Widget, WidgetId, WidgetTree};
pub use box_widget::{Box, Flex};
pub use input::Input;
pub use list::{List, ListItem};
pub use spinner::{Spinner, SpinnerStyle};
pub use text_widget::Text;
