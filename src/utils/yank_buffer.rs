// Important for pasting. Keeping track of the yank type makes pasting easier
#[derive(Debug)]
pub enum YankType {
    Char,
    Line,
    Word,
}

impl Default for YankType {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Debug, Default)]
pub struct YankBuffer {
    pub content: String,
    pub yank_type: YankType,
    pub original_dir: Option<String>,
}
impl YankBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn set_yank_type(&mut self, yank_type: YankType) {
        self.yank_type = yank_type;
    }

    pub fn get_yank_type(&self) -> &YankType {
        &self.yank_type
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn set_original_dir(&mut self, dir: String) {
        self.original_dir = Some(dir);
    }
}
