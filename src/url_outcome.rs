use reqwest;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone, Debug, Serialize)]
enum Status {
    Timeout,
    Code(String),
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct UrlOutcome {
    url: String,
    status: Status,
    duration: Duration,
}

impl UrlOutcome {
    pub fn build(
        url: &str,
        result: Result<reqwest::Response, reqwest::Error>,
        duration: Duration,
    ) -> Self {
        UrlOutcome {
            url: url.to_string(),
            status: result.map(|v| v.status()).into(),
            duration,
        }
    }
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
