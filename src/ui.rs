use crate::app::{App, Mode};
use crate::file_helper::PathHelper;
use crate::utils::{buffer_storage::State, system};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    prelude::Position,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
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

    if app.need_confirmation {
        let area = popup_area(frame.area(), 40, 20);

        let block = Block::bordered()
            .title(Line::from(" Confirm (y/n) ").centered())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let content = get_confirmation_content(&block, app);

        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(content, area);
    }

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
        Mode::Command => Line::from(vec![Span::raw(format!(
            ":{}",
            app.command.clone().unwrap_or("".to_string())
        ))]),
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
    let mut current_dir: PathHelper = app.path.clone();
    let mut current_files = current_dir
        .get_dir_names_printable(true)
        .unwrap_or(vec![])
        .iter()
        .map(PathHelper::trim_path)
        .collect::<Vec<_>>();

    current_files.insert(0, String::from("../"));

    if app.rerender_dir_content {
        app.buffer_content = current_files.join("\n");
        app.rerender_dir_content = false;
    }

    let parent_dir: PathHelper = match current_dir.sim_cd("..") {
        Ok(path) => PathHelper::new(&path, &system::pwd()),
        Err(_) => PathHelper::new("..", &system::pwd()),
    };
    let parent_files: Vec<String> = parent_dir
        .get_dir_names_trimmed()
        .unwrap_or(vec![])
        .iter()
        .map(PathHelper::trim_path)
        .collect();

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

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    area
}

pub fn get_confirmation_content<'a>(block: &Block<'a>, app: &mut App) -> Paragraph<'a> {
    let files_modified = app.get_files(State::Modified);
    let files_created = app.get_files(State::Created);
    let files_deleted = app.get_files(State::Deleted);
    let files_moved = app.get_files(State::Moved);

    let lines_modified = files_modified
        .iter()
        .map(|file| {
            Line::from(vec![
                Span::styled("RENAME ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} -> {}", file.original_name.clone(), file.name)),
            ])
        })
        .collect::<Vec<_>>();

    let lines_created = files_created
        .iter()
        .map(|file| {
            Line::from(vec![
                Span::styled("CREATE ", Style::default().fg(Color::Cyan)),
                Span::raw(file.name.clone()),
            ])
        })
        .collect::<Vec<_>>();

    let lines_deleted = files_deleted
        .iter()
        .map(|file| {
            Line::from(vec![
                Span::styled("DELETE ", Style::default().fg(Color::Red)),
                Span::raw(file.name.clone()),
            ])
        })
        .collect::<Vec<_>>();

    let lines_moved = files_moved
        .iter()
        .map(|file| {
            Line::from(vec![
                Span::styled("MOVE ", Style::default().fg(Color::Magenta)),
                Span::raw(file.name.clone()),
            ])
        })
        .collect::<Vec<_>>();

    let all_lines = lines_modified
        .into_iter()
        .chain(lines_created)
        .chain(lines_deleted)
        .chain(lines_moved)
        .collect::<Vec<_>>();

    Paragraph::new(all_lines).block(block.clone())
}
