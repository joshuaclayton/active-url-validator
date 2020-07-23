use futures::{stream, StreamExt};
use indicatif::ProgressBar;
use reqwest;
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio;

#[derive(Debug)]
struct UrlOutcome {
    url: String,
    status: Result<reqwest::StatusCode, reqwest::Error>,
    duration: Duration,
}

fn urls_from_stdin() -> Vec<String> {
    io::stdin().lock().lines().filter_map(|v| v.ok()).collect()
}

fn build_client_with_timeout(timeout: u64) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let lines = urls_from_stdin();
    let client = build_client_with_timeout(8)?;
    let inner2: Vec<UrlOutcome> = vec![];
    let results = Arc::new(Mutex::new(inner2));
    let bar = Arc::new(Mutex::new(ProgressBar::new(
        lines.len().try_into().unwrap(),
    )));

    let statuses = stream::iter(&lines)
        .map(|url| {
            let client = &client;
            let bar = Arc::clone(&bar);
            async move {
                let start = Instant::now();
                let result = client.get(&*url).send().await;
                let duration = start.elapsed();

                bar.lock().unwrap().inc(1);

                UrlOutcome {
                    url: url.to_string(),
                    status: result.map(|v| v.status()),
                    duration,
                }
            }
        })
        .buffer_unordered(20);

    statuses
        .for_each(|outcome| {
            let results = Arc::clone(&results);
            async move {
                let mut inner = results.lock().unwrap();
                inner.push(outcome);
            }
        })
        .await;

    bar.lock().unwrap().finish();
    println!("final: {:?}", results.lock().unwrap());
    Ok(())
}
