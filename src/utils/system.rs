use std::path::Path;
use std::{
    fs,
    io::{BufRead, BufReader},
    process,
};

pub fn whoami() -> String {
    let output = std::process::Command::new("whoami").output();

    match output {
        Ok(output) => {
            let user = String::from_utf8_lossy(&output.stdout);

            String::from(user)
        }
        Err(_) => {
            process::exit(1);
        }
    }
}

pub fn hostname() -> String {
    let output = std::process::Command::new("hostname").output();

    match output {
        Ok(output) => String::from(String::from_utf8_lossy(&output.stdout)),
        Err(_) => match try_get_hostnamectl() {
            Some(hostname) => hostname,
            None => process::exit(1),
        },
    }
}

fn try_get_hostnamectl() -> Option<String> {
    let output = std::process::Command::new("hostnamectl").output();

    match output {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut hostname = None;

            for line in text.lines() {
                if !line.trim().starts_with("Static hostname:") {
                    continue;
                }

                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    hostname = Some(parts[1].trim().to_string());
                }
            }

            hostname
        }
        Err(_) => None,
    }
}

pub fn pwd() -> String {
    let output = std::process::Command::new("pwd").output();

    match output {
        Ok(output) => {
            let path = String::from_utf8_lossy(&output.stdout);

            String::from(path.trim())
        }
        Err(_) => {
            process::exit(1);
        }
    }
}

pub fn get_file_permissions() -> String {
    let output = std::process::Command::new("ls")
        .arg("-l")
        .arg(".")
        .output()
        .expect("Failed to execute command");

    let permissions = String::from_utf8_lossy(&output.stdout);

    permissions.trim().to_string()
}

pub fn get_file_permission(file_name: String) -> String {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(file_name)
        .map(|metadata| format!("{:o}", metadata.permissions().mode() & 0o7777))
        .unwrap_or_default()
}

pub fn get_file_size(filename: String) -> String {
    fs::metadata(filename)
        .map(|metadata| human_size(metadata.len()))
        .unwrap_or_default()
}

pub fn is_dir(path: String) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{}{}", bytes, UNITS[unit])
    } else {
        format!("{:.1}{}", size, UNITS[unit])
    }
}

pub fn delete_file(file_name: String) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("rm")
        .arg("-rf")
        .arg(file_name)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    Ok(result.trim().to_string())
}

pub fn rename_file(old_name: String, new_name: String) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("mv")
        .arg(old_name)
        .arg(new_name)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    Ok(result.trim().to_string())
}

pub fn create_file(dir: &str, file_name: String) -> Result<String, std::io::Error> {
    let cmd = if file_name.ends_with("/") {
        "mkdir"
    } else {
        "touch"
    };

    let full_path = format!("{}/{}", dir, file_name);

    let output = std::process::Command::new(cmd).arg(full_path).output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    Ok(result.trim().to_string())
}

pub fn move_file(
    original_dir: String,
    file_name: String,
    new_dir: String,
) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("mv")
        .arg(format!("{}/{}", original_dir, file_name))
        .arg(new_dir)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    Ok(result.trim().to_string())
}

pub fn get_file_preview(file_name: String, max_lines: usize) -> Result<String, std::io::Error> {
    let file = fs::File::open(file_name)?;
    let preview = BufReader::new(file)
        .lines()
        .take(max_lines)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    if preview.is_empty() {
        return Err(std::io::Error::other("File is empty"));
    }
    Ok(preview)
}

pub fn get_dir_preview(dir_name: String) -> Result<String, std::io::Error> {
    let mut entries = fs::read_dir(Path::new(&dir_name))?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    entries.sort();
    Ok(entries.join("\n"))
}
