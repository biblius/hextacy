use tui::{
    backend::Backend,
    layout::Alignment,
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::layout;

pub fn header<B: Backend>(frame: &mut Frame<B>) {
    let welcome_text = Paragraph::new(Span::from("Welcome to XTC!"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    let _header = layout::vertical(frame)[0];
    frame.render_widget(welcome_text, _header);
}

pub fn footer<B: Backend>(frame: &mut Frame<B>, describe: &str) {
    let footer = Paragraph::new(Span::from(describe))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let foot = layout::vertical(frame)[2];
    frame.render_widget(footer, foot);
}
