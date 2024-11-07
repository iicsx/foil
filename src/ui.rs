use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    app.load_new_buffer("./data/test_file.txt");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let bar = Block::default().style(Style::default());

    let header = get_header(&bar);
    frame.render_widget(header, chunks[0]);

    let footer = get_footer(&bar, &app);
    frame.render_widget(footer, chunks[2]);

    let body = get_body(&bar, &app);
    frame.render_widget(body, chunks[1]);
}

fn get_header<'a>(bar: &Block<'a>) -> Paragraph<'a> {
    Paragraph::new(Text::styled(
        "File Manager",
        Style::default().fg(Color::Green),
    ))
    .block(bar.clone())
}

fn get_footer<'a>(bar: &Block<'a>, app: &App) -> Paragraph<'a> {
    Paragraph::new(Text::styled(
        app.mode.to_string(),
        Style::default().fg(Color::Green),
    ))
    .block(bar.clone())
}

fn get_body<'a>(bar: &Block<'a>, app: &'a App) -> Paragraph<'a> {
    Paragraph::new(Text::styled(
        &app.buffer_content,
        Style::default().fg(Color::White),
    ))
    .block(bar.clone())
}
