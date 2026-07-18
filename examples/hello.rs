//! Minimal RusTUI example: a centered "hello" message.

use rustui::{App, Color, Flex, Text};

fn main() -> rustui::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut app = App::default();
    runtime.block_on(app.run(|_ctx| {
        Flex::column()
            .align(rustui::Align::Center)
            .justify(rustui::Justify::Center)
            .child(Text::new("hello from RusTUI").fg(Color::CYAN))
    }))?;
    Ok(())
}
