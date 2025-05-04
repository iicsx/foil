use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathHelper {
    pub current_path: PathBuf,
}

impl PathHelper {
    pub fn new(path: &str) -> Self {
        PathHelper {
            current_path: Path::new(path).to_path_buf(),
        }
    }

    pub fn from_path(path: PathHelper) -> Self {
        PathHelper {
            current_path: path.current_path,
        }
    }

    pub fn get_dir_names(&self) -> io::Result<Vec<fs::DirEntry>> {
        let mut dir_names: Vec<fs::DirEntry> = Vec::new();

        for entry in fs::read_dir(&self.current_path)? {
            let dir = entry?;

            dir_names.push(dir);
        }

        Ok(dir_names)
    }

    pub fn get_path_str(&self) -> String {
        match self.current_path.to_str() {
            Some(path_str) => path_str.to_string(),
            None => "".to_string(),
        }
    }

    pub fn get_dir_names_printable(&self, trim_start: bool) -> Result<Vec<String>, std::io::Error> {
        let mut names = Vec::new();

        for name in self.get_dir_names()? {
            match name.path().into_os_string().into_string() {
                Ok(mut pathname) => {
                    if pathname.starts_with("./") && trim_start {
                        pathname = pathname.trim_start_matches("./").to_string();
                    }

                    names.push(pathname);
                }
                Err(_) => {}
            }
        }

        Ok(names)
    }

    pub fn get_file_name(&self) -> Result<&str, ()> {
        let n = match self.current_path.file_name() {
            Some(n) => n,
            None => return Err(()),
        };

        match n.to_str() {
            Some(res) => Ok(res),
            None => Err(()),
        }
    }

    pub fn get_parent(&mut self) -> Result<String, ()> {
        let parent = match self.current_path.parent() {
            Some(p) => p,
            None => return Err(()),
        };

        let buf = parent.to_path_buf();
        self.current_path = buf.clone();

        match buf.to_str() {
            Some(path_str) => {
                if path_str == "" {
                    return Ok("..".to_string());
                }

                Ok(path_str.to_string())
            }
            None => Err(()),
        }
    }

    pub fn cd(&mut self, path: &str) -> Result<(), ()> {
        let path_str = match self.current_path.as_os_str().to_str() {
            Some(path_str) => path_str,
            None => return Err(()),
        };

        let full_path: String = path_str.to_string() + "/" + path;
        let new_path = Path::new(&full_path);

        if !new_path.exists() {
            return Err(());
        }

        self.current_path = new_path.to_path_buf();
        Ok(())
    }

    pub fn set_path(&mut self, path: &str) -> Result<(), ()> {
        let new_path = Path::new(path);

        if !new_path.exists() {
            return Err(());
        }

        self.current_path = new_path.to_path_buf();
        Ok(())
    }

    pub fn get_file_count(&self) -> Result<usize, std::io::Error> {
        let dir_entries = self.get_dir_names()?;
        let count = dir_entries.len();

        Ok(count)
    }

    pub fn get_line_length(&self, y: u16) -> Result<usize, std::io::Error> {
        let dir_entries = self.get_dir_names()?;
        let count = dir_entries.len();

        if y as usize > count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "y is out of bounds",
            ));
        }

        let entry = &dir_entries[y as usize];
        let name = entry.file_name();

        match name.into_string() {
            Ok(name) => Ok(name.len()),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to convert OsString to String",
            )),
        }
    }
}
