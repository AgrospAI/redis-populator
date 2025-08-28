use dotenv::dotenv;
use rustis::{
    client::Client,
    commands::{ConnectionCommands, FlushingMode, HashCommands, ServerCommands},
};
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

mod data;
use data::config::{Config, MarkdownConfig};

use crate::data::config::RedisConfig;



fn setup() -> Result<Config, ()> {
    dotenv().ok();

    let config_path_str: String =
        env::var("CONFIG_PATH").expect("CONFIG_PATH must be set in .env or environment");

    let config_path = Path::new(&config_path_str).join("config.yaml");

    Ok(Config::load(&config_path).expect(&format!(
        "Failed to load configuration from {}",
        config_path.display()
    )))
}

async fn get_markdown(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?.text().await?;

    println!("Fetched {} bytes", response.len());
    Ok(response.lines().map(|s| s.to_string()).collect())
}

async fn parse_table<F, Fut>(lines: &[String], config: &MarkdownConfig, mut callback: F)
where
    F: FnMut(&str, Vec<(&str, &str)>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut table_started = false;
    let mut header_index: usize = 0;
    let mut headers: Vec<&str> = Vec::new();

    for line in lines {
        let line = line.trim();

        if line.starts_with('|') {
            // Split line into columns as &str slices
            let columns: Vec<&str> = line
                .trim_matches('|')
                .split('|')
                .map(|s| s.trim())
                .collect();

            if !table_started {
                // First line is the header
                table_started = true;
                headers = columns.clone(); // store &str slices

                header_index = headers
                    .iter()
                    .position(|c| *c == config.key)
                    .expect("Key column not found!");

                println!("Headers: {:?}", headers);
            } else if !columns
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
            {
                // Skip separator line |---|---|
                let key = columns[header_index];

                // Build Vec<(&str, &str)> pairs
                let row_vec: Vec<(&str, &str)> = headers
                    .iter()
                    .zip(columns.iter())
                    .map(|(&h, &v)| (h, v))
                    .collect();

                callback(key, row_vec).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> rustis::Result<()> {
    let start = Instant::now();
    let config: Config = setup().expect("There was an error in the configuration setup");

    let client = Client::connect(config.redis.url()).await?;
    client.flushdb(FlushingMode::Sync).await?;

    let markdown_lines = get_markdown(&config.markdown.url)
        .await
        .expect("Error fetching markdown file");

    let count = Arc::new(Mutex::new(0usize));
    let error_count = Arc::new(Mutex::new(0usize));

    parse_table(&markdown_lines, &config.markdown, |key, values| {
        let key = key.to_string();
        let values: Vec<(String, String)> = values
            .iter()
            .map(|(f, v)| (f.to_string(), v.to_string()))
            .collect();

        let count = Arc::clone(&count);
        let error_count = Arc::clone(&error_count);

        let client_clone = client.clone();
        async move {
            if let Err(_) = client_clone.hset(&key, values).await {
                let mut e = error_count.lock().await;
                *e += 1;
            } else {
                let mut c = count.lock().await;
                *c += 1;
            }
        }
    })
    .await;

    let count = count.lock().await;
    let error_count = error_count.lock().await;

    println!(
        "Successfully added {ok}/{err} entries",
        ok = *count,
        err = *error_count + *count,
    );

    let elapsed = start.elapsed();
    println!("Exceuted in {:.2?}", elapsed);

    Ok(())
}
