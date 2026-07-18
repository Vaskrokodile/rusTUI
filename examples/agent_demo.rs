//! A small agent-harness demo: streaming text + tool-call panel + status line.

use std::time::Duration;

use rustui::{
    App, Color, DiffViewer, Flex, Length, Message, MessageList, MessageRole, Spans, Spinner,
    StatusLine, StreamingText, Text, ToolCall, ToolCallPanel, ToolCallStatus,
};

fn main() -> rustui::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let mut app = App::default();
    runtime.block_on(app.run(|ctx| {
        // Exit on `q` or Ctrl-C.
        if let Some(ev) = &ctx.event {
            if let Some(k) = ev.as_key() {
                if k.is(rustui::event::KeyCode::Esc) || k.is(rustui::event::KeyCode::Char('q')) {
                    ctx.exit();
                }
            }
        }
        // Animate the spinner.
        ctx.request_wakeup(Duration::from_millis(80));

        let transcript = MessageList::from_messages(vec![
            Message::new(MessageRole::User, "refactor my buffer module to use smallvec"),
            Message::new(
                MessageRole::Assistant,
                Spans::plain("I'll start by reading the current buffer module, then propose a diff."),
            ),
        ])
        .grow(1.0);

        let streaming = StreamingText::new(Spans::plain("Looking at `src/buffer.rs`, I see the cells vector is heap-allocated per frame. Switching to SmallVec would avoid allocations for small frames…"))
            .streaming(true)
            .grow(1.0);

        let tools = ToolCallPanel::from_calls(vec![
            ToolCall::new("read_file", "src/buffer.rs", ToolCallStatus::Success)
                .result("184 lines, 3 impls"),
            ToolCall::new("bash", "cargo bench --bench buffer", ToolCallStatus::Running),
            ToolCall::new("edit_file", "src/buffer.rs", ToolCallStatus::Pending),
        ])
        .title("tool calls")
        .grow(0.0);

        let diff = DiffViewer::parse(
            "--- a/src/buffer.rs\n+++ b/src/buffer.rs\n@@ -10,5 +10,8 @@\n pub struct Buffer {\n     pub width: u16,\n     pub height: u16,\n-    cells: Vec<Cell>,\n+    cells: SmallVec<[Cell; 64]>,\n }\n+impl Buffer {\n+    // ...\n+}\n",
        )
        .grow(1.0);

        let status = StatusLine::new(
            "gpt-5 · 12.3k tokens · $0.04",
            "NORMAL · esc to quit",
        );

        // Layout: left column = transcript + streaming + input; right column =
        // tools + diff. Status line spans the bottom.
        Flex::column()
            .child(
                Flex::row()
                    .grow(1.0)
                    .child(
                        Flex::column()
                            .grow(1.0)
                            .padding_all(1.0)
                            .child(transcript)
                            .child(streaming)
                            .child(
                                Flex::row()
                                    .child(Spinner::new().color(Color::CYAN))
                                    .child(Text::new("> _").fg(Color::palette256(8)).grow(1.0)),
                            ),
                    )
                    .child(
                        Flex::column()
                            .width(Length::Fixed(40.0))
                            .padding_all(1.0)
                            .child(tools)
                            .child(diff),
                    ),
            )
            .child(status)
    }))?;
    Ok(())
}
