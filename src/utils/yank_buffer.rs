// Important for pasting. Keeping track of the yank type makes pasting easier
#[derive(Debug)]
pub enum YankType {
    Line,
    Word,
    Char,
}

#[derive(Debug)]
pub struct YankBuffer {
    pub content: String,
    pub yank_type: YankType,
}

impl Default for YankBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl YankBuffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            yank_type: YankType::Line,
        }
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
}
