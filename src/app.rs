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
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Command => "COMMAND",
            Self::Visual => "VISUAL",
        };

        write!(f, "{}", string)
    }
}

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub mode: Mode,

    pub buffer_content: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            mode: Mode::default(),
            buffer_content: String::from(""),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
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
}
