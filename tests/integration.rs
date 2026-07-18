//! Integration tests using the headless backend.

use std::time::Duration;

use rustui::{
    Align, Color, Event, EventSender, Flex, Justify, KeyCode, KeyEvent, KeyModifiers, Spans, Text,
};

#[test]
fn headless_backend_renders_text() {
    use rustui::buffer::Buffer;
    use rustui::style::Style;

    let mut backend = rustui::HeadlessBackend::new(20, 3);
    let mut buf = Buffer::empty(20, 3);
    buf.print(0, 1, "hello", Style::empty().fg(Color::GREEN));

    // Simulate the renderer drawing changed cells via the Backend trait.
    use rustui::Backend;
    for y in 0..3 {
        for x in 0..20 {
            if let Some(cell) = buf.cell(x, y) {
                if !cell.is_blank() {
                    backend.draw_cell(x, y, cell).unwrap();
                }
            }
        }
    }
    let rows = backend.rows();
    assert_eq!(rows[1], "hello               ");
}

#[test]
fn event_sender_delivers_events_in_order() {
    let sender = EventSender::new();
    sender.send(Event::Wakeup);
    sender.send(Event::FocusGained);
    assert!(matches!(sender.try_recv(), Some(Event::Wakeup)));
    assert!(matches!(sender.try_recv(), Some(Event::FocusGained)));
    assert!(sender.try_recv().is_none());
}

#[test]
fn flex_layout_column_stacks_children() {
    use rustui::layout::{FlexProps, LayoutNode, LayoutTree, Length};
    // Root fills the viewport.
    let root = LayoutNode {
        width: Length::Percent(1.0),
        height: Length::Percent(1.0),
        ..LayoutNode::default()
    };
    let mut tree = LayoutTree::new(root);
    let _child0 = tree.add_child(0, FlexProps::column().grow(1.0).to_node(vec![]));
    let _child1 = tree.add_child(0, FlexProps::column().grow(1.0).to_node(vec![]));
    let rects = tree.compute(80.0, 24.0);
    assert_eq!(rects[0], rustui::Rect::new(0, 0, 80, 24));
    assert_eq!(rects[1].h, 12);
    assert_eq!(rects[2].h, 12);
    assert_eq!(rects[1].y, 0);
    assert_eq!(rects[2].y, 12);
}

#[test]
fn flex_layout_row_places_side_by_side() {
    use rustui::layout::{FlexDirection, FlexProps, LayoutNode, LayoutTree, Length};
    let root = LayoutNode {
        direction: FlexDirection::Row,
        width: Length::Percent(1.0),
        height: Length::Percent(1.0),
        ..LayoutNode::default()
    };
    let mut tree = LayoutTree::new(root);
    let _c0 = tree.add_child(0, FlexProps::row().grow(1.0).to_node(vec![]));
    let _c1 = tree.add_child(0, FlexProps::row().grow(1.0).to_node(vec![]));
    let rects = tree.compute(80.0, 24.0);
    assert_eq!(rects[1].w, 40);
    assert_eq!(rects[2].w, 40);
    assert_eq!(rects[1].x, 0);
    assert_eq!(rects[2].x, 40);
}

#[test]
fn diff_viewer_parses_unified_diff() {
    let diff = "--- a/foo.rs\n+++ b/foo.rs\n@@ -1,3 +1,4 @@\n fn main() {\n-    println!(\"hi\");\n+    println!(\"hello\");\n+    todo!();\n }\n";
    let viewer = rustui::DiffViewer::parse(diff);
    assert_eq!(viewer.hunks.len(), 1);
    let hunk = &viewer.hunks[0];
    // 2 file headers + 1 hunk header + 5 content lines = 8.
    assert_eq!(hunk.lines.len(), 8);
    assert_eq!(hunk.lines[0].kind, rustui::DiffLineKind::File);
    assert_eq!(hunk.lines[1].kind, rustui::DiffLineKind::File);
    assert_eq!(hunk.lines[2].kind, rustui::DiffLineKind::Hunk);
    assert_eq!(hunk.lines[3].kind, rustui::DiffLineKind::Context);
    assert_eq!(hunk.lines[4].kind, rustui::DiffLineKind::Remove);
    assert_eq!(hunk.lines[5].kind, rustui::DiffLineKind::Add);
    assert_eq!(hunk.lines[6].kind, rustui::DiffLineKind::Add);
    assert_eq!(hunk.lines[7].kind, rustui::DiffLineKind::Context);
}

#[test]
fn text_widget_wraps_long_lines() {
    use rustui::buffer::Buffer;
    use rustui::widgets::base::{PaintCtx, Widget};

    let text = Text::new("hello world this is a long line that should wrap");
    let mut buf = Buffer::empty(10, 5);
    let rects = vec![rustui::Rect::new(0, 0, 10, 5)];
    let mut ctx = PaintCtx {
        buffer: &mut buf,
        rect: rects[0],
        rects: &rects,
        elapsed: Duration::from_secs(0),
    };
    text.paint(&mut ctx);
    // "hello world this is a long line that should wrap"
    //  h(0) e(1) l(2) l(3) o(4) (5) w(6) o(7) r(8) l(9)
    assert_eq!(buf.cell(0, 0).unwrap().grapheme, "h");
    assert_eq!(buf.cell(9, 0).unwrap().grapheme, "l");
    // "d" at position 10 wraps to the next line.
    assert_eq!(buf.cell(0, 1).unwrap().grapheme, "d");
}

#[test]
fn spinner_cycles_frames_over_time() {
    use rustui::buffer::Buffer;
    use rustui::widgets::base::{PaintCtx, Widget};
    use rustui::Spinner;

    let spinner = Spinner::new().spinning(true).interval(80);
    let mut buf = Buffer::empty(1, 1);
    let rects = vec![rustui::Rect::new(0, 0, 1, 1)];
    let mut ctx = PaintCtx {
        buffer: &mut buf,
        rect: rects[0],
        rects: &rects,
        elapsed: Duration::from_millis(0),
    };
    spinner.paint(&mut ctx);
    assert_eq!(buf.cell(0, 0).unwrap().grapheme, "⠋");

    let mut ctx2 = PaintCtx {
        buffer: &mut buf,
        rect: rects[0],
        rects: &rects,
        elapsed: Duration::from_millis(80),
    };
    spinner.paint(&mut ctx2);
    assert_eq!(buf.cell(0, 0).unwrap().grapheme, "⠙");
}

#[test]
fn key_event_is_helper() {
    let k = KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
    };
    assert!(k.is(KeyCode::Char('q')));
    assert!(!k.is(KeyCode::Esc));

    let k_shift = KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::SHIFT,
    };
    assert!(!k_shift.is(KeyCode::Char('q')));
}

#[test]
fn spans_compose() {
    let s = Spans::plain("hello").push_styled(" world", rustui::Style::empty().fg(Color::RED));
    assert_eq!(s.spans.len(), 2);
    assert_eq!(s.to_plain(), "hello world");
    assert_eq!(s.width(), 11);
}

#[test]
fn flex_builder_chains() {
    let f = Flex::column()
        .align(Align::Center)
        .justify(Justify::SpaceBetween)
        .grow(2.0)
        .padding_all(1.0)
        .child(Text::new("a"))
        .child(Text::new("b"));
    assert_eq!(f.children.len(), 2);
    assert_eq!(f.props.justify, Justify::SpaceBetween);
    assert_eq!(f.props.grow, 2.0);
}
