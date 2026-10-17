use crate::{
    app::{App, Mode},
    file_helper::PathHelper,
    utils::{
        buffer_storage::{FileType, State},
        system,
    },
};

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Mutex, OnceLock},
    time::UNIX_EPOCH,
};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};

pub struct BodyLayout<'a> {
    pub parent: Paragraph<'a>,
    pub current: Paragraph<'a>,
    pub child: Paragraph<'a>,
}

pub fn get_header<'a>(block: &Block<'a>, app: &App) -> Paragraph<'a> {
    let spans = Line::from(vec![
        Span::styled(app.hostname.clone(), Style::default().fg(Color::Yellow)),
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
                format!(
                    " {}        ",
                    system::get_file_permission(format!(
                        "{}/{}",
                        app.path.get_path_str(),
                        app.get_hovered_filename()
                    ))
                ),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(
                format!(
                    " {}        ",
                    system::get_file_size(format!(
                        "{}/{}",
                        app.path.get_path_str(),
                        app.get_hovered_filename()
                    ))
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
    };

    Paragraph::new(spans).block(block.clone())
}

pub fn get_body<'a>(app: &'a mut App, child_max_lines: usize) -> BodyLayout<'a> {
    let current_dir: PathHelper = app.path.clone();

    let cl = current_dir.clone().get_absolute_path();
    let dir_parts = cl.split("/").collect::<Vec<_>>();
    let current_folder = dir_parts[dir_parts.len() - 1].to_string();

    let mut current_files = current_dir
        .get_dir_names_printable(true)
        .unwrap_or(vec![])
        .iter()
        .map(|s| PathHelper::trim_path(s.as_str()))
        .collect::<Vec<_>>();

    current_files.sort_by(|a: &String, b: &String| {
        let file_type_a = app.get_file_type(&current_dir.get_absolute_path(), a);
        let file_type_b = app.get_file_type(&current_dir.get_absolute_path(), b);

        match (file_type_a, file_type_b) {
            (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
            (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
            _ => a.to_lowercase().cmp(&b.to_lowercase()),
        }
    });

    if app.rerender_dir_content {
        let offset = app.offset_top;
        let end = std::cmp::min(offset + app.layout[1].height as usize, current_files.len());
        let mut current_files = current_files[offset..end].to_vec();
        current_files.insert(0, String::from("../"));

        app.buffer_content = current_files.join("\n");
        app.undo_stack.reset(app.buffer_content.clone());
        app.rerender_dir_content = false;
    }

    let mut parent_dir: PathHelper = current_dir.clone();
    let _ = parent_dir.cd("..");
    let mut parent_files: Vec<String> = parent_dir
        .get_dir_names_trimmed()
        .unwrap_or(vec![])
        .iter()
        .map(|s| PathHelper::trim_path(s.as_str()))
        .collect::<Vec<_>>();

    parent_files.sort_by(|a: &String, b: &String| {
        let file_type_a = app.get_file_type(&parent_dir.get_absolute_path(), a);
        let file_type_b = app.get_file_type(&parent_dir.get_absolute_path(), b);

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

    let right = Paragraph::new(get_file_preview_content(
        app,
        hovered_file.clone(),
        current_dir.clone(),
        child_max_lines,
    ))
    .block(
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
    app: &'a mut App,
    hovered_file: String,
    current_view: PathHelper,
    max_lines: usize,
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
                .unwrap_or_else(|| {
                    app.buffer_storage
                        .get_view(&app.path.get_absolute_path())
                        .unwrap()
                })
        }
    };

    let mut files = match file_type {
        FileType::File => {
            let path = Path::new(&current_view.get_path_str()).join(&hovered_file);
            let path_string = path.to_string_lossy().into_owned();
            let metadata = std::fs::metadata(&path).ok();
            let extension = path
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or("");
            let size = metadata
                .as_ref()
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            let modified = metadata
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let cache_key = PreviewCacheKey {
                path: path_string.clone(),
                size,
                modified,
                max_lines,
                extension: extension.to_ascii_lowercase(),
            };

            if let Some(lines) = preview_cache()
                .lock()
                .ok()
                .and_then(|cache| cache.get(&cache_key).cloned())
            {
                lines
            } else {
                let content = system::get_file_preview(path_string, max_lines)
                    .unwrap_or_else(|_| "Error Reading".to_string());
                let lines = if size <= 256 * 1024 {
                    get_styled_preview(&content, extension)
                } else {
                    plain_preview(&content)
                };
                if let Ok(mut cache) = preview_cache().lock() {
                    if cache.len() >= 64 {
                        cache.clear();
                    }
                    cache.insert(cache_key, lines.clone());
                }
                lines
            }
        }
        FileType::Directory => {
            system::get_dir_preview(current_view.get_path_str() + "/" + &hovered_file)
                .unwrap_or("Error Reading".to_string())
                .lines()
                .map(|line| {
                    let (bg, fg) = get_line_colors(app, &child_view.dir, line, &hovered_file);

                    Line::from(vec![Span::styled(
                        line.to_string(),
                        Style::default().bg(bg).fg(fg),
                    )])
                })
                .collect::<Vec<_>>()
        }
        FileType::Unknown => {
            let err = match hovered_file == "../" {
                true => "".to_string(),
                false => String::from("Unknown File Type"),
            };

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

#[derive(Hash, PartialEq, Eq)]
struct PreviewCacheKey {
    path: String,
    size: u64,
    modified: u128,
    max_lines: usize,
    extension: String,
}

fn preview_cache() -> &'static Mutex<HashMap<PreviewCacheKey, Vec<Line<'static>>>> {
    static CACHE: OnceLock<Mutex<HashMap<PreviewCacheKey, Vec<Line<'static>>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn get_styled_preview(content: &str, file_extension: &str) -> Vec<Line<'static>> {
    let ps = syntax_set();
    let ts = theme_set();

    let syntax = ps
        .find_syntax_by_extension(file_extension)
        .unwrap_or(ps.find_syntax_plain_text());
    let Some(theme) = ts.themes.get("Solarized (dark)") else {
        return plain_preview(content);
    };
    let mut h = HighlightLines::new(syntax, theme);

    let mut lines = Vec::new();
    for line in LinesWithEndings::from(content) {
        let spans: Vec<Span<'static>> = match h.highlight_line(line, ps) {
            Ok(regions) => regions
                .into_iter()
                .filter_map(|segment| {
                    syntect_tui::into_span(segment).ok().map(|span| {
                        Span::styled(span.content.to_string(), span.style.bg(Color::Reset))
                    })
                })
                .collect(),
            Err(_) => vec![Span::raw(line.to_string())],
        };

        lines.push(Line::from(spans));
    }

    lines
}

fn plain_preview(content: &str) -> Vec<Line<'static>> {
    content
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect()
}
