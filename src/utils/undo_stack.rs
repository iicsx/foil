#[derive(Debug)]
pub struct UndoStack {
    stack: Vec<String>,
    pointers: Vec<(usize, usize)>,
    current_index: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        UndoStack {
            stack: Vec::new(),
            pointers: Vec::new(),
            current_index: 0,
        }
    }

    pub fn push(&mut self, state: String, x: usize, y: usize) {
        self.stack.push(state);
        self.pointers.push((x, y));
        self.current_index = self.stack.len();
    }

    pub fn undo(&mut self) -> Option<String> {
        if self.current_index == 0 {
            return None; // already at oldest change
        }

        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.stack[self.current_index].clone())
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<String> {
        if self.current_index == self.stack.len() - 1 {
            return None; // already at most recent change
        }

        if self.current_index < self.stack.len() {
            let state = &self.stack[self.current_index];
            self.current_index += 1;
            Some(state.clone())
        } else {
            None
        }
    }

    pub fn get_pointers(&self) -> Option<(usize, usize)> {
        let index = self.current_index;

        if index < self.pointers.len() {
            Some(self.pointers[index])
        } else {
            None
        }
    }
}
