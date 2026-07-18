# Changelog

All notable changes to RusTUI are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Headless test backend (`HeadlessBackend`) for testing without a real terminal.
- `EventSender` wired into the `App` event loop for cross-task event injection.

## [0.1.0] - 2026-07-18

### Added

- **Core primitives:**
  - `Color` — RGBA with Porter-Duff alpha blending, 256-color palette, lerp.
  - `Style` — foreground/background color + attributes (bold, italic,
    underline, reverse, dim, blink, hidden, strike) with parent composition.
  - `Cell` — grapheme cluster + style + cached display width, wide-char aware.
  - `Buffer` — cell grid with `print`, `fill_rect`, `box_border`, `composite`
    (alpha compositing), `Rect` with `intersect`/`contains`.
  - `Span` / `Spans` — styled text data types.
  - Unicode helpers — grapheme iteration and display-width calculation
    (handles emoji, combining marks, zero-width joiners).

- **Backend abstraction:**
  - `Backend` trait — `enter`/`leave`, `size`, `poll`, `draw_cell`,
    `fill_rect`, synchronized-frame hooks.
  - `CrosstermBackend` — reference implementation using `crossterm` (raw mode,
    alternate screen, DECSET 2026 synced updates, full key/mouse/resize/
    focus/paste event translation). Gated behind `backend-crossterm` (default).

- **Layout:**
  - Flexbox layout via `taffy` (Yoga-compatible).
  - `FlexProps`, `FlexDirection`, `Align`, `Justify`, `Length`, `LayoutTree`.

- **Renderer:**
  - Double-buffered renderer with diff detection — only changed cells are
    re-emitted each frame.

- **App runtime:**
  - `App` / `AppBuilder` / `Context` — tokio-native async event loop with
    frame budgeting, `request_wakeup` for animations, `Context::state` for
    cross-frame state, `exit`.

- **Base widgets:**
  - `Flex` (column/row flex container), `Box` (decorator with bg/border),
    `Text` (wrapping), `List` (selectable), `Input` (cursor + placeholder),
    `Spinner` (braille/line/box/pulse animation styles).

- **Agent-harness widgets:**
  - `StreamingText` — LLM token streaming with blinking cursor.
  - `ToolCallPanel` + `ToolCall` + `ToolCallStatus` — tool call list with
    pending/running/success/failed/awaiting-approval/rejected states.
  - `DiffViewer` + `DiffHunk` + `DiffLine` — unified diff parser with
    color-coded add/remove/context lines and background tints.
  - `MessageList` + `Message` + `MessageRole` — scrollable chat transcript
    (user/assistant/system/tool).
  - `StatusLine` — status bar with left/right segments.

- **Examples:**
  - `hello.rs` — minimal centered text.
  - `agent_demo.rs` — full agent layout with transcript, streaming text,
    tool panel, diff viewer, and status line.

- **Project infrastructure:**
  - MIT license.
  - CI workflow (fmt + clippy + test + build on Linux/macOS/Windows,
    Rust stable + 1.80 MSRV).
  - Release workflow (crates.io publish + GitHub Release on tag).
  - Bug report and feature request issue templates.
  - Pull request template.
  - `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`.
  - `rustfmt.toml` with stable-only options.

[Unreleased]: https://github.com/Vaskrokodile/rusTUI/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Vaskrokodile/rusTUI/releases/tag/v0.1.0
