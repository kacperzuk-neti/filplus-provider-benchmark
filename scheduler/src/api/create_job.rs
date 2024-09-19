use axum::{
    debug_handler,
    extract::{Json, State},
};
use rabbitmq::{JobMessage, Message};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info};
use url::Url;
use uuid::Uuid;

use crate::api::api_response::*;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct JobInput {
    pub url: String,
    pub routing_key: String,
}

#[derive(Serialize)]
pub struct JobResponse {
    pub job_id: Uuid,
}

/// POST /job
/// Create a new job to be processed by the worker
#[debug_handler]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JobInput>,
) -> Result<ApiResponse<JobResponse>, ApiResponse<()>> {
    let url = validate_url(&payload)?;
    validate_routing_key(&payload)?;

    let (start_range, end_range) = get_file_range_for_file(&url.as_ref()).await?;

    let job_id = Uuid::new_v4();
    let now_plus = SystemTime::now() + Duration::new(5, 0);
    let start_timestamp = now_plus
        .duration_since(UNIX_EPOCH)
        .map_err(|_| internal_server_error("Failed to calculate start time"))?;

    debug!(
        "Job ID: {}, URL: {}, Start Timestamp: {:?}",
        job_id, url, start_timestamp
    );

    let job_message = Message::WorkerJob {
        job_id,
        payload: JobMessage {
            url: url.to_string(),
            start_timestamp,
            start_range,
            end_range,
        },
    };

    info!("Publishing job message: {:?}", job_message);

    state
        .job_queue
        .lock()
        .await
        .publish(&job_message, &payload.routing_key)
        .await
        .map_err(|_| internal_server_error("Failed to publish job message"))?;

    info!("Job message published successfully");

    Ok(ok_response(JobResponse { job_id }))
}

/// Validate url and its scheme
fn validate_url(payload: &JobInput) -> Result<Url, ApiResponse<()>> {
    let url = Url::parse(&payload.url).map_err(|_| bad_request("Invalid URL provided"))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        _ => Err(bad_request("URL scheme must be http or https")),
    }
}

/// Validate routing key
/// In future we want to validate if the routing key is valid, maybe by checking a set of allowed keys
fn validate_routing_key(payload: &JobInput) -> Result<(), ApiResponse<()>> {
    if payload.routing_key.is_empty() {
        return Err(bad_request("Routing key cannot be empty"));
    }

    Ok(())
}

/// Get a random range of 100MB from the file using HEAD request
async fn get_file_range_for_file(url: &str) -> Result<(u64, u64), ApiResponse<()>> {
    let response = Client::new()
        .head(url)
        .send()
        .await
        .map_err(|e| bad_request(format!("Failed to publish job message {}", e)))?;

    debug!("Response: {:?}", response);

    // For some freak reason response.content_length() is returning 0
    let content_length = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .ok_or_else(|| bad_request("Content-Length header is missing in the response"))?
        .to_str()
        .map_err(|e| bad_request(format!("Failed to parse Content-Length header: {}", e)))?
        .parse::<u64>()
        .map_err(|e| bad_request(format!("Failed to parse Content-Length header: {}", e)))?;

    debug!("Content-Length: {:?}", content_length);

    let size = 100 * 1024 * 1024; // 100 MB

    if content_length < size {
        return Err(bad_request("File size is less than 100MB"));
    }

    let mut rng = rand::thread_rng();
    let start_range = rng.gen_range(0..content_length - size);
    let end_range = start_range + size;

    Ok((start_range, end_range))
}
