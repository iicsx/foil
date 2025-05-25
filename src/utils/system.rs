use std::process;

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
    let output = std::process::Command::new("ls")
        .arg("-l")
        .arg(file_name)
        .output()
        .expect("Failed to execute command");

    let permissions = String::from_utf8_lossy(&output.stdout);

    let mut res = String::new();
    for line in permissions.lines() {
        if line.trim().starts_with("total") {
            continue;
        }

        let string: Vec<&str> = line.split(" ").collect();

        res = string[0].trim().to_string()
    }

    res
}

pub fn get_file_size(filename: String) -> String {
    let output = std::process::Command::new("du")
        .arg("-sh")
        .arg(filename)
        .output()
        .expect("Failed to execute command");

    let size = String::from_utf8_lossy(&output.stdout);

    let mut res = String::new();
    for line in size.lines() {
        if line.trim().starts_with("total") {
            continue;
        }

        let string: Vec<&str> = line.split("\t").collect();

        res = string[0].trim().to_string()
    }

    res
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

pub fn create_file(file_name: String) -> Result<String, std::io::Error> {
    let cmd = if file_name.ends_with("/") {
        "mkdir"
    } else {
        "touch"
    };

    let output = std::process::Command::new(cmd).arg(file_name).output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    Ok(result.trim().to_string())
}

pub fn move_file(file_name: String, new_dir: String) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("mv")
        .arg(file_name)
        .arg(new_dir)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    Ok(result.trim().to_string())
}

pub fn get_file_preview(file_name: String, max_lines: usize) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("head")
        .arg("-n")
        .arg(max_lines.to_string())
        .arg(file_name)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.is_empty() || result.starts_with("head: error") {
        return Err(std::io::Error::other("File not found"));
    }

    Ok(result.trim().to_string())
}

pub fn get_dir_preview(dir_name: String) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("ls")
        .arg("-l")
        .arg(dir_name)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);

    let lines = result.lines();
    let mut res = vec![];
    for line in lines {
        if line.starts_with("total") {
            continue;
        }

        let parts = line.split_whitespace().collect::<Vec<&str>>();
        let file_name = parts.last().unwrap_or(&"").to_string();

        res.push(file_name.to_string());
    }

    Ok(res.join("\n"))
}
