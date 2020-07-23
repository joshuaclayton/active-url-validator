use futures::{stream, StreamExt};
use indicatif::ProgressBar;
use reqwest;
use reqwest::header;
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio;

#[derive(Debug)]
enum Status {
    Timeout,
    Code(String),
    Unknown,
}

#[derive(Debug)]
struct UrlOutcome {
    url: String,
    status: Status,
    duration: Duration,
}

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

impl From<Result<reqwest::StatusCode, reqwest::Error>> for Status {
    fn from(outcome: Result<reqwest::StatusCode, reqwest::Error>) -> Self {
        match outcome {
            Ok(status) => Status::Code(status.as_str().into()),
            Err(e) => {
                if e.is_timeout() {
                    Status::Timeout
                } else {
                    match e.status() {
                        Some(status) => Status::Code(status.as_str().into()),
                        None => Status::Unknown,
                    }
                }
            }
        }
    }
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
                    status: result.map(|v| v.status()).into(),
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
