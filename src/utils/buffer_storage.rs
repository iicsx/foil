use crate::utils::system;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Created,
    Modified,
    Deleted,
    Moved,
    Unmodified,
}

#[allow(dead_code)]
fn get_file_type(path: &str) -> FileType {
    let metadata = fs::metadata(path);

    match metadata {
        Ok(meta) => {
            if meta.is_file() {
                FileType::File
            } else if meta.is_dir() {
                FileType::Directory
            } else {
                FileType::Unknown
            }
        }
        Err(_) => FileType::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub original_name: String,
    pub name: String,
    original_dir: String,
    pub dir: String,
    pub state: State,
    pub file_type: FileType,
}

#[derive(Debug, Clone)]
pub struct DirBuffer {
    pub dir: String,
    pub files: HashMap<String, FileEntry>,
}

impl DirBuffer {
    pub fn new(dir: &str) -> Result<Self, std::io::Error> {
        let mut files = HashMap::new();

        let dir = if dir.starts_with('/') {
            dir.to_string()
        } else if dir == "." {
            system::pwd()
        } else {
            format!("{}/{}", system::pwd(), dir)
        };

        let entries = fs::read_dir(&dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() || path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();

                files.insert(
                    name.clone(),
                    FileEntry {
                        original_name: name.clone(),
                        name,
                        dir: dir.clone(),
                        original_dir: dir.clone(),
                        state: State::Unmodified,
                        file_type: get_file_type(&path.to_string_lossy()),
                    },
                );
            }
        }

        Ok(DirBuffer {
            dir: String::from(dir),
            files,
        })
    }

    pub fn from_raw(raw: &str) -> Self {
        let mut files = HashMap::new();
        let lines = raw.lines();

        for line in lines {
            let trimmed = line.trim();

            if !trimmed.is_empty() {
                let name = trimmed.to_string();
                files.insert(
                    name.clone(),
                    FileEntry {
                        original_name: name.clone(),
                        name,
                        dir: String::new(),
                        original_dir: String::new(),
                        state: State::Unmodified,
                        file_type: get_file_type(trimmed),
                    },
                );
            }
        }

        DirBuffer {
            dir: String::new(),
            files,
        }
    }

    pub fn into_raw(&self) -> String {
        let mut raw = String::new();

        for file in self.files.values() {
            raw.push_str(&file.name);
            raw.push('\n');
        }

        raw
    }

    // filenames must be unique per directory so this should work
    pub fn set_state(&mut self, name: &str, state: State) {
        if let Some((_, file)) = self.files.iter_mut().find(|(_, file)| file.name == name) {
            file.state = state;
        }
    }

    pub fn set_name(&mut self, name: &str, new_name: &str) {
        if let Some((_, mut file)) = self.files.remove_entry(name) {
            file.name = new_name.to_string();

            if file.state == State::Created {
                self.files.insert(new_name.to_string(), file);
                return;
            }

            if file.original_name == name {
                file.state = State::Unmodified;
            } else {
                file.state = State::Modified;
            }

            self.files.insert(new_name.to_string(), file);
        }
    }

    pub fn set_path(&mut self, name: &str, path: &str) {
        if let Some(file) = self.files.get_mut(name) {
            file.dir = path.to_string();
            file.state = State::Modified;

            if file.original_dir == path {
                file.state = State::Unmodified;
            } else {
                file.state = State::Moved;
            }
        }
    }

    pub fn delete_file(&mut self, name: &str) {
        if let Some(file) = self.files.get_mut(name) {
            file.state = State::Deleted;
        }
    }

    pub fn get_files_by_state(&self, state: State) -> Vec<FileEntry> {
        self.files
            .values()
            .filter(|file| file.state == state)
            .cloned()
            .collect()
    }

    pub fn add_file(&mut self, name: &str, file_type: FileType) {
        let file = FileEntry {
            original_name: name.to_string(),
            name: name.to_string(),
            original_dir: self.dir.clone(),
            dir: self.dir.clone(),
            state: State::Created,
            file_type,
        };
        self.files.insert(name.to_string(), file);
    }

    pub fn set_dir(&mut self, name: &str, dir: &str) {
        if let Some(file) = self.files.get_mut(name) {
            file.dir = dir.to_string();
            file.state = State::Modified;
        }
    }

    // these two might be unused tbh
    pub fn get_file_move_dirs(&self, name: &str) -> Option<(String, String)> {
        if let Some(file) = self.files.get(name) {
            Some((file.original_dir.clone(), file.dir.clone()))
        } else {
            None
        }
    }
    pub fn get_rename(&self, name: &str) -> Option<(String, String)> {
        if let Some(file) = self.files.get(name) {
            Some((file.original_name.clone(), file.name.clone()))
        } else {
            None
        }
    }

    pub fn get_file(&self, name: &str) -> Option<FileEntry> {
        self.files.get(name).cloned()
    }
}

#[derive(Debug)]
pub struct BufferStorage {
    pub views: HashMap<String, DirBuffer>,
}

impl BufferStorage {
    pub fn new() -> Self {
        BufferStorage {
            views: HashMap::new(),
        }
    }

    pub fn add_view(&mut self, dir: String) -> Result<(), std::io::Error> {
        if let Some(_) = self.get_view(&dir) {
            return Ok(());
        }

        let buffer = DirBuffer::new(&dir)?;
        self.views.insert(dir, buffer);

        Ok(())
    }

    pub fn add_view_raw(&mut self, dir: String, raw: &str) {
        let buffer = DirBuffer::from_raw(raw);
        self.views.insert(dir, buffer);
    }

    pub fn get_view(&self, dir: &str) -> Option<DirBuffer> {
        self.views.get(dir).cloned()
    }

    pub fn update_view(&mut self, dir: &str, buffer: DirBuffer) {
        self.views.insert(dir.to_string(), buffer);
    }

    pub fn has_changes(&self) -> bool {
        for buffer in self.views.values() {
            for file in buffer.files.values() {
                if file.state != State::Unmodified {
                    return true;
                }
            }
        }

        false
    }
}
