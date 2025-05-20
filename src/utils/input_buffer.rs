use regex::Regex;

#[derive(Debug, Clone)]
pub struct InputBuffer {
    initializers: Vec<String>,
    pub buffer: String,
    pattern: String,
}

impl InputBuffer {
    pub fn new() -> Self {
        InputBuffer {
            initializers: vec![
                String::from("c"),
                String::from("d"),
                String::from("g"),
                String::from("f"),
            ],
            buffer: String::new(),
            pattern: String::from(
                r#"^(([cdy][ai][wWeEbB\(\)\[\]\{\}"'`])|([cd]f.)|(g[gfd])|([cd][GwWeEbBhjkl\{\}$0])|([cd](gg|ga))|(\d+[hjklwWbBeE])|(\d+[cd][wWeEbB])|yy|cc|dd|([cd][fFtT].))$"#,
            ),
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
        let rx = Regex::new(&self.pattern)?;

        let result = rx.is_match(&self.buffer);

        Ok(result)
    }
}
