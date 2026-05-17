use anyhow::{Context, Result};

const HOME_PLACEHOLDER: &str = "{{HOME}}";

pub struct Sanitizer {
    home: String,
}

impl Sanitizer {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").context("HOME environment variable is not set")?;
        Ok(Self { home })
    }

    pub fn sanitize(&self, content: &str) -> String {
        content.replace(&self.home, HOME_PLACEHOLDER)
    }

    pub fn desanitize(&self, content: &str) -> String {
        content.replace(HOME_PLACEHOLDER, &self.home)
    }
}
