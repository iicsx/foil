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
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
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
use unicode_width::UnicodeWidthChar;

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
    let filename = app.get_hovered_filename();
    let path = Path::new(&app.path.get_absolute_path()).join(&filename);
    let status = system::get_file_permissions(&path)
        .unwrap_or_else(|_| format!("?????????? - - - - {}", filename));
    let mode = match app.mode {
        Mode::Command => format!(":{}", app.command.as_deref().unwrap_or("")),
        _ => format!("{}", app.mode),
    };
    let spans = Line::from(vec![
        Span::styled(format!("  {} | ", mode), Style::default().fg(Color::White)),
        Span::styled(status, Style::default().fg(Color::Magenta)),
    ]);

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

    sort_names(
        app,
        &current_dir.get_absolute_path(),
        &mut current_files,
        true,
    );

    if app.rerender_dir_content {
        refresh_current_buffer(app, &current_files);
    }

    let mut parent_dir: PathHelper = current_dir.clone();
    let _ = parent_dir.cd("..");
    let mut parent_files: Vec<String> = parent_dir
        .get_dir_names_trimmed()
        .unwrap_or(vec![])
        .iter()
        .map(|s| PathHelper::trim_path(s.as_str()))
        .collect::<Vec<_>>();

    sort_names(
        app,
        &parent_dir.get_absolute_path(),
        &mut parent_files,
        false,
    );

    let left = Paragraph::new(
        parent_files
            .iter()
            .map(|line| {
                let (bg, fg) =
                    get_line_colors(app, &parent_dir.get_absolute_path(), line, &current_folder);
                let row_width = app
                    .layout
                    .first()
                    .map(|pane| pane.width.saturating_sub(2) as usize)
                    .unwrap_or_else(|| line.chars().count());
                let padding = " ".repeat(row_width.saturating_sub(line.chars().count()));

                Line::from(vec![
                    Span::styled(line.clone(), Style::default().bg(bg).fg(fg)),
                    Span::styled(padding, Style::default().bg(bg)),
                ])
            })
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title(parent_dir.get_absolute_path())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    let current_view_path = app.path.get_absolute_path();
    let current_view = app
        .buffer_storage
        .views
        .get(&current_view_path)
        .map(|view| view.dir.as_str())
        .unwrap_or(&current_view_path);

    let hovered_file = app.get_hovered_filename();

    let middle = Paragraph::new(
        app.buffer_content
            .lines()
            .enumerate()
            .map(|(row, line)| {
                let (bg, fg) = get_line_colors_for_hover(
                    app,
                    current_view,
                    line,
                    row == app.cursor.y.saturating_sub(1) as usize,
                );
                let row_width = app
                    .layout
                    .get(1)
                    .map(|pane| pane.width.saturating_sub(2) as usize)
                    .unwrap_or_else(|| line.chars().count());
                let padding = " ".repeat(row_width.saturating_sub(line.chars().count()));

                Line::from(vec![
                    Span::styled(line.to_string(), Style::default().bg(bg).fg(fg)),
                    Span::styled(padding, Style::default().bg(bg)),
                ])
            })
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title("Current Directory")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    let preview_width = app
        .layout
        .get(2)
        .map(|pane| pane.width.saturating_sub(4) as usize)
        .unwrap_or(0);
    let preview_lines = wrap_preview_lines(
        get_file_preview_content(
            app,
            hovered_file.clone(),
            current_dir.clone(),
            child_max_lines,
        ),
        preview_width,
    );

    let right = Paragraph::new(preview_lines)
        .wrap(Wrap { trim: false })
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

fn refresh_current_buffer(app: &mut App, files: &[String]) {
    let offset = app.offset_top.min(files.len());
    let visible_height = app
        .layout
        .get(1)
        .map(|pane| pane.height as usize)
        .unwrap_or(0);
    let end = offset.saturating_add(visible_height).min(files.len());
    let mut visible_files = files[offset..end].to_vec();
    visible_files.insert(0, String::from("../"));

    // Keep pending names visible when a resize or directory refresh occurs.
    if let Some(view) = app.buffer_storage.get_view(&app.path.get_absolute_path()) {
        for file in view.files.values() {
            if file.state != crate::utils::buffer_storage::State::Unmodified {
                if let Some(existing) = visible_files
                    .iter_mut()
                    .find(|name| **name == file.original_name)
                {
                    *existing = file.name.clone();
                }
            }
        }
    }

    app.buffer_content = visible_files.join("\n");
    app.undo_stack.reset(app.buffer_content.clone());
    app.rerender_dir_content = false;
}

fn sort_names(app: &App, directory: &str, names: &mut [String], ignore_case: bool) {
    let mut entries = names
        .iter()
        .map(|name| {
            (
                name.clone(),
                app.get_file_type(directory, name),
                ignore_case.then(|| name.to_lowercase()),
            )
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| match (&a.1, &b.1) {
        (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
        (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
        _ if ignore_case => a.2.cmp(&b.2),
        _ => a.0.cmp(&b.0),
    });

    for (name, (sorted_name, _, _)) in names.iter_mut().zip(entries) {
        *name = sorted_name;
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

pub fn get_current_file_permissions(app: &App) -> std::io::Result<String> {
    let filename = app.get_hovered_filename();

    system::get_file_permission(&filename)
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
    let launch_dir = Path::new(&system::pwd()).to_path_buf();

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
            let from = relative_file_path(&launch_dir, &file.original_dir, &file.original_name);
            let to = relative_file_path(&launch_dir, &file.dir, &file.name);
            Line::from(vec![
                Span::styled("MOVE ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{} -> {}", from, to)),
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

fn relative_file_path(launch_dir: &Path, directory: &str, name: &str) -> String {
    let path = Path::new(directory).join(name);
    path.strip_prefix(launch_dir)
        .unwrap_or(&path)
        .to_string_lossy()
        .into_owned()
}

pub fn get_line_colors(
    app: &App,
    current_view: &str,
    line: &str,
    hovered_file: &str,
) -> (Color, Color) {
    get_line_colors_for_hover(app, current_view, line, line == hovered_file.trim())
}

fn get_line_colors_for_hover(
    app: &App,
    current_view: &str,
    line: &str,
    is_hovered: bool,
) -> (Color, Color) {
    match app.get_file_type(current_view, line.trim()) {
        FileType::File if is_hovered => (Color::White, Color::Black),
        FileType::File => (Color::default(), Color::LightGreen),
        FileType::Directory if is_hovered => (Color::Blue, Color::Black),
        FileType::Directory => (Color::default(), Color::LightBlue),
        FileType::Unknown => (Color::default(), Color::default()),
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
    let child_view_path = child_view.get_absolute_path();
    if !app.buffer_storage.views.contains_key(&child_view_path) {
        let _ = app.buffer_storage.add_view(child_view_path.clone());
    }
    let child_view_dir = app
        .buffer_storage
        .views
        .get(&child_view_path)
        .map(|view| view.dir.clone())
        .unwrap_or(child_view_path);

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
                    let (bg, fg) = get_line_colors(app, &child_view_dir, line, &hovered_file);

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
                    let (bg, fg) = get_line_colors(app, &child_view_dir, line, &hovered_file);

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
            let file_type_a = app.get_file_type(&child_view_dir, a.to_string().as_str());
            let file_type_b = app.get_file_type(&child_view_dir, b.to_string().as_str());

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

fn wrap_preview_lines<'a>(lines: Vec<Line<'a>>, width: usize) -> Vec<Line<'a>> {
    if width == 0 {
        return lines;
    }

    let mut wrapped = Vec::new();
    for line in lines {
        let mut current_spans = Vec::new();
        let mut current_width = 0;

        for span in line.spans {
            let style = span.style;
            for ch in span.content.chars() {
                let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                if char_width > 0 && current_width + char_width > width {
                    wrapped.push(Line::from(std::mem::take(&mut current_spans)));
                    current_width = 0;
                }

                if ch == '\t' {
                    let spaces = 4 - (current_width % 4);
                    for _ in 0..spaces {
                        if current_width == width {
                            wrapped.push(Line::from(std::mem::take(&mut current_spans)));
                            current_width = 0;
                        }
                        current_spans.push(Span::styled(" ", style));
                        current_width += 1;
                    }
                } else if !ch.is_control() {
                    current_spans.push(Span::styled(ch.to_string(), style));
                    current_width += char_width;
                }
            }
        }

        wrapped.push(Line::from(current_spans));
    }
    wrapped
}
