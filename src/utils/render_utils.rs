use crate::{
    app::{App, Mode},
    file_helper::PathHelper,
    utils::{
        buffer_storage::{FileType, State},
        system,
    },
};
use ratatui::text::{Line, Span};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect_tui::into_span;

pub struct BodyLayout {
    pub parent: Paragraph<'static>,
    pub current: Paragraph<'static>,
    pub child: Paragraph<'static>,
}

pub fn get_header<'a>(block: &Block<'a>, app: &App) -> Paragraph<'a> {
    let spans = Line::from(vec![
        Span::styled(get_hostname(), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {}", get_dirname(app)),
            Style::default().fg(Color::Blue),
        ),
    ]);

    Paragraph::new(spans).block(block.clone())
}

pub fn get_footer<'a>(block: &Block<'a>, app: &App) -> Paragraph<'a> {
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

pub fn get_body<'a>(app: &mut App) -> BodyLayout {
    let current_dir: PathHelper = app.path.clone();

    let cl = current_dir.clone().get_absolute_path();
    let dir_parts = cl.split("/").collect::<Vec<_>>();
    let current_folder = dir_parts[dir_parts.len() - 1].to_string();

    let mut current_files = current_dir
        .get_dir_names_printable(true)
        .unwrap_or(vec![])
        .iter()
        .map(PathHelper::trim_path)
        .collect::<Vec<_>>();

    current_files.sort_by(|a: &String, b: &String| {
        let file_type_a = app.get_file_type(&current_dir.get_absolute_path(), a);
        let file_type_b = app.get_file_type(&current_dir.get_absolute_path(), b);

        match (file_type_a, file_type_b) {
            (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
            (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });

    current_files.insert(0, String::from("../"));

    if app.rerender_dir_content {
        app.buffer_content = current_files.join("\n");
        app.rerender_dir_content = false;
    }

    let mut parent_dir: PathHelper = current_dir.clone();
    let _ = parent_dir.cd("..");
    let mut parent_files: Vec<String> = parent_dir
        .get_dir_names_trimmed()
        .unwrap_or(vec![])
        .iter()
        .map(PathHelper::trim_path)
        .collect::<Vec<_>>();

    parent_files.sort_by(|a: &String, b: &String| {
        let file_type_a = app.get_file_type(&current_dir.get_absolute_path(), a);
        let file_type_b = app.get_file_type(&current_dir.get_absolute_path(), b);

        match (file_type_a, file_type_b) {
            (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
            (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });

    let left = Paragraph::new(
        parent_files
            .iter()
            .map(|line| {
                let (bg, fg) =
                    get_line_colors(app, &parent_dir.get_absolute_path(), line, &current_folder);

                Line::from(vec![Span::styled(
                    format!("{:<width$}", line, width = u16::MAX as usize),
                    Style::default().bg(bg).fg(fg),
                )])
            })
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title(parent_dir.get_absolute_path())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    let current_view = &app
        .buffer_storage
        .get_view(&app.path.get_absolute_path())
        .unwrap_or_else(|| app.buffer_storage.get_view(&system::pwd()).unwrap())
        .dir;

    let hovered_file = app.get_hovered_filename();

    let middle = Paragraph::new(
        app.buffer_content
            .lines()
            .map(|line| {
                let (bg, fg) = get_line_colors(app, current_view, line, &hovered_file);

                Line::from(Span::styled(
                    format!("{:<width$}", line, width = u16::MAX as usize),
                    Style::default().bg(bg).fg(fg),
                ))
            })
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title("Current Directory")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    let p = Paragraph::new(get_file_preview_content(
        app,
        &hovered_file.clone(),
        current_dir.clone(),
    ));

    let right = p.block(
        Block::default()
            .title(if hovered_file == "../" {
                "Parent Directory".to_string()
            } else {
                hovered_file
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

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

pub fn get_dirname(app: &App) -> String {
    app.path.get_absolute_path()
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

pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
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

pub fn get_line_colors(
    app: &App,
    current_view: &str,
    line: &str,
    hovered_file: &str,
) -> (Color, Color) {
    match app.get_file_type(current_view, line.trim()) {
        FileType::File => match line == hovered_file.trim() {
            false => (Color::default(), Color::LightGreen),
            true => (Color::LightGreen, Color::Black),
        },
        FileType::Directory => match line == hovered_file.trim() {
            false => (Color::default(), Color::LightBlue),
            true => (Color::LightBlue, Color::Black),
        },
        FileType::Unknown => match line == hovered_file.trim() {
            true => (Color::default(), Color::default()),
            false => (Color::default(), Color::default()),
        },
    }
}

pub fn get_file_preview_content<'a>(
    app: &mut App,
    hovered_file: &'a String,
    current_view: PathHelper,
) -> Vec<Line<'a>> {
    let file_type = app.get_file_type(&current_view.get_absolute_path(), &hovered_file);

    let mut child_view = current_view.clone();
    let _ = child_view.cd(&hovered_file);
    let child_view = match app.buffer_storage.get_view(&child_view.get_absolute_path()) {
        Some(view) => view,
        None => {
            let _ = app.buffer_storage.add_view(child_view.get_absolute_path());

            app.buffer_storage
                .get_view(&child_view.get_absolute_path())
                .unwrap_or_else(|| app.buffer_storage.get_view(&system::pwd()).unwrap())
        }
    };

    let mut files = match file_type {
        FileType::File => apply_syntax_highlighting(
            system::get_file_preview(hovered_file.clone(), 30)
                .unwrap_or("Error Reading".to_string()),
            &hovered_file,
        ),
        FileType::Directory => system::get_dir_preview(hovered_file.clone())
            .unwrap_or("Error Reading".to_string())
            .lines()
            .map(|line| {
                let (bg, fg) = get_line_colors(app, &child_view.dir, line, &hovered_file);

                Line::from(vec![Span::styled(
                    line.to_string(),
                    Style::default().bg(bg).fg(fg),
                )])
            })
            .collect::<Vec<_>>(),
        FileType::Unknown => {
            let err = String::from("Unknown File Type");

            err.lines()
                .map(|line| {
                    let (bg, fg) = get_line_colors(app, &child_view.dir, line, &hovered_file);

                    Line::from(vec![Span::styled(
                        line.to_string(),
                        Style::default().bg(bg).fg(fg),
                    )])
                })
                .collect::<Vec<_>>()
        }
    };

    if file_type == FileType::Directory {
        files.sort_by(|a: &Line, b: &Line| {
            let file_type_a = app.get_file_type(&child_view.dir, a.to_string().as_str());
            let file_type_b = app.get_file_type(&child_view.dir, b.to_string().as_str());

            match (file_type_a, file_type_b) {
                (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
                (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
                _ => a.to_string().cmp(&b.to_string()),
            }
        });
    }

    files
}

pub fn apply_syntax_highlighting<'a>(file_name: String, content: &'a String) -> Vec<Line> {
    let file_extension = file_name.split('.').last().unwrap_or("txt").to_string();

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension(&file_extension).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut lines = Vec::new();
    for line in LinesWithEndings::from(&content) {
        let spans: Vec<Span> = h
            .highlight_line(line, &ps)
            .unwrap()
            .into_iter()
            .filter_map(|segment| into_span(segment).ok())
            .collect();

        let line = Line::from(spans);
        lines.push(line);
    }

    lines
}
