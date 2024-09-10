use axum::{
    debug_handler,
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use chrono::{Duration, Utc};
use rabbitmq::{JobMessage, Message, QueueHandler};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use url::Url;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct JobInput {
    pub url: String,
}

#[derive(Serialize)]
pub struct JobResponse {
    pub job_id: Uuid,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[debug_handler]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JobInput>,
) -> impl IntoResponse {
    let url = Url::parse(&payload.url);

    // URL Validation
    if url.is_err() {
        let error_response = ErrorResponse {
            error: "Invalid URL provided".to_string(),
        };
        return (StatusCode::BAD_REQUEST, ResponseJson(error_response)).into_response();
    }

    let now = Utc::now();
    debug!("Current timestamp: {}", now);
    let now_plus = now + Duration::seconds(5); // TODO: make it a config ??
    debug!("New timestamp (after 5 seconds): {}", now_plus);

    let job_id = Uuid::new_v4();

    let start_time: i64 = now_plus.timestamp();
    let job_message = Message::WorkerJob {
        job_id,
        payload: JobMessage {
            url: payload.url,
            start_time,
        },
    };

    info!("Publishing job message: {:?}", job_message);

    match state.job_queue.publish(&job_message).await {
        Ok(_) => info!("Job message published successfully"),
        Err(e) => {
            info!("Failed to publish job message: {:?}", e);
            let error_response = ErrorResponse {
                error: "Failed to publish job message".to_string(),
            };
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(error_response),
            )
                .into_response();
        }
    }

    (StatusCode::OK, ResponseJson(JobResponse { job_id })).into_response()
}
