use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use regex::Regex;

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
    base_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl RedisConfig {
    pub fn url(&self) -> String {
        match (&self.username, &self.password) {
            (Some(username), Some(password)) => {
                // Build URL with username and password
                let url = self.base_url.clone();

                if let Some(stripped) = url.strip_prefix("redis://") {
                    format!("redis://{}:{}@{}", username, password, stripped)
                } else if let Some(stripped) = url.strip_prefix("rediss://") {
                    format!("rediss://{}:{}@{}", username, password, stripped)
                } else {
                    panic!("Invalid Redis URL scheme: {}", url);
                }
            }
            (None, None) => self.base_url.clone(),
            _ => panic!("Both 'redis.username' and 'redis.password' must be set together"),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut yaml = String::new();
        file.read_to_string(&mut yaml)?;

        // Replace ${VAR} with environment variables
        let re = Regex::new(r"\$\{(\w+)\}")?;
        let processed_yaml = re.replace_all(&yaml, |caps: &regex::Captures| {
            std::env::var(&caps[1]).expect("Failed to load variable from environmen")
        });

        let config: Config = serde_yaml::from_str(&processed_yaml)?;
        Ok(config)
    }
}
