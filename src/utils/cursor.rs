use ratatui::layout::Rect;
use ratatui::prelude::Position;

#[derive(Debug)]
pub struct Cursor {
    pub container: Option<Rect>,
    pub x: u16,
    pub y: u16,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            container: None,
            x: 1,
            y: 1,
        }
    }
}

impl Cursor {
    pub fn new(container: Rect) -> Self {
        Self {
            container: Some(container),
            // defaults to 1 because of borders, this might need a change
            // since we also need to account for these borders whenever we calculate literally anything
            x: 1,
            y: 1,
        }
    }

    pub fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_x(&mut self, x: u16) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: u16) {
        self.y = y;
    }

    pub fn set_position(&mut self, pos: Position) {
        self.x = pos.x;
        self.y = pos.y;
    }

    pub fn up(&mut self) {
        match self.container {
            Some(container) => {
                let new_y = container.y - 1 + self.y - 1;

                if new_y >= container.y {
                    self.y -= 1;
                }
            }
            None => self.y -= 1,
        };
    }

    pub fn down(&mut self) {
        match self.container {
            Some(_) => {
                self.y += 1;
            }
            None => self.y += 1,
        };
    }

    pub fn left(&mut self) {
        match self.container {
            Some(container) => {
                let new_x = container.x - 1 + self.x - 1;

                if new_x >= container.x {
                    self.x -= 1;
                }
            }
            None => self.x -= 1,
        };
    }

    pub fn right(&mut self, constraint: u16) {
        match self.container {
            Some(container) => {
                let new_x = container.x - 1 + self.x + 1;
                let within_bounds = (self.x + 1) <= constraint;

                if constraint > 0 && !within_bounds {
                    return;
                }

                if new_x >= container.x {
                    self.x += 1;
                }
            }
            None => self.x += 1,
        };
    }

    pub fn reset_x(&mut self) {
        self.x = 1;
    }

    pub fn reset_y(&mut self) {
        self.y = 1;
    }

    pub fn update_frame(&mut self, frame: &mut ratatui::Frame) {
        if let Some(container) = self.container {
            let position = Position {
                x: container.x + self.x,
                y: container.y + self.y,
            };
            frame.set_cursor_position(position);
        } else {
            panic!("Cursor container is not set");
        }
    }

    pub fn move_word(&mut self, line: &str, new_x: usize) {
        if new_x > line.len() {
            self.x = line.len() as u16;
        } else {
            self.x = new_x as u16;
        }
    }
}
