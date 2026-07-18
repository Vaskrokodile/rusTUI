# Contributing to RusTUI

Thanks for your interest in contributing! This document covers the basics.

## Getting started

1. Fork and clone the repo.
2. Install Rust 1.80+ (`rustup`).
3. Build: `cargo build`
4. Test: `cargo test`
5. Run the examples: `cargo run --example hello` or `cargo run --example agent_demo`

## Development workflow

Before opening a PR, make sure all of these pass:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
```

CI runs all of the above on Linux, macOS, and Windows, on both `stable` and
`1.80` (our MSRV).

## Architecture overview

```
src/
├── lib.rs              — crate root, re-exports
├── app.rs              — App, AppBuilder, Context (the event loop)
├── backend.rs          — Backend trait
├── backend/
│   └── crossterm_impl.rs — reference crossterm backend
├── buffer.rs           — Buffer, Rect (cell grid + compositing)
├── cell.rs             — Cell (grapheme + style + width)
├── color.rs            — Color (RGBA + alpha blending)
├── error.rs            — Error / Result
├── event.rs            — Event, KeyEvent, MouseEvent
├── input.rs            — input parsing helpers
├── layout.rs           — FlexProps, LayoutTree (taffy-based flexbox)
├── renderer.rs         — double-buffered renderer with diff
├── style.rs            — Style, Attr
├── text.rs             — Span, Spans (styled text data)
├── unicode.rs          — grapheme + display-width helpers
├── widgets.rs          — module root + re-exports
├── widgets/
│   ├── base.rs         — Widget trait, WidgetTree walker
│   ├── box_widget.rs   — Flex, Box
│   ├── input.rs        — Input
│   ├── list.rs         — List, ListItem
│   ├── spinner.rs      — Spinner, SpinnerStyle
│   └── text_widget.rs  — Text
└── agent.rs            — agent-harness widgets
    ├── diff_viewer.rs
    ├── message_list.rs
    ├── status_line.rs
    ├── streaming_text.rs
    └── tool_call_panel.rs
```

### Design principles

- **Immediate-mode widgets.** Widgets are built fresh each frame and dropped
  after painting. State that needs to persist across frames lives in
  `Context::state`, not in widget structs.
- **Backend-agnostic.** Everything goes through the `Backend` trait. Don't
  call `crossterm` directly outside `src/backend/crossterm_impl.rs`.
- **No `unsafe`.** The crate is `#![forbid(unsafe_code)]`. Keep it that way.
- **Zero opinions about your app structure.** RusTUI provides the rendering,
  layout, and event loop; you decide how to structure your agent harness.

### Adding a new widget

1. If it's a general-purpose widget, add it under `src/widgets/`. If it's
   agent-specific, add it under `src/agent/`.
2. Implement the `Widget` trait (`layout`, `paint`, and `take_children` if
   it's a container).
3. Re-export it from `src/widgets.rs` or `src/agent.rs`, and from `src/lib.rs`.
4. Add a test that exercises the widget's paint output into a `Buffer`.
5. Add a `CHANGELOG.md` entry.

### Adding a new backend

1. Implement the `Backend` trait in a new module under `src/backend/`.
2. Gate it behind a `backend-<name>` feature in `Cargo.toml`.
3. Add it to the `default_backend()` function (conditionally).
4. Test it with the headless test backend pattern (see
   `src/backend/headless.rs`).

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) loosely:

```
feat: add Markdown widget
fix: handle zero-width graphemes in Text wrapping
docs: update README quick start
refactor: split buffer compositing into its own method
ci: add Windows to test matrix
```

## Releases

- Versions follow [SemVer](https://semver.org/).
- The `CHANGELOG.md` tracks changes under `## [Unreleased]`.
- On release, the maintainer moves unreleased entries to a new version
  section, tags `vX.Y.Z`, and the release workflow publishes to crates.io.

## Code of conduct

Be kind. Disagree respectfully. Assume good intent.
