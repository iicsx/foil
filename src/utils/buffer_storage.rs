use std::fs;

pub enum BufferType {
    File,
    Directory,
    Unknown,
}

fn get_file_type(path: &str) -> BufferType {
    let metadata = fs::metadata(path);

    match metadata {
        Ok(meta) => {
            if meta.is_file() {
                BufferType::File
            } else if meta.is_dir() {
                BufferType::Directory
            } else {
                BufferType::Unknown
            }
        }
        Err(_) => BufferType::Unknown,
    }
}

pub struct DirBuffer {
    pub dir: String,
    pub files: Vec<String>,
}

impl DirBuffer {
    pub fn new(dir: String) -> Result<Self, std::io::Error> {
        let mut files = Vec::new();
        let entries = fs::read_dir(&dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                files.push(path.to_string_lossy().to_string());
            }
        }

        Ok(DirBuffer { dir, files })
    }

    pub fn into_raw(&self) -> String {
        let mut raw = String::new();
        for file in &self.files {
            raw.push_str(file);
            raw.push('\n');
        }
        raw
    }
}

pub struct BufferStorage {
    pub views: Vec<DirBuffer>,
}

impl BufferStorage {
    pub fn new() -> Self {
        BufferStorage { views: Vec::new() }
    }
}
