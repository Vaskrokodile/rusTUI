//! A comprehensive agent harness demo showcasing all RusTUI widgets.
//!
//! This example simulates a full agentic coding assistant UI with:
//! - Message history with scrolling
//! - Streaming text output
//! - Tool call visualization
//! - Diff viewer
//! - Status bar with model/token info
//! - Command palette (Ctrl+P)
//! - Toast notifications
//! - Markdown rendering
//! - Tab-based panel switching
//!
//! Run with: `cargo run --example agent_harness`

use std::time::Duration;

use rustui::{
    App, Block, BorderType, Color, Command, CommandPalette, DiffViewer, Flex, FocusManager, Gauge,
    Input, KeyBindings, Length, Markdown, Message, MessageList, MessageRole, Modal, Paragraph,
    Scrollable, Spans, Spinner, StatusLine, StreamingText, Style, Tabs, Toast, ToolCall,
    ToolCallPanel, ToolCallStatus, Tree, TreeNode,
};

fn main() -> rustui::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let mut app = App::default();
    let sender = app.event_sender();

    // Simulate streaming tokens from an LLM.
    runtime.spawn(async move {
        let tokens = [
            "I'll ",
            "analyze ",
            "the ",
            "codebase ",
            "and ",
            "propose ",
            "changes.\n\n",
            "## Plan\n\n",
            "1. ",
            "Read ",
            "the ",
            "current ",
            "implementation\n",
            "2. ",
            "Identify ",
            "bottlenecks\n",
            "3. ",
            "Apply ",
            "optimizations\n",
        ];
        for _token in tokens {
            sender.send(rustui::Event::Wakeup);
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    let mut show_palette = false;
    let mut show_modal = false;
    let mut active_tab = 0usize;
    let mut scroll = 0u16;
    let mut focus = FocusManager::with_ids(["input", "messages", "tools", "diff"]);

    // Set up keybindings (used for documentation purposes; in a real app
    // you'd look up actions via keybindings.lookup_event()).
    let _keybindings = KeyBindings::emacs();

    runtime.block_on(app.run(move |ctx| {
        // Handle events.
        let mut should_exit = false;
        if let Some(ev) = &ctx.event {
            if let Some(k) = ev.as_key() {
                // Quit.
                if k.is(rustui::event::KeyCode::Esc) || k.is(rustui::event::KeyCode::Char('q')) {
                    should_exit = true;
                }
                // Toggle command palette.
                if k.is(rustui::event::KeyCode::Char('p'))
                    && k.modifiers.contains(rustui::event::KeyModifiers::CONTROL)
                {
                    show_palette = !show_palette;
                }
                // Toggle modal.
                if k.is(rustui::event::KeyCode::Char('m'))
                    && k.modifiers.contains(rustui::event::KeyModifiers::CONTROL)
                {
                    show_modal = !show_modal;
                }
                // Tab switching.
                if k.is(rustui::event::KeyCode::Tab) {
                    active_tab = (active_tab + 1) % 3;
                }
                // Scroll down.
                if k.is(rustui::event::KeyCode::Down) {
                    scroll = scroll.saturating_add(1);
                }
                // Scroll up.
                if k.is(rustui::event::KeyCode::Up) {
                    scroll = scroll.saturating_sub(1);
                }
                // Focus cycling.
                if k.is(rustui::event::KeyCode::Char('j'))
                    && k.modifiers.contains(rustui::event::KeyModifiers::CONTROL)
                {
                    focus.focus_next();
                }
            }
        }
        if should_exit {
            ctx.exit();
        }

        // Animate.
        ctx.request_wakeup(Duration::from_millis(80));

        // Persist state.
        ctx.set_state("scroll", scroll);
        ctx.set_state("active_tab", active_tab);
        ctx.set_state("show_palette", show_palette);
        ctx.set_state("show_modal", show_modal);

        // Build the message list.
        let messages = MessageList::from_messages(vec![
            Message::new(
                MessageRole::User,
                "Help me optimize the buffer module for large terminals",
            ),
            Message::new(
                MessageRole::Assistant,
                Spans::plain("I'll analyze the codebase and propose changes."),
            ),
            Message::new(
                MessageRole::User,
                "Can you also add snapshot testing support?",
            ),
        ])
        .grow(1.0);

        // Streaming text (simulated LLM output).
        let streaming = StreamingText::new(Spans::plain(
            "Looking at `src/buffer.rs`, I see the cells vector is heap-allocated per frame. \
             Switching to SmallVec would avoid allocations for small frames…",
        ))
        .streaming(true)
        .grow(0.5);

        // Markdown content.
        let markdown = Markdown::new(
            "## Optimization Plan\n\n\
             1. **Read** the current implementation\n\
             2. **Identify** bottlenecks in the hot path\n\
             3. **Apply** SmallVec for small buffers\n\
             4. **Benchmark** before and after\n\n\
             ```rust\n\
             use smallvec::SmallVec;\n\
             \n\
             pub struct Buffer {\n\
                 cells: SmallVec<[Cell; 64]>,\n\
             }\n\
             ```\n",
        )
        .grow(1.0);

        // Tool calls.
        let tools = ToolCallPanel::from_calls(vec![
            ToolCall::new("read_file", "src/buffer.rs", ToolCallStatus::Success)
                .result("184 lines, 3 impls"),
            ToolCall::new(
                "bash",
                "cargo bench --bench buffer",
                ToolCallStatus::Running,
            ),
            ToolCall::new("edit_file", "src/buffer.rs", ToolCallStatus::Pending),
        ])
        .title("tool calls")
        .grow(0.0);

        // Diff viewer.
        let diff = DiffViewer::parse(
            "--- a/src/buffer.rs\n+++ b/src/buffer.rs\n@@ -10,5 +10,8 @@\n \
             pub struct Buffer {\n     pub width: u16,\n     pub height: u16,\n\
             -    cells: Vec<Cell>,\n+    cells: SmallVec<[Cell; 64]>,\n }\n",
        )
        .grow(1.0);

        // File tree.
        let tree = Tree::new()
            .root(
                TreeNode::new("src")
                    .expanded(true)
                    .add_child(TreeNode::leaf("buffer.rs").icon("📄"))
                    .add_child(TreeNode::leaf("cell.rs").icon("📄"))
                    .add_child(
                        TreeNode::new("widgets")
                            .expanded(true)
                            .add_child(TreeNode::leaf("markdown.rs").icon("📄"))
                            .add_child(TreeNode::leaf("table.rs").icon("📄")),
                    ),
            )
            .root(TreeNode::leaf("Cargo.toml").icon("📦"))
            .grow(1.0);

        // Tabs for the right panel.
        let tabs = Tabs::new(vec!["Tools", "Diff", "Files"])
            .active(active_tab)
            .grow(0.0);

        // Right panel content based on active tab.
        let right_content = match active_tab {
            0 => Flex::column().grow(1.0).child(tools),
            1 => Flex::column().grow(1.0).child(diff),
            2 => Flex::column().grow(1.0).child(tree),
            _ => Flex::column().grow(1.0),
        };

        // Status bar.
        let status = StatusLine::new(
            "gpt-5 · 12.3k tokens · $0.04",
            "NORMAL · Ctrl+P palette · Ctrl+M modal · Tab switch · q quit",
        );

        // Progress gauge.
        let gauge = Gauge::new(0.65)
            .label("65% · context window")
            .grow(0.0)
            .height(1);

        // Input area.
        let input = Input::new()
            .placeholder("Type a message... (Ctrl+J for newline)")
            .grow(0.0);

        // Main layout.
        let mut root = Flex::column()
            .child(
                Flex::row()
                    .grow(1.0)
                    .child(
                        Flex::column()
                            .grow(1.0)
                            .padding_all(1.0)
                            .child(
                                Block::new()
                                    .title("Messages")
                                    .border(Style::empty().fg(Color::rgb(80, 80, 80)))
                                    .border_type(BorderType::Rounded)
                                    .child(Scrollable::new("msg_scroll").grow(1.0).child(messages))
                                    .grow(1.0),
                            )
                            .child(
                                Block::new()
                                    .title("Streaming")
                                    .border(Style::empty().fg(Color::rgb(80, 80, 80)))
                                    .border_type(BorderType::Rounded)
                                    .child(streaming)
                                    .grow(0.5),
                            )
                            .child(
                                Block::new()
                                    .title("Analysis")
                                    .border(Style::empty().fg(Color::rgb(80, 80, 80)))
                                    .border_type(BorderType::Rounded)
                                    .child(markdown)
                                    .grow(1.0),
                            )
                            .child(gauge)
                            .child(
                                Flex::row()
                                    .child(Spinner::new().color(Color::CYAN))
                                    .child(input),
                            ),
                    )
                    .child(
                        Flex::column()
                            .width(Length::Fixed(45.0))
                            .padding_all(1.0)
                            .child(tabs)
                            .child(right_content),
                    ),
            )
            .child(status);

        // Overlay: command palette.
        if show_palette {
            let palette = CommandPalette::new(vec![
                Command::new("Save File").shortcut("Ctrl+S"),
                Command::new("Open File").shortcut("Ctrl+O"),
                Command::new("Run Tests").shortcut("Ctrl+T"),
                Command::new("Toggle Modal").shortcut("Ctrl+M"),
                Command::new("Quit").shortcut("q"),
            ]);
            root = Flex::column().child(palette).child(root);
        }

        // Overlay: modal dialog.
        if show_modal {
            let modal = Modal::new("About")
                .child(
                    Paragraph::new(
                        "RusTUI Agent Harness Demo\n\n\
                         A comprehensive TUI for agentic coding assistants.\n\
                         Built with the RusTUI toolkit.\n\n\
                         Press Ctrl+M to close this dialog.",
                    )
                    .grow(1.0),
                )
                .width_pct(0.5)
                .height_pct(0.3);
            root = Flex::column().child(modal).child(root);
        }

        // Toast notification (ephemeral).
        if ctx.elapsed.as_secs() < 3 {
            let toast = Toast::info("Welcome to RusTUI Agent Harness!")
                .position(rustui::ToastPosition::BottomRight);
            root = Flex::column().child(toast).child(root);
        }

        root
    }))?;
    Ok(())
}
