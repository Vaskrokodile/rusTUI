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

## Architecture

```
your agent harness
        │
agent widgets: StreamingText, ToolCallPanel, DiffViewer, MessageList, StatusLine
        │
base widgets: Text, Flex, Box, List, Input, Spinner
        │
layout (taffy)  │  renderer (double-buffered, diffed)
        │
buffer / cell / style / color  (core primitives)
        │
Backend trait  ←  crossterm | termion | custom
```

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

See `examples/agent_demo.rs` for a fuller example with streaming text, a
tool-call panel, a diff viewer, and a status line.

## Feature flags

- `backend-crossterm` (default): reference backend using `crossterm`.
- `agent-full`: pulls in markdown + syntax highlighting for rich agent output.
- `syntax-highlight`: `syntect`-backed syntax highlighting for code/diff blocks.

## License

MIT.
