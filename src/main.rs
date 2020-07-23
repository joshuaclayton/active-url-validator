use futures::{stream, StreamExt};
use indicatif::ProgressBar;
use reqwest;
use reqwest::header;
use serde::Serialize;
use serde_json;
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio;

#[derive(Clone, Debug, Serialize)]
enum Status {
    Timeout,
    Code(String),
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
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

struct UrlProgress(Mutex<ProgressBar>);

impl UrlProgress {
    pub fn for_urls<T>(urls: &Vec<T>) -> Self {
        UrlProgress(Mutex::new(ProgressBar::new(urls.len().try_into().unwrap())))
    }

    pub fn incr(&self) {
        self.0.lock().unwrap().inc(1)
    }

    pub fn finish(&self) {
        self.0.lock().unwrap().finish()
    }
}

struct UrlOutcomes(Mutex<Vec<UrlOutcome>>);

impl Default for UrlOutcomes {
    fn default() -> Self {
        UrlOutcomes(Mutex::new(vec![]))
    }
}

impl UrlOutcomes {
    fn push(&self, outcome: UrlOutcome) {
        let mut inner = self.0.lock().unwrap();
        inner.push(outcome)
    }

    fn values(&self) -> Vec<UrlOutcome> {
        self.0.lock().unwrap().to_vec()
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let lines = urls_from_stdin();
    let client = build_client_with_timeout(8)?;
    let outcomes = UrlOutcomes::default();
    let progress_bar = UrlProgress::for_urls(&lines);

    let statuses = stream::iter(&lines)
        .map(|url| {
            let client = &client;
            let bar = &progress_bar;
            async move {
                let start = Instant::now();
                let result = client.get(&*url).send().await;
                let duration = start.elapsed();

                bar.incr();

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
            let outcomes = &outcomes;
            async move {
                outcomes.push(outcome);
            }
        })
        .await;

    progress_bar.finish();

    println!("{}", serde_json::to_string(&outcomes.values()).unwrap());

    Ok(())
}
