use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct InputBuffer {
    initializers: Vec<String>,
    pub buffer: String,
}

impl InputBuffer {
    pub fn new() -> Self {
        InputBuffer {
            initializers: vec![
                String::from("c"),
                String::from("d"),
                String::from("g"),
                String::from("f"),
                String::from("F"),
                String::from("y"),
            ],
            buffer: String::new(),
        }
    }

    pub fn is_initializer(&self, input: &str) -> bool {
        self.initializers.contains(&input.to_string())
    }

    pub fn add(&mut self, input: &str) {
        self.buffer.push_str(input);

        if self.buffer.len() == 3 && !self.valid().unwrap_or(false) {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn valid(&self) -> Result<bool, regex::Error> {
        static COMMAND_PATTERN: OnceLock<Regex> = OnceLock::new();
        let rx = COMMAND_PATTERN.get_or_init(|| {
            Regex::new(
                r#"^(([cdy][ai][wWeEbB\(\)\[\]\{\}"'`])|([cd]f.)|(g[gfd])|([cdy][GwWeEbBhjkl\{\}$0])|([cd](gg|ga))|(\d+[hjklwWbBeE])|(\d+[cd][wWeEbB])|yy|cc|dd|([cd][fFtT].))$"#,
            )
            .expect("the built-in command pattern is valid")
        });

        Ok(rx.is_match(&self.buffer))
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}
