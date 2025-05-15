use crate::app::{App, Mode};
use crate::file_helper::PathHelper;
use crate::utils::system;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Position,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
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
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let bordered_block = Block::default()
        .style(Style::default())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let default_block = Block::default().style(Style::default());

    let header = get_header(&default_block);
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

    let body: BodyLayout = get_body(app);
    frame.render_widget(body.parent, body_chunks[0]);
    frame.render_widget(body.current, body_chunks[1]);
    frame.render_widget(body.child, body_chunks[2]);

    app.cursor.container = Some(body_chunks[1]);

    let position = Position {
        x: body_chunks[1].x + app.cursor.x,
        y: body_chunks[1].y + app.cursor.y,
    };
    frame.set_cursor_position(position);

    app.cursor.update_frame(frame);
}

fn get_header<'a>(block: &Block<'a>) -> Paragraph<'a> {
    let spans = Line::from(vec![
        Span::styled(get_hostname(), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {}", get_dirname()),
            Style::default().fg(Color::Blue),
        ),
    ]);

    Paragraph::new(spans).block(block.clone())
}

fn get_footer<'a>(block: &Block<'a>, app: &App) -> Paragraph<'a> {
    let spans: Line = match app.mode {
        Mode::Command => Line::from(Span::raw(app.command.clone().unwrap_or("".to_string()))),
        _ => Line::from(vec![
            Span::styled(format!("{}", app.mode), Style::default().fg(Color::White)),
            Span::styled(
                format!(" {}        ", get_current_file_permissions(app)),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(
                format!(" {}        ", get_current_file_size(app)),
                Style::default().fg(Color::Green),
            ),
        ]),
    };

    Paragraph::new(spans).block(block.clone())
}

fn get_body<'a>(app: &mut App) -> BodyLayout {
    let mut current_dir: PathHelper = match app.path {
        Some(ref path) => path.clone(),
        None => PathHelper::new("./"),
    };
    let current_files = current_dir.get_dir_names_printable(true).unwrap_or(vec![]);
    if app.buffer_content.is_empty() {
        app.buffer_content = current_files.join("\n");
    }

    let parent_dir: PathHelper = match current_dir.get_parent() {
        Ok(path) => PathHelper::new(&path),
        Err(_) => PathHelper::new(".."),
    };
    let parent_files: Vec<String> = parent_dir.get_dir_names_printable(true).unwrap_or(vec![]);

    let left = Paragraph::new(parent_files.join("\n"))
        .block(
            Block::default()
                .title("Parent Directory")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default());

    let middle = Paragraph::new(app.buffer_content.clone()).block(
        Block::default()
            .title("Current Directory")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    let right = Paragraph::new("")
        .block(
            Block::default()
                .title("Child Directory")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default());

    BodyLayout {
        parent: left,
        current: middle,
        child: right,
    }
}

pub fn get_hostname() -> String {
    let name = system::whoami();
    let host = system::hostname();

    let prompt = format!("{}@{}", name.trim(), host.trim());

    prompt
}

pub fn get_dirname() -> String {
    system::pwd()
}

pub fn get_current_file(app: &App) -> String {
    app.get_hovered_filename()
}

pub fn get_current_file_permissions(app: &App) -> String {
    let filename = app.get_hovered_filename();

    system::get_file_permission(filename)
}

pub fn get_current_file_size(app: &App) -> String {
    let filename = app.get_hovered_filename();

    system::get_file_size(filename)
}
