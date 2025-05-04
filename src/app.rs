use std::error;
use std::fmt;
use std::fs;

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
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
            Self::Pending => "PENDING",
        };

        write!(f, "{}", string)
    }
}

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub mode: Mode,
    pub buffer_content: String,
    pub command: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            mode: Mode::default(),
            buffer_content: String::from(""),
            command: None,
        }
    }
}

impl App {
    fn new(mode: Mode, buffer_content: String) -> Self {
        Self {
            running: true,
            command: None,
            mode,
            buffer_content,
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }

    pub fn load_new_buffer(&mut self, path: &str) {
        let file_buffer = fs::read_to_string(String::from(path));

        let content = match file_buffer {
            Ok(content) => content,
            Err(_) => panic!("[FIXME] File Buffer could not be read"),
        };

        self.buffer_content = content
    }

    pub fn append_to_buffer(&mut self, content: &str) {
        self.buffer_content += content
    }

    pub fn pop_character(&mut self) {
        self.buffer_content.pop();
    }

    pub fn pop_word(&mut self) {
        let trimmed = self.buffer_content.trim_end();

        if let Some(last_space_index) = trimmed.rfind(' ') {
            self.buffer_content.truncate(last_space_index);
        } else {
            self.buffer_content.clear();
        }
    }

    pub fn append_linebreak(&mut self) {
        self.buffer_content += "\n";
    }
}
