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
            if let Ok(mut pathname) = name.path().into_os_string().into_string() {
                if pathname.starts_with("./") && trim_start {
                    pathname = pathname.trim_start_matches("./").to_string();
                }

                names.push(pathname);
            }
        }

        Ok(names)
    }

    pub fn get_dir_names_trimmed(&self) -> Result<Vec<String>, std::io::Error> {
        let mut names = Vec::new();

        for name in self.get_dir_names()? {
            if let Ok(pathname) = name.path().into_os_string().into_string() {
                let trimmed = pathname
                    .trim_start_matches("./")
                    .to_string()
                    .trim_start_matches("../")
                    .to_string();

                names.push(trimmed);
            }
        }

        Ok(names)
    }

    pub fn get_file_name(&self) -> Option<&str> {
        let n = self.current_path.file_name()?;

        n.to_str()
    }

    pub fn get_parent(&mut self) -> Option<PathHelper> {
        let mut path = self.clone();
        path.cd("..").map(|_| path)
    }

    pub fn cd(&mut self, path: &str) -> Option<()> {
        if self.get_absolute_path().is_empty() {
            return None;
        }

        let path = match path.starts_with("/") {
            true => path.trim_start_matches("/"),
            false => path,
        };

        let path_str = self.current_path.as_os_str().to_str()?;

        let full_path: String = if path_str.ends_with("..") || path_str.ends_with("../") {
            match path_str.ends_with("/") {
                true => path_str.to_string() + path,
                false => path_str.to_string() + "/" + path,
            }
        } else {
            match path_str.ends_with("/") {
                false => {
                    if path_str != "." && (path == ".." || path == "../") {
                        let last_slash = path_str.rfind('/').unwrap_or(path_str.len());
                        path_str[0..last_slash].to_string()
                    } else {
                        path_str.to_string() + "/" + path
                    }
                }
                true => {
                    if path_str != "." && (path == ".." || path == "../") {
                        let last_slash = path_str.rfind('/').unwrap_or(path_str.len());
                        path_str[0..last_slash].to_string()
                    } else {
                        path_str.to_string() + path
                    }
                }
            }
        };

        let new_path = Path::new(&full_path);

        if !new_path.exists() {
            return None;
        }

        self.current_path = new_path.to_path_buf();
        Some(())
    }

    pub fn sim_cd(&mut self, path: &str) -> Option<String> {
        let path_str = self.current_path.as_os_str().to_str()?;
        let full_path = path_str.to_string() + "/" + path;

        let new_path = Path::new(&full_path);
        if !new_path.exists() {
            return None;
        }

        Some(new_path.to_str()?.to_string())
    }

    pub fn set_path(&mut self, path: &str) -> Option<()> {
        let new_path = Path::new(path);

        if !new_path.exists() {
            return None;
        }

        self.current_path = new_path.to_path_buf();
        Some(())
    }

    pub fn get_file_count(&self) -> Result<usize, std::io::Error> {
        let dir_entries = self.get_dir_names()?;
        let count = dir_entries.len();

        Ok(count)
    }

    pub fn get_line_length(&self, y: u16) -> Result<usize, std::io::Error> {
        let dir_entries = self.get_dir_names()?;
        let count = dir_entries.len();

        if usize::from(y) >= count {
            return Err(io::Error::other("y is out of bounds"));
        }

        let entry = &dir_entries[usize::from(y)];
        let name = entry.file_name();

        match name.into_string() {
            Ok(name) => Ok(name.len()),
            Err(_) => Err(io::Error::other("Failed to convert OsString to String")),
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

        let temp = temp.trim_start_matches("./");

        match !temp.is_empty() && temp != "." {
            true => format!("{}/{}", absolute_path, &temp),
            false => absolute_path,
        }
    }

    pub fn trim_path(path: &str) -> String {
        let parts = path.split('/').collect::<Vec<&str>>();
        let file_name = parts.last().unwrap_or(&"");

        file_name.to_string()
    }
}
