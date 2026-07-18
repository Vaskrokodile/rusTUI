//! Performance benchmarks for RusTUI rendering pipelines.
//!
//! Covers buffer operations, text wrapping, widget tree building/layout,
//! full frame rendering, and ANSI/Markdown parsing. Organized into criterion
//! benchmark groups so each pipeline stage can be tracked independently.

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use rustui::widgets::base::{PaintCtx, Widget, WidgetTree};
use rustui::{ansi, wrap, Buffer, Color, Flex, Markdown, Rect, Spans, Style, Text};

/// Helper: build a small widget tree (a column with a few text children).
fn small_tree() -> Box<dyn Widget> {
    Box::new(
        Flex::column()
            .child(Text::new("header line"))
            .child(Text::new(
                "some body text that wraps across a couple of lines",
            ))
            .child(Text::new("footer")),
    )
}

/// Helper: build a medium widget tree (nested flex containers with text).
fn medium_tree() -> Box<dyn Widget> {
    let mut root = Flex::column().child(Text::new("title").fg(Color::CYAN));
    for i in 0..8 {
        let row = Flex::row()
            .child(Text::new(format!("label {i}")).grow(1.0))
            .child(Text::new("value that is somewhat longer and will wrap nicely").grow(2.0));
        root = root.child(row);
    }
    root = root.child(Text::new("bottom status line"));
    Box::new(root)
}

/// Buffer creation and clearing at various sizes.
fn bench_buffer_create_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer/create_clear");
    for &(w, h) in &[(80u16, 24u16), (200, 50), (500, 100)] {
        group.bench_with_input(
            BenchmarkId::new("empty", format!("{w}x{h}")),
            &(w, h),
            |b, &(w, h)| {
                b.iter(|| {
                    let buf = Buffer::empty(black_box(w), black_box(h));
                    black_box(buf);
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("clear", format!("{w}x{h}")),
            &(w, h),
            |b, &(w, h)| {
                b.iter_batched(
                    || Buffer::empty(w, h),
                    |mut buf| {
                        buf.clear();
                        black_box(buf);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Buffer printing: short, long, and unicode text.
fn bench_buffer_print(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer/print");

    let short = "hello world";
    let long: String = "the quick brown fox jumps over the lazy dog ".repeat(20);
    let unicode = "héllo wörld 😀😀😀 日本語テスト \u{1f600} café naïve";

    group.bench_function("short", |b| {
        b.iter_batched(
            || Buffer::empty(80, 24),
            |mut buf| {
                buf.print(0, 0, black_box(short), Style::empty());
                black_box(buf);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.bench_function("long", |b| {
        b.iter_batched(
            || Buffer::empty(200, 50),
            |mut buf| {
                buf.print(0, 0, black_box(&long), Style::empty());
                black_box(buf);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.bench_function("unicode", |b| {
        b.iter_batched(
            || Buffer::empty(80, 24),
            |mut buf| {
                buf.print(0, 0, black_box(unicode), Style::empty());
                black_box(buf);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// Word wrapping at various text lengths and widths.
fn bench_word_wrap(c: &mut Criterion) {
    let mut group = c.benchmark_group("wrap/word_wrap");

    let short = Spans::plain("hello world foo bar baz");
    let medium = Spans::plain(
        "the quick brown fox jumps over the lazy dog and then keeps running ".repeat(5),
    );
    let long = Spans::plain("word ".repeat(200));

    for &(text, width, label) in &[
        (&short, 80usize, "short/w80"),
        (&medium, 80, "medium/w80"),
        (&medium, 40, "medium/w40"),
        (&long, 80, "long/w80"),
        (&long, 40, "long/w40"),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| {
                let lines = wrap::word_wrap(black_box(text), black_box(width));
                black_box(lines);
            });
        });
    }
    group.finish();
}

/// Widget tree building and layout computation.
fn bench_widget_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("widget_tree");

    group.bench_function("build_small", |b| {
        b.iter(|| {
            let tree = WidgetTree::build(black_box(small_tree()));
            black_box(tree);
        });
    });
    group.bench_function("build_medium", |b| {
        b.iter(|| {
            let tree = WidgetTree::build(black_box(medium_tree()));
            black_box(tree);
        });
    });
    group.bench_function("layout_small", |b| {
        b.iter_batched(
            || WidgetTree::build(small_tree()),
            |tree| {
                let rects = tree.compute_rects(black_box(80.0), black_box(24.0));
                black_box(rects);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.bench_function("layout_medium", |b| {
        b.iter_batched(
            || WidgetTree::build(medium_tree()),
            |tree| {
                let rects = tree.compute_rects(black_box(200.0), black_box(50.0));
                black_box(rects);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// Full frame render: build tree, compute layout, paint into a buffer.
fn bench_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_frame");

    group.bench_function("small_80x24", |b| {
        b.iter_batched(
            || (WidgetTree::build(small_tree()), Buffer::empty(80, 24)),
            |(tree, mut buf)| {
                let rects = tree.compute_rects(80.0, 24.0);
                tree.paint(&mut buf, &rects, Duration::ZERO);
                black_box(buf);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.bench_function("medium_200x50", |b| {
        b.iter_batched(
            || (WidgetTree::build(medium_tree()), Buffer::empty(200, 50)),
            |(tree, mut buf)| {
                let rects = tree.compute_rects(200.0, 50.0);
                tree.paint(&mut buf, &rects, Duration::ZERO);
                black_box(buf);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// ANSI escape-sequence parsing at various input sizes.
fn bench_ansi_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi/parse");

    let short = "\x1b[1;31mError:\x1b[0m something went wrong";
    let medium: String = (0..20)
        .map(|i| {
            let color = i % 8;
            format!("\x1b[1;3{color}mline {i} of output\x1b[0m ")
        })
        .collect();
    let long: String = (0..200)
        .map(|i| {
            let r = i % 255;
            format!("\x1b[38;2;{r};100;200mtext\x1b[0m ")
        })
        .collect();

    group.bench_function("short", |b| {
        b.iter(|| {
            let spans = ansi::parse(black_box(short));
            black_box(spans);
        });
    });
    group.bench_function("medium", |b| {
        b.iter(|| {
            let spans = ansi::parse(black_box(&medium));
            black_box(spans);
        });
    });
    group.bench_function("long", |b| {
        b.iter(|| {
            let spans = ansi::parse(black_box(&long));
            black_box(spans);
        });
    });
    group.finish();
}

/// Markdown parsing/rendering at various input sizes.
///
/// This paints a [`Markdown`] widget directly into a buffer (bypassing the
/// widget tree/layout stages) so the cost is dominated by markdown block
/// parsing, inline formatting, and line wrapping.
fn bench_markdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown");

    let short = "# Title\n\nThis is a **bold** paragraph with *italic* text.\n";
    let medium: String = {
        let mut s = String::new();
        s.push_str("# Heading\n\n");
        for i in 0..10 {
            s.push_str(&format!("## Subheading {i}\n\n"));
            s.push_str("This is a paragraph with **bold** and *italic* text.\n\n");
            s.push_str("- item one\n- item two\n- item three\n\n");
            s.push_str("```\nfn main() { println!(\"hi\"); }\n```\n\n");
        }
        s
    };
    let long: String = {
        let mut s = String::new();
        for i in 0..100 {
            s.push_str(&format!("## Section {i}\n\n"));
            s.push_str("A paragraph with **bold**, *italic*, and `code` spans.\n\n");
            s.push_str("1. first\n2. second\n3. third\n\n");
            s.push_str("> a quoted line\n> another quoted line\n\n");
        }
        s
    };

    let paint_md = |src: &str, w: u16, h: u16| {
        let mut buf = Buffer::empty(w, h);
        let md = Markdown::new(src);
        let rect = Rect::new(0, 0, w, h);
        let mut ctx = PaintCtx {
            buffer: &mut buf,
            rect,
            rects: &[rect],
            elapsed: Duration::ZERO,
        };
        md.paint(&mut ctx);
        black_box(buf);
    };

    group.bench_function("short", |b| {
        b.iter(|| paint_md(black_box(short), 80, 24));
    });
    group.bench_function("medium", |b| {
        b.iter(|| paint_md(black_box(&medium), 80, 40));
    });
    group.bench_function("long", |b| {
        b.iter(|| paint_md(black_box(&long), 100, 50));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_buffer_create_clear,
    bench_buffer_print,
    bench_word_wrap,
    bench_widget_tree,
    bench_full_frame,
    bench_ansi_parse,
    bench_markdown,
);
criterion_main!(benches);
