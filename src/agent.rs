//! Widgets purpose-built for agentic coding harnesses.
//!
//! These cover the patterns that show up in essentially every agent harness:
//! streaming LLM output, tool-call panels, diff viewers, message logs, and
//! status lines.

mod diff_viewer;
mod message_list;
mod status_line;
mod streaming_text;
mod tool_call_panel;

pub use diff_viewer::{DiffHunk, DiffLine, DiffLineKind, DiffViewer};
pub use message_list::{Message, MessageList, MessageRole};
pub use status_line::StatusLine;
pub use streaming_text::StreamingText;
pub use tool_call_panel::{ToolCall, ToolCallPanel, ToolCallStatus};
