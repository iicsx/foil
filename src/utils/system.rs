use std::process;

pub fn whoami() -> String {
    let output = std::process::Command::new("whoami").output();

    return match output {
        Ok(output) => {
            let user = String::from_utf8_lossy(&output.stdout);

            String::from(user)
        }
        Err(_) => {
            process::exit(1);
        }
    };
}

pub fn hostname() -> String {
    let output = std::process::Command::new("hostname").output();

    return match output {
        Ok(output) => String::from(String::from_utf8_lossy(&output.stdout)),
        Err(_) => match try_get_hostnamectl() {
            Some(hostname) => hostname,
            None => process::exit(1),
        },
    };
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

    return match output {
        Ok(output) => {
            let path = String::from_utf8_lossy(&output.stdout);

            String::from(path)
        }
        Err(_) => {
            process::exit(1);
        }
    };
}
