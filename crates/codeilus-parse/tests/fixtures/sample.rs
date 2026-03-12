use std::fs;

pub struct Config {
    pub path: String,
}

impl Config {
    pub fn load(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.path).unwrap_or_default()
    }
}

pub fn process(config: &Config) -> String {
    config.read().to_uppercase()
}
