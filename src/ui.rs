use crate::app::{App, Mode};
use crate::file_helper::PathHelper;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Position,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

struct BodyLayout {
    parent: Paragraph<'static>,
    current: Paragraph<'static>,
    child: Paragraph<'static>,
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let bordered_block = Block::default()
        .style(Style::default())
        .borders(Borders::ALL);

    let header = get_header(&bordered_block);
    frame.render_widget(header, chunks[0]);

    let footer = get_footer(&bordered_block, &app);
    frame.render_widget(footer, chunks[2]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 2),
        ])
        .split(chunks[1]);

    let body: BodyLayout = get_body(&app);
    frame.render_widget(body.parent, body_chunks[0]);
    frame.render_widget(body.current, body_chunks[1]);
    frame.render_widget(body.child, body_chunks[2]);

    let (cursor_x, cursor_y) = app.cursor_position;
    let position = Position {
        x: body_chunks[1].x + 1 + cursor_x,
        y: body_chunks[1].y + 1 + cursor_y,
    };
    frame.set_cursor_position(position);
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

fn get_body<'a>(app: &'a App) -> BodyLayout {
    let mut current_dir: PathHelper = match app.path {
        Some(ref path) => path.clone(),
        None => PathHelper::new("./"),
    };
    let current_files = current_dir.get_dir_names_printable(true).unwrap_or(vec![]);

    let parent_dir: PathHelper = match current_dir.get_parent() {
        Ok(path) => PathHelper::new(&path),
        Err(_) => PathHelper::new(".."),
    };
    let parent_files: Vec<String> = parent_dir.get_dir_names_printable(true).unwrap_or(vec![]);

    let left = Paragraph::new(parent_files.join("\n"))
        .block(
            Block::default()
                .title("Parent Directory")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    let middle = Paragraph::new(current_files.join("\n"))
        .block(
            Block::default()
                .title("Current Directory")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black));

    let right = Paragraph::new("")
        .block(
            Block::default()
                .title("Child Directory")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    BodyLayout {
        parent: left,
        current: middle,
        child: right,
    }
}
