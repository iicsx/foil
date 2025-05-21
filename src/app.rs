use crate::file_helper::PathHelper;
use crate::utils::{
    buffer_storage::{BufferStorage, FileEntry, FileType, State},
    cursor::Cursor,
    input_buffer::InputBuffer,
    system,
    undo_stack::UndoStack,
    yank_buffer::YankBuffer,
};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::widgets::Paragraph;
use std::{error, fmt, result::Result};

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
    pub buffer_storage: BufferStorage,
    pub yank_buffer: YankBuffer,
    pub undo_stack: UndoStack,
    pub command: Option<String>,
    pub path: PathHelper,

    pub parent_pane: Option<Paragraph<'a>>,
    pub current_pane: Option<Paragraph<'a>>,
    pub child_pane: Option<Paragraph<'a>>,

    pub cursor: Cursor,
    pub command_buffer: InputBuffer,

    pub need_confirmation: bool,
    pub rerender_dir_content: bool,
}

impl Default for App<'_> {
    fn default() -> Self {
        let mut buffer_storage = BufferStorage::new();
        let _ = buffer_storage.add_view(system::pwd());

        Self {
            running: true,
            mode: Mode::default(),
            buffer_content: String::from(""),
            buffer_storage: buffer_storage,
            yank_buffer: YankBuffer::new(),
            undo_stack: UndoStack::new(),
            command: None,
            path: PathHelper::new("./", &system::pwd()),

            parent_pane: None,
            current_pane: None,
            child_pane: None,

            cursor: Cursor::default(),
            command_buffer: InputBuffer::new(),
            need_confirmation: false,
            rerender_dir_content: true,
        }
    }
}

#[allow(dead_code)]
impl App<'_> {
    fn new(mode: Mode, buffer_content: String, path: &str) -> Self {
        let mut undo_stack = UndoStack::new();
        undo_stack.push(buffer_content.clone(), 0, 0);

        let mut buffer_storage = BufferStorage::new();
        let _ = buffer_storage.add_view(path.to_string());

        Self {
            running: true,
            command: None,
            mode,
            buffer_content: buffer_content.clone(),
            buffer_storage: buffer_storage,
            yank_buffer: YankBuffer::new(),
            undo_stack: undo_stack,
            path: PathHelper::new(path, &system::pwd()),

            parent_pane: None,
            current_pane: None,
            child_pane: None,

            cursor: Cursor::default(),
            command_buffer: InputBuffer::new(),
            need_confirmation: false,
            rerender_dir_content: true,
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
        self.mode = mode.clone();

        match mode {
            Mode::Normal => execute!(std::io::stdout(), SetCursorStyle::SteadyBlock)?,
            Mode::Visual => execute!(std::io::stdout(), SetCursorStyle::SteadyBlock)?,
            Mode::Pending => execute!(std::io::stdout(), SetCursorStyle::SteadyUnderScore)?,
            Mode::Insert => execute!(std::io::stdout(), SetCursorStyle::BlinkingBar)?,
            _ => execute!(std::io::stdout(), SetCursorStyle::SteadyBlock)?,
        };

        Ok(())
    }

    pub fn pop_word(&mut self) {
        let trimmed = self.buffer_content.trim_end();

        match trimmed.rfind(' ') {
            Some(last_space_index) => {
                self.buffer_content.truncate(last_space_index);
            }
            None => {
                self.buffer_content.clear();
            }
        }
    }

    pub fn append_linebreak(&mut self) {
        self.buffer_content += "\n";
    }

    pub fn insert_at(&mut self, x: u16, y: u16, content: &str) {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();

        let line = &mut lines[y as usize];

        if x as usize > line.len() {
            return;
        }

        let identifier = line.clone();

        line.insert_str(x as usize, content);

        let view = self.buffer_storage.get_view(&self.path.get_absolute_path());
        match view {
            Some(mut view) => {
                if identifier.trim().is_empty() {
                    view.add_file(line, FileType::File);
                } else {
                    view.set_name(&identifier, line);
                }

                self.buffer_storage
                    .update_view(&self.path.get_absolute_path(), view);
            }
            _ => {}
        }

        self.buffer_content = lines.join("\n");
    }

    pub fn delete_at(&mut self, x: u16, y: u16) {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();
        if y as usize >= lines.len() {
            return;
        }

        let line = &mut lines[y as usize];
        if x as usize >= line.len() {
            return;
        }
        let identifier = line.clone();

        line.remove(x as usize);

        let view = self.buffer_storage.get_view(&self.path.get_absolute_path());
        match view {
            Some(mut view) => {
                view.set_name(&identifier, line);
                self.buffer_storage
                    .update_view(&self.path.get_absolute_path(), view);
            }
            _ => {}
        }

        self.buffer_content = lines.join("\n");
    }

    pub fn delete_range(&mut self, x: u16, y: u16, length: usize) {
        let mut lines: Vec<String> = self.buffer_content.lines().map(String::from).collect();

        if y as usize >= lines.len() {
            return;
        }

        let line = &mut lines[y as usize];

        if x as usize >= line.len() {
            return;
        }

        let end = (x + length as u16).min(line.len() as u16);

        line.replace_range(x as usize..end as usize, "");

        self.buffer_content = lines.join("\n");
    }

    pub fn delete_line_full(&mut self, y: u16) {
        let mut lines: Vec<&str> = self.buffer_content.lines().collect();

        if y as usize >= lines.len() {
            return;
        }

        lines.remove(y as usize);

        self.buffer_content = lines.join("\n");
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

        lines[y as usize].clear();

        self.buffer_content = lines.join("\n");
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

    pub fn get_end_x(&self, s: &String, start: usize, inclusive: bool) -> usize {
        let current_char = s.chars().nth(start.max(1) - 1).unwrap_or('.');
        if !current_char.is_alphanumeric() {
            return start + 1;
        }

        let mut end = start;
        while end < s.len() && s.chars().nth(end).unwrap_or(' ').is_alphanumeric() {
            if s.chars().nth(end).unwrap() == ' ' {
                break;
            }
            end += 1;
        }

        if inclusive {
            end += 1;
        }

        end
    }

    pub fn get_start_x(&self, s: &String, start: usize) -> usize {
        if start <= 1 {
            return 1;
        }
        let mut end = start - 1;

        while end > 1 && s.chars().nth(end - 1).unwrap_or(' ').is_alphanumeric() {
            end -= 1;
        }

        end
    }

    pub fn delete_word(&mut self, x: u16, y: u16) {
        let mut lines: Vec<String> = self
            .buffer_content
            .lines()
            .map(|line| String::from(line))
            .collect();

        if y as usize >= lines.len() {
            return;
        }

        let line = &mut lines[y as usize];

        if x as usize >= line.len() || line.is_empty() {
            return;
        }

        let end = self.get_end_x(line, x as usize, false);

        line.replace_range(x as usize..end, "");

        self.buffer_content = lines.join("\n");
    }

    pub fn merge_lines(&self, y1: usize, y2: usize) -> Result<String, std::io::Error> {
        let lines: Vec<&str> = self.buffer_content.lines().collect();

        if y1 >= lines.len() || y2 >= lines.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Line out of bounds",
            ));
        }

        let line1 = lines[y1];
        let line2 = lines[y2];

        let mut new_buffer_content = self.buffer_content.clone();
        // [FIXME]
        new_buffer_content.replace_range(
            new_buffer_content.find(line1).unwrap()..new_buffer_content.find(line2).unwrap(),
            &line1, // idk why but this works
        );

        Ok(new_buffer_content)
    }

    pub fn get_line_count(&self) -> usize {
        self.buffer_content.lines().count()
    }

    pub fn get_line_length(&self, y: u16) -> Result<usize, std::io::Error> {
        let lines: Vec<&str> = self.buffer_content.lines().collect();

        if y as usize >= lines.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Line out of bounds",
            ));
        }

        let line = lines[y as usize];

        Ok(line.chars().count())
    }

    pub fn get_hovered_filename(&self) -> String {
        let lines: Vec<&str> = self.buffer_content.lines().collect();

        if self.cursor.y as usize > lines.len() {
            return String::new();
        }

        let y = (self.cursor.y.max(1) - 1) as usize;
        let line = lines[y as usize];

        if self.cursor.x as usize > line.chars().count() {
            return String::new();
        }

        let filename = &line[0 as usize..];

        filename.to_string()
    }

    pub fn seek_whitespace_forward(&self, s: &String, start: usize) -> usize {
        let mut end = start;

        while end < s.len() && s.chars().nth(end).unwrap_or(' ').is_whitespace() {
            end += 1;
        }

        end
    }

    pub fn seek_whitespace_backward(&self, s: &String, start: usize) -> usize {
        let mut end = start;

        while end > 0 && s.chars().nth(end).unwrap_or(' ').is_whitespace() {
            end -= 1;
        }

        end
    }

    pub fn seek_special_character_forward(&self, s: &String, start: usize) -> usize {
        let mut end = start;

        while end < s.len() && s.chars().nth(end).unwrap_or(' ').is_alphanumeric() {
            end += 1;
        }

        end
    }

    pub fn seek_special_character_backward(&self, s: &String, start: usize) -> usize {
        let mut end = start;

        while end > 0 && s.chars().nth(end).unwrap_or(' ').is_alphanumeric() {
            end -= 1;
        }

        // make match exclusive
        if !s.chars().nth(end).unwrap_or(' ').is_alphanumeric() {
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
        }
    }

    pub fn redo(&mut self) {
        if let Some(redo) = self.undo_stack.redo() {
            self.buffer_content = redo;
        }
    }

    pub fn get_files(&self, state: State) -> Vec<FileEntry> {
        let views = self.buffer_storage.views.clone();
        let mut files = Vec::new();

        for (_, view) in views {
            let files_view = view.get_files_by_state(state.clone());
            for file in files_view {
                files.push(file.clone());
            }
        }

        files
    }

    pub fn save(&mut self) {
        if self.buffer_storage.has_changes() {
            self.need_confirmation = true;
        }
    }
}
