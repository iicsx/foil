use crate::file_helper::PathHelper;
use crate::utils::{
    buffer_storage::{BufferStorage, FileEntry, FileType, State},
    cursor::Cursor,
    input_buffer::InputBuffer,
    system,
    undo_stack::UndoStack,
    yank_buffer::YankBuffer,
};
use crossterm::cursor::SetCursorStyle;
use ratatui::{prelude::Rect, widgets::Paragraph};
use std::{
    error, fmt, fs,
    path::{Path, PathBuf},
    rc::Rc,
    result::Result,
};

#[derive(Debug, Default, Clone)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    VisualBlock,
    VisualLine,
    Insert,
    Command,
    Pending,
    Search,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Command => "COMMAND",
            Self::Visual => "VISUAL",
            Self::VisualBlock => "V-BLOCK",
            Self::VisualLine => "V-LINE",
            Self::Pending => "PENDING",
            Self::Search => "SEARCH",
        };

        write!(f, "{}", string)
    }
}

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App<'a> {
    pub running: bool,
    pub mode: Mode,
    pub buffer_content: String,
    pub offset_top: usize,
    pub child_preview: String,
    pub buffer_storage: BufferStorage,
    pub yank_buffer: YankBuffer,
    pub undo_stack: UndoStack,
    pub command: Option<String>,
    pub path: PathHelper,
    pub pending_editor_path: Option<PathBuf>,

    pub layout: Rc<[Rect]>,
    pub parent_pane: Option<Paragraph<'a>>,
    pub current_pane: Option<Paragraph<'a>>,
    pub child_pane: Option<Paragraph<'a>>,

    pub cursor: Cursor,
    pub command_buffer: InputBuffer,

    pub need_confirmation: bool,
    pub rerender_dir_content: bool,
    pub hostname: String,
    pub current_file_permissions: String,
    pub current_file_size: String,
}

impl Default for App<'_> {
    fn default() -> Self {
        let mut buffer_storage = BufferStorage::new();
        let _ = buffer_storage.add_view(system::pwd());

        Self {
            running: true,
            mode: Mode::default(),
            buffer_content: String::from(""),
            offset_top: 0,
            child_preview: String::from(""),
            buffer_storage,
            yank_buffer: YankBuffer::new(),
            undo_stack: UndoStack::new(),
            command: None,
            path: PathHelper::new("./", &system::pwd()),
            pending_editor_path: None,
            layout: Rc::new([]),

            parent_pane: None,
            current_pane: None,
            child_pane: None,

            cursor: Cursor::default(),
            command_buffer: InputBuffer::new(),
            need_confirmation: false,
            rerender_dir_content: true,
            hostname: format!("{}@{}", system::whoami().trim(), system::hostname().trim()),
            current_file_permissions: String::new(),
            current_file_size: String::new(),
        }
    }
}

impl App<'_> {
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
        let cursor_style = match &mode {
            Mode::Pending => SetCursorStyle::SteadyUnderScore,
            Mode::Insert => SetCursorStyle::BlinkingBar,
            _ => SetCursorStyle::SteadyBlock,
        };

        crossterm::execute!(std::io::stdout(), cursor_style)?;
        self.mode = mode;
        Ok(())
    }

    pub fn pop_word(&mut self) {
        let trimmed = self.buffer_content.trim_end();

        match trimmed.rfind(' ') {
            Some(last_space_index) => self.buffer_content.truncate(last_space_index),
            None => self.buffer_content.clear(),
        }
    }

    pub fn append_linebreak(&mut self) {
        self.buffer_content += "\n";
    }

    fn set_buffer_lines(&mut self, lines: Vec<String>) {
        self.buffer_content = lines.join("\n");
    }

    fn sync_filename_edit(&mut self, old_name: &str, new_name: &str, row: usize) {
        if row == 0 || old_name.trim() == new_name.trim() {
            return;
        }
        let current_path = self.path.get_absolute_path();
        let Some(view) = self.buffer_storage.get_view_mut(&current_path) else {
            return;
        };
        let old_name = old_name.trim();
        let new_name = new_name.trim();
        if old_name.is_empty() {
            if !new_name.is_empty() && !view.files.contains_key(new_name) {
                view.add_file(new_name, FileType::File);
            }
        } else {
            view.set_name_for_edit(old_name, new_name);
        }
    }

    pub fn insert_at(&mut self, x: u16, y: u16, content: &str) {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();
        let Some(line) = lines.get_mut(y as usize) else {
            return;
        };

        let Some(byte_x) = char_to_byte_index(line, x as usize) else {
            return;
        };

        let old_name = line.clone();
        line.insert_str(byte_x, content);
        let new_name = line.clone();
        self.set_buffer_lines(lines);
        self.sync_filename_edit(&old_name, &new_name, y as usize);
    }

    pub fn delete_at(&mut self, x: u16, y: u16) {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();
        if y as usize >= lines.len() {
            return;
        }

        let line = &mut lines[y as usize];
        let Some(start) = char_to_byte_index(line, x as usize) else {
            return;
        };
        let Some(end) = char_to_byte_index(line, x as usize + 1) else {
            return;
        };
        let old_name = line.clone();
        line.replace_range(start..end, "");
        let new_name = line.clone();
        self.set_buffer_lines(lines);
        self.sync_filename_edit(&old_name, &new_name, y as usize);
    }

    pub fn delete_range(&mut self, x: u16, y: u16, length: usize) {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();

        if y as usize >= lines.len() {
            return;
        }

        let line = &mut lines[y as usize];

        let Some(start) = char_to_byte_index(line, x as usize) else {
            return;
        };

        let end_char = (x as usize)
            .saturating_add(length)
            .min(line.chars().count());
        let Some(end) = char_to_byte_index(line, end_char) else {
            return;
        };

        let old_name = line.clone();
        line.replace_range(start..end, "");
        let new_name = line.clone();
        self.set_buffer_lines(lines);
        self.sync_filename_edit(&old_name, &new_name, y as usize);
    }

    pub fn delete_line_full(&mut self, y: u16) {
        let mut lines: Vec<&str> = self.buffer_content.lines().collect();

        if y as usize >= lines.len() {
            return;
        }

        let removed = lines.remove(y as usize).trim().to_string();

        self.set_buffer_lines(lines.iter().map(|line| (*line).to_string()).collect());
        if y > 0 && removed != "../" {
            let current_path = self.path.get_absolute_path();
            if let Some(view) = self.buffer_storage.get_view_mut(&current_path) {
                view.delete_file(&removed);
            }
        }
    }

    pub fn delete_line(&mut self, y: u16) {
        let mut lines: Vec<String> = self
            .buffer_content
            .lines()
            .map(|line| line.to_string())
            .collect();

        if y as usize >= lines.len() {
            return;
        }

        let removed = lines[y as usize].clone();
        lines[y as usize].clear();

        self.set_buffer_lines(lines);
        if y > 0 {
            self.sync_filename_edit(&removed, "", y as usize);
        }
    }

    pub fn move_max_x(&mut self) {
        const NEUTRAL_ELEMENT: u16 = 1;

        let new_x = self
            .get_line_length(self.cursor.y - 1)
            .unwrap_or(NEUTRAL_ELEMENT as usize)
            .try_into()
            .unwrap_or(NEUTRAL_ELEMENT);

        self.cursor.x = new_x;
    }

    pub fn move_max_y(&mut self) {
        const NEUTRAL_ELEMENT: u16 = 1;

        let new_y = self.get_line_count().try_into().unwrap_or(NEUTRAL_ELEMENT);

        self.cursor.y = new_y;
    }

    pub fn scroll_down(&mut self) {
        let lines = self.path.get_file_count().unwrap_or(0);
        let offset = self.offset_top;
        let Some(current_pane) = self.layout.get(1) else {
            return;
        };
        let max_height = current_pane.height.saturating_sub(2) as usize;

        if offset + max_height >= lines {
            return;
        }

        self.offset_top += 1;
        self.rerender_dir_content = true;
    }

    pub fn scroll_up(&mut self) {
        if self.offset_top == 0 {
            return;
        }

        self.offset_top -= 1;
        self.rerender_dir_content = true;
    }

    pub fn get_end_x(&self, s: &str, start: usize, inclusive: bool) -> usize {
        let chars = s.chars().collect::<Vec<_>>();
        if start >= chars.len() {
            return chars.len();
        }

        let current_char = chars.get(start.max(1) - 1).copied().unwrap_or('.');
        if !current_char.is_alphanumeric() {
            let mut next_start = start + 1;
            while chars
                .get(next_start)
                .is_some_and(|ch| !ch.is_alphanumeric())
            {
                next_start += 1;
            }

            return next_start;
        }

        let mut end = start + 1;
        while chars.get(end).is_some_and(|ch| ch.is_alphanumeric()) {
            end += 1;
        }

        if inclusive {
            end += 1;
        }

        end
    }

    pub fn get_start_x(&self, s: &str, start: usize) -> usize {
        if start <= 1 {
            return 1;
        }
        let chars = s.chars().collect::<Vec<_>>();
        let mut end = start - 1;

        while end > 1 && chars.get(end - 1).is_some_and(|ch| ch.is_alphanumeric()) {
            end -= 1;
        }

        end
    }

    pub fn delete_word(&mut self, x: u16, y: u16) -> String {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();

        if y as usize >= lines.len() {
            return String::new();
        }

        let line = &mut lines[y as usize];

        if x as usize >= line.chars().count() || line.is_empty() {
            return String::new();
        }

        let end = self.get_end_x(line, x as usize, false);
        let Some(start_byte) = char_to_byte_index(line, x as usize) else {
            return String::new();
        };
        let Some(end_byte) = char_to_byte_index(line, end) else {
            return String::new();
        };

        let old_name = line.clone();
        line.replace_range(start_byte..end_byte, "");
        let new_name = line.clone();
        self.set_buffer_lines(lines);
        self.sync_filename_edit(&old_name, &new_name, y as usize);

        old_name[start_byte..end_byte].to_string()
    }

    pub fn merge_lines(&self, y1: usize, y2: usize) -> Result<String, std::io::Error> {
        let mut lines: Vec<String> = self.buffer_content.lines().map(str::to_owned).collect();

        if y1 >= lines.len() || y2 >= lines.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Line out of bounds",
            ));
        }

        if y2 != y1 + 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Lines to merge must be adjacent",
            ));
        }

        let next_line = lines.remove(y2);
        lines[y1].push_str(&next_line);
        Ok(lines.join("\n"))
    }

    pub fn get_line_count(&self) -> usize {
        self.buffer_content.lines().count()
    }

    pub fn get_character_range(s: &str, start: usize, end: usize) -> Option<&str> {
        let start_byte = char_to_byte_index(s, start)?;
        let end_byte = char_to_byte_index(s, end)?;
        s.get(start_byte..end_byte)
    }

    pub fn current_line(&self) -> &str {
        self.buffer_content
            .lines()
            .nth(self.cursor.y.saturating_sub(1) as usize)
            .unwrap_or("")
    }

    pub fn is_at_end_of_line(&self) -> bool {
        let line_length = self.get_line_length(self.cursor.y - 1).unwrap_or(0);
        self.cursor.x as usize >= line_length
    }

    pub fn get_line_length(&self, y: u16) -> Result<usize, std::io::Error> {
        let line = self.buffer_content.lines().nth(y as usize).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Line out of bounds")
        })?;

        Ok(line.chars().count())
    }

    pub fn get_hovered_filename(&self) -> String {
        let y = (self.cursor.y.max(1) - 1) as usize;
        let line = match self.buffer_content.lines().nth(y) {
            Some(line) => line,
            None => return String::new(),
        };

        if self.cursor.x as usize > line.chars().count() {
            return String::new();
        }

        let filename = &line[0_usize..];

        filename.trim().to_string()
    }

    pub fn seek_whitespace_forward(&self, s: &str, start: usize) -> usize {
        let chars = s.chars().collect::<Vec<_>>();
        let mut end = start;

        while chars.get(end).is_some_and(|ch| ch.is_whitespace()) {
            end += 1;
        }

        end
    }

    pub fn seek_whitespace_backward(&self, s: &str, start: usize) -> usize {
        let chars = s.chars().collect::<Vec<_>>();
        let mut end = start;

        while end > 0 && chars.get(end).is_some_and(|ch| ch.is_whitespace()) {
            end -= 1;
        }

        end
    }

    pub fn seek_special_character_forward(&self, s: &str, start: usize) -> usize {
        let chars = s.chars().collect::<Vec<_>>();
        let mut end = start;

        while chars.get(end).is_some_and(|ch| ch.is_alphanumeric()) {
            end += 1;
        }

        end
    }

    pub fn seek_special_character_backward(&self, s: &str, start: usize) -> usize {
        let chars = s.chars().collect::<Vec<_>>();
        let mut end = start;

        while end > 0 && chars.get(end).is_some_and(|ch| ch.is_alphanumeric()) {
            end -= 1;
        }

        // make match exclusive
        if !chars.get(end).is_some_and(|ch| ch.is_alphanumeric()) {
            end += 1;
        }

        end
    }

    // both of the following hurt me to re-implement here but it's necessary
    // to update the buffer content
    pub fn undo(&mut self) {
        if let Some(undo) = self.undo_stack.undo() {
            self.buffer_content = undo;
            if let Some((x, y)) = self.undo_stack.get_pointers() {
                self.cursor.x = x as u16;
                self.cursor.y = y as u16;
            }
            self.sync_visible_file_names();
        }
    }

    pub fn redo(&mut self) {
        if let Some(redo) = self.undo_stack.redo() {
            self.buffer_content = redo;
            if let Some((x, y)) = self.undo_stack.get_pointers() {
                self.cursor.x = x as u16;
                self.cursor.y = y as u16;
            }
            self.sync_visible_file_names();
        }
    }

    fn sync_visible_file_names(&mut self) {
        let path = self.path.get_absolute_path();
        let Some(view) = self.buffer_storage.get_view_mut(&path) else {
            return;
        };
        let lines = self.buffer_content.lines().collect::<Vec<_>>();
        for (index, name) in lines.iter().enumerate().skip(1) {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let key = view
                .files
                .iter()
                .find(|(_, file)| file.name.trim() == name || file.original_name == name)
                .map(|(key, _)| key.clone());
            if let Some(key) = key {
                view.set_name(&key, name);
            } else if index > 0 && !view.files.values().any(|file| file.name.trim() == name) {
                view.add_file(name, FileType::File);
            }
        }
    }

    pub fn get_files(&self, state: State) -> Vec<FileEntry> {
        self.buffer_storage
            .views
            .values()
            .flat_map(|view| view.files.values())
            .filter(|file| file.state == state)
            .cloned()
            .collect()
    }

    pub fn save(&mut self) {
        if self.buffer_storage.has_changes() {
            self.need_confirmation = true;
        }
    }

    pub fn get_file_type(&self, view: &str, filename: &str) -> FileType {
        if let Some(file) = self
            .buffer_storage
            .views
            .get(view)
            .and_then(|view| view.files.get(filename))
        {
            return file.file_type.clone();
        }

        let path = Path::new(view).join(filename);
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => FileType::File,
            Ok(metadata) if metadata.is_dir() => FileType::Directory,
            _ => FileType::Unknown,
        }
    }
}

fn char_to_byte_index(text: &str, char_index: usize) -> Option<usize> {
    if char_index == text.chars().count() {
        Some(text.len())
    } else {
        text.char_indices()
            .nth(char_index)
            .map(|(byte_index, _)| byte_index)
    }
}
