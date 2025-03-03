use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let default_block = Block::default().style(Style::default());

    let bordered_block = Block::default()
        .style(Style::default())
        .borders(Borders::ALL);

    let header = get_header(&bordered_block);
    frame.render_widget(header, chunks[0]);

    let footer = get_footer(&bordered_block, &app);
    frame.render_widget(footer, chunks[2]);

    let body = get_body(&default_block, &app);
    frame.render_widget(body, chunks[1]);
}

fn get_header<'a>(block: &Block<'a>) -> Paragraph<'a> {
    Paragraph::new(Text::styled(
        "File Manager",
        Style::default().fg(Color::Green),
    ))
    .block(block.clone())
}

fn get_footer<'a>(block: &Block<'a>, app: &App) -> Paragraph<'a> {
    Paragraph::new(Text::styled(
        match app.mode {
            Mode::Command => app.command.clone().unwrap_or("".to_string()),
            _ => app.mode.to_string(),
        },
        Style::default().fg(Color::Green),
    ))
    .block(block.clone())
}

fn get_body<'a>(block: &Block<'a>, app: &'a App) -> Paragraph<'a> {
    Paragraph::new(Text::styled(
        &app.buffer_content,
        Style::default().fg(Color::White),
    ))
    .block(block.clone())
}
