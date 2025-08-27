use serde::Deserialize;

use std::path::Path;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub markdown: MarkdownConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Deserialize)]
pub struct MarkdownConfig {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let config: Config = serde_yaml::from_reader(file)?;
        Ok(config)
    }
}
