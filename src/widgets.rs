//! Base widgets: the building blocks every TUI uses.
//!
//! These are deliberately small and composable. Agent-specific widgets live
//! in [`crate::agent`].

pub mod base;
pub mod block;
pub(crate) mod box_widget;
pub(crate) mod gauge;
pub(crate) mod input;
pub(crate) mod list;
pub(crate) mod paragraph;
pub(crate) mod scroll;
pub(crate) mod spinner;
pub(crate) mod tabs;
pub(crate) mod text_widget;

pub use base::{PaintCtx, Widget, WidgetId, WidgetTree};
pub use block::{Block, BorderType};
pub use box_widget::{Box, Flex};
pub use gauge::{Gauge, LineGauge};
pub use input::Input;
pub use list::{List, ListItem};
pub use paragraph::Paragraph;
pub use scroll::{scrollbar_thumb, Scrollable};
pub use spinner::{Spinner, SpinnerStyle};
pub use tabs::Tabs;
pub use text_widget::Text;
