# RusTUI

![CI](https://github.com/Vaskrokodile/rusTUI/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)

An opinionated TUI toolkit for building **agentic coding harnesses** in Rust.

RusTUI sits one layer above a raw rendering engine: it gives you a
backend-agnostic renderer, a Flexbox layout system, a tokio-native event loop,
and a set of widgets purpose-built for the patterns that show up in agent
harnesses — streaming LLM output, tool-call panels, diff viewers, message
logs, status lines, and spinners.

## Design

- **Agent-harness toolkit.** Not a generic widget library — the built-in
  widgets cover the patterns every agent harness needs.
- **Backend-agnostic.** Everything goes through a `Backend` trait. A
  `crossterm` reference backend ships behind `backend-crossterm`; plug in
  `termion` or your own.
- **Tokio-native.** The event loop is async; streaming LLM tokens and tool
  output compose naturally with `tokio::select!` and friends.
- **Flexbox layout** via [`taffy`](https://crates.io/crates/taffy)
  (Yoga-compatible).
- **Double-buffered renderer** with diff detection — only changed cells are
  re-emitted each frame.
- **Immediate-mode widgets.** Build a fresh widget tree each frame; persistent
  state lives in `Context::state`, not in widgets.
- **Unicode-aware.** Grapheme cluster segmentation and display width via
  `unicode-segmentation` and `unicode-width`.
- **ANSI color parsing.** Built-in ANSI escape sequence parser for rendering
  colored output from external processes.

## Architecture

```
your agent harness
        │
agent widgets: StreamingText, ToolCallPanel, DiffViewer, MessageList, StatusLine
        │
content widgets: Markdown, Table, Tree, Tabs, CommandPalette, Modal, Dialog
        │
base widgets: Text, Flex, Box, List, Input, TextArea, Block, Scrollable, Gauge, Spinner, Toast
        │
layout (taffy)  │  renderer (double-buffered, diffed)
        │
buffer / cell / style / color / ansi  (core primitives)
        │
Backend trait  ←  crossterm | termion | custom
```

## Widgets

### Base Widgets
- **Block** — bordered container with titles, multiple border styles
- **Text** — styled text with foreground/background colors
- **Flex** — horizontal/vertical flexbox layout
- **Box** — simple container
- **List** — selectable list of items
- **Input** — single-line text input with cursor
- **TextArea** — multi-line text input with cursor movement
- **Scrollable** — scrollable container with scrollbar
- **Gauge** — progress bar (horizontal and line gauge)
- **Spinner** — animated loading indicator
- **Paragraph** — multi-line text with wrapping and alignment

### Content Widgets
- **Markdown** — renders markdown with headings, bold, italic, code blocks, lists, blockquotes, links
- **Table** — tabular data with column widths, headers, row selection
- **Tree** — hierarchical collapsible tree view
- **Tabs** — tabbed panel switching
- **CommandPalette** — fuzzy-searchable command menu
- **Modal** — overlay dialog container
- **Dialog** — simple yes/no/confirm dialog
- **Toast** — transient notification overlays (info/success/warning/error)

### Agent Widgets
- **StreamingText** — LLM token streaming display
- **ToolCallPanel** — tool call visualization with status
- **DiffViewer** — unified diff rendering
- **MessageList** — chat message history
- **StatusLine** — bottom status bar

## Systems

- **Theme** — color presets and syntax token colors (DARK, LIGHT, DRACULA, etc.)
- **Keybindings** — configurable key binding maps with emacs/vim presets
- **Focus** — focus management for widget traversal
- **Syntax highlighting** — `syntect` backend (feature-gated) with simple fallback
- **ANSI parsing** — parse and render ANSI color codes
- **Word wrapping** — Unicode-aware word-aware text wrapping
- **Snapshot testing** — buffer comparison utilities for testing

## Quick start

```toml
[dependencies]
rustui = { version = "0.1" }
tokio = { version = "1", features = ["full"] }
```

```rust
use rustui::{App, Flex, Text, Color};

fn main() -> rustui::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all().build()?;
    let mut app = App::default();
    rt.block_on(app.run(|_ctx| {
        Flex::column()
            .align(rustui::Align::Center)
            .justify(rustui::Justify::Center)
            .child(Text::new("hello from RusTUI").fg(Color::CYAN))
    }))
}
```

## Examples

- `cargo run --example hello` — minimal hello world
- `cargo run --example agent_demo` — streaming text + tool calls + diff viewer
- `cargo run --example agent_harness` — comprehensive demo with all widgets:
  message list, streaming, markdown, tabs, tree, command palette, modal, toast,
  gauge, and focus management

## Feature flags

- `backend-crossterm` (default): reference backend using `crossterm`.
- `agent-full`: pulls in markdown + syntax highlighting for rich agent output.
- `syntax-highlight`: `syntect`-backed syntax highlighting for code/diff blocks.

## Testing

```bash
cargo test              # run all tests
cargo clippy -- -D warnings  # lint
cargo bench             # run benchmarks (criterion)
```

The library includes snapshot testing utilities (`assert_buffer`,
`render_buffer`) for verifying widget rendering output.

## License

MIT.
