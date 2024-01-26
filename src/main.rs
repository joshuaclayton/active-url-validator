use active_url_validator::{UrlOutcome, UrlOutcomes, UrlProgress};
use futures::{stream, StreamExt};
use reqwest;
use reqwest::header;
use serde_json;
use std::io;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use tokio;

fn urls_from_stdin() -> Vec<String> {
    io::stdin().lock().lines().filter_map(|v| v.ok()).collect()
}

fn build_client_with_timeout(timeout: u64) -> Result<reqwest::Client, reqwest::Error> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("URL Verifier"),
    );

    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .default_headers(headers)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let lines = urls_from_stdin();
    let client = build_client_with_timeout(4)?;
    let outcomes = UrlOutcomes::default();
    let progress_bar = UrlProgress::for_urls(&lines);

    stream::iter(&lines)
        .map(|url| {
            let client = &client;
            let bar = &progress_bar;
            async move {
                let start = Instant::now();
                let result = client.head(&*url).send().await;
                let duration = start.elapsed();

                bar.incr();

                UrlOutcome::build(url, result, duration)
            }
        })
        .buffer_unordered(40)
        .for_each(|outcome| {
            let outcomes = &outcomes;
            async move {
                outcomes.push(outcome);
            }
        })
        .await;

    progress_bar.finish();

    println!("{}", serde_json::to_string(&outcomes.values()).unwrap());

    std::thread::spawn(move || drop(outcomes));
    Ok(())
}
