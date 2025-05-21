use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathHelper {
    absolute_path: String,
    pub current_path: PathBuf,
}

impl PathHelper {
    pub fn new(path: &str, pwd: &str) -> Self {
        PathHelper {
            current_path: Path::new(path).to_path_buf(),
            absolute_path: pwd.to_string(),
        }
    }

    pub fn from_path(path: PathHelper) -> Self {
        PathHelper {
            current_path: path.current_path,
            absolute_path: path.absolute_path,
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

    pub fn get_file_name(&self) -> Option<&str> {
        let n = match self.current_path.file_name() {
            Some(n) => n,
            None => return None,
        };

        match n.to_str() {
            Some(res) => Some(res),
            None => None,
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

    pub fn sim_cd(&mut self, path: &str) -> Result<String, ()> {
        let path_str = match self.current_path.as_os_str().to_str() {
            Some(path_str) => path_str,
            None => return Err(()),
        };

        let full_path: String = path_str.to_string() + "/" + path;
        let new_path = Path::new(&full_path);

        if !new_path.exists() {
            return Err(());
        }

        let new_path_str = match new_path.to_str() {
            Some(path_str) => path_str,
            None => return Err(()),
        };

        Ok(new_path_str.to_string())
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

        if y as usize >= count {
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

    pub fn get_absolute_path(&self) -> String {
        let mut temp = self
            .current_path
            .to_str()
            .unwrap_or("")
            .to_string()
            .trim_start_matches("./")
            .to_string();
        let mut absolute_path = self.absolute_path.clone();

        while temp.starts_with("../") || temp == ".." {
            let last_slash = absolute_path.rfind('/').unwrap_or(temp.len());
            absolute_path = absolute_path[0..last_slash].to_string();
            temp = temp[temp.len().min(3)..].to_string();
        }

        let temp = temp.trim_start_matches("./").trim_start_matches(".");

        return match temp.len() > 0 {
            true => format!("{}/{}", absolute_path, &temp),
            false => absolute_path,
        };
    }

    pub fn trim_path(path: &String) -> String {
        let parts = path.split('/').collect::<Vec<&str>>();
        let file_name = parts.last().unwrap_or(&"");

        file_name.to_string()
    }
}
