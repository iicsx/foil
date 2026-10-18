use std::path::Path;
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    process,
    sync::OnceLock,
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

use std::os::unix::fs::PermissionsExt;

pub fn get_file_permission(path: &str) -> std::io::Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    let mode = metadata.permissions().mode();

    let mut permissions = String::with_capacity(10);
    permissions.push(if metadata.file_type().is_dir() {
        'd'
    } else {
        '-'
    });

    for (bit, flag) in [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ] {
        permissions.push(if mode & bit != 0 { flag } else { '-' });
    }

    Ok(permissions)
}

pub fn get_file_permissions(path: &Path) -> std::io::Result<String> {
    use std::os::unix::fs::MetadataExt;

    let metadata = fs::symlink_metadata(path)?;
    let mode = metadata.mode();
    let file_type = match mode & libc::S_IFMT {
        libc::S_IFDIR => 'd',
        libc::S_IFLNK => 'l',
        libc::S_IFIFO => 'p',
        libc::S_IFSOCK => 's',
        libc::S_IFCHR => 'c',
        libc::S_IFBLK => 'b',
        libc::S_IFREG => '-',
        _ => '?',
    };

    let mut permissions = String::with_capacity(9);
    for (bit, flag) in [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ] {
        permissions.push(if mode & bit != 0 { flag } else { '-' });
    }
    set_special_permission(&mut permissions, mode, 0o4000, 2, 's', 'S');
    set_special_permission(&mut permissions, mode, 0o2000, 5, 's', 'S');
    set_special_permission(&mut permissions, mode, 0o1000, 8, 't', 'T');

    let owner =
        account_name("/etc/passwd", metadata.uid()).unwrap_or_else(|| metadata.uid().to_string());
    let group =
        account_name("/etc/group", metadata.gid()).unwrap_or_else(|| metadata.gid().to_string());
    let modified = format_local_time(metadata.mtime());

    Ok(format!(
        "{}{} {} {} {} {:>8} {} {}",
        file_type,
        permissions,
        metadata.nlink(),
        owner,
        group,
        human_size(metadata.len()),
        modified,
        path.file_name().unwrap_or_default().to_string_lossy(),
    ))
}

fn set_special_permission(
    permissions: &mut String,
    mode: u32,
    bit: u32,
    index: usize,
    set: char,
    set_without_execute: char,
) {
    if mode & bit != 0 {
        let execute_present = permissions.as_bytes()[index] == b'x';
        permissions.replace_range(
            index..=index,
            if execute_present {
                set.to_string()
            } else {
                set_without_execute.to_string()
            }
            .as_str(),
        );
    }
}

fn account_name(database: &'static str, id: u32) -> Option<String> {
    static PASSWD: OnceLock<HashMap<u32, String>> = OnceLock::new();
    static GROUP: OnceLock<HashMap<u32, String>> = OnceLock::new();
    let entries = if database == "/etc/passwd" {
        PASSWD.get_or_init(|| load_account_names(database, 2))
    } else {
        GROUP.get_or_init(|| load_account_names(database, 2))
    };
    entries.get(&id).cloned()
}

fn load_account_names(path: &str, id_field: usize) -> HashMap<u32, String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(':');
            let name = fields.next()?;
            let id = fields.nth(id_field - 1)?.parse().ok()?;
            Some((id, name.to_string()))
        })
        .collect()
}

fn format_local_time(timestamp: i64) -> String {
    let timestamp = timestamp as libc::time_t;
    let mut broken_down = std::mem::MaybeUninit::<libc::tm>::zeroed();
    let result = unsafe { libc::localtime_r(&timestamp, broken_down.as_mut_ptr()) };
    if result.is_null() {
        return timestamp.to_string();
    }

    let broken_down = unsafe { broken_down.assume_init() };
    let format = b"%Y-%m-%d %H:%M\0";
    let mut output = [0_u8; 32];
    let length = unsafe {
        libc::strftime(
            output.as_mut_ptr().cast(),
            output.len(),
            format.as_ptr().cast(),
            &broken_down,
        )
    };
    if length == 0 {
        timestamp.to_string()
    } else {
        String::from_utf8_lossy(&output[..length]).into_owned()
    }
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
    let path = Path::new(&file_name);
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_dir() {
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(String::new())
}

pub fn rename_file(old_name: String, new_name: String) -> Result<String, std::io::Error> {
    fs::rename(old_name, new_name)?;
    Ok(String::new())
}

pub fn create_file(dir: &str, file_name: String) -> Result<String, std::io::Error> {
    let path = Path::new(dir).join(&file_name);
    if file_name.ends_with("/") {
        fs::create_dir(path)?;
    } else {
        fs::File::create(path)?;
    }
    Ok(String::new())
}

pub fn move_file(
    original_dir: String,
    file_name: String,
    new_dir: String,
) -> Result<String, std::io::Error> {
    let source = Path::new(&original_dir).join(&file_name);
    let destination = Path::new(&new_dir).join(&file_name);
    fs::rename(source, destination)?;
    Ok(String::new())
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
