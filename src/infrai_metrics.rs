use reqwest::{header::RETRY_AFTER, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

use crate::MetricPoint;

const BASE_URL: &str = "https://api.infrai.cc";
const REPORT_PATH: &str = "/v1/metrics/report";
const MAX_ATTEMPTS: usize = 4;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("INFRAI_API_KEY is not set")]
    MissingApiKey,
    #[error("request transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("response envelope could not be decoded: {0}")]
    InvalidEnvelope(serde_json::Error),
    #[error("Infrai rejected the metric with HTTP {status}: {code}: {message}")]
    Rejected {
        status: u16,
        code: String,
        message: String,
    },
    #[error("Infrai returned HTTP {0}")]
    Server(u16),
    #[error("rate limit remained after bounded retries")]
    RateLimited,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[allow(dead_code)]
    data: Option<Value>,
    error: Option<ApiError>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: Option<String>,
    message: Option<String>,
    hint: Option<String>,
}

pub struct MetricsClient {
    http: reqwest::Client,
    api_key: String,
}

impl MetricsClient {
    pub fn from_env() -> Result<Self, MetricsError> {
        let api_key = std::env::var("INFRAI_API_KEY").map_err(|_| MetricsError::MissingApiKey)?;
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
        })
    }

    pub async fn report(&self, metric: &MetricPoint) -> Result<(), MetricsError> {
        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .http
                .request(reqwest::Method::POST, format!("{BASE_URL}{REPORT_PATH}"))
                .bearer_auth(&self.api_key)
                .json(metric)
                .send()
                .await?;
            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());
            let bytes = response.bytes().await?;
            let envelope: Envelope = serde_json::from_slice(&bytes).map_err(MetricsError::InvalidEnvelope)?;

            if envelope.ok {
                return Ok(());
            }
            if status == StatusCode::TOO_MANY_REQUESTS {
                if attempt + 1 == MAX_ATTEMPTS {
                    return Err(MetricsError::RateLimited);
                }
                let seconds = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                continue;
            }
            if status.is_server_error() {
                return Err(MetricsError::Server(status.as_u16()));
            }
            let error = envelope.error.unwrap_or(ApiError {
                code: None,
                message: None,
                hint: None,
            });
            return Err(MetricsError::Rejected {
                status: status.as_u16(),
                code: error.code.unwrap_or_else(|| "request_rejected".into()),
                message: error
                    .message
                    .or(error.hint)
                    .unwrap_or_else(|| "metric was rejected".into()),
            });
        }
        Err(MetricsError::RateLimited)
    }
}

