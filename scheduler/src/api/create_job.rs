use axum::{
    debug_handler,
    extract::{Json, State},
};
use rabbitmq::{JobMessage, Message};
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
        },
    };

    info!("Publishing job message: {:?}", job_message);

    state
        .job_queue
        .publish(&job_message, "all")
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
