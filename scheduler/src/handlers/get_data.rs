use crate::state::AppState;
use axum::{
    debug_handler, extract::{Query, State}, http::StatusCode, response::{IntoResponse, Json as ResponseJson}
};
use rabbitmq::ResultMessage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GetDataQuery {
    job_id: String,
}

#[derive(Serialize)]
pub struct GetDataResponse {
    result: Option<ResultMessage>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[debug_handler]
pub async fn handle(
    Query(params): Query<GetDataQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Validate the job_id
    let job_id = match Uuid::parse_str(&params.job_id) {
        Ok(job_id) => job_id,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "Invalid job_id; must be a valid UUID".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error_response)).into_response();
        }
    };

    info!("Getting data for job_id: {}", job_id);

    let results = state.results.lock().unwrap();
    let result = results.get(&job_id).cloned();

    if result.is_none() {
        info!("Job data not found for job_id: {}", job_id);

        let error_response = ErrorResponse {
            error: "Job data not found".to_string(),
        };
        return (StatusCode::NOT_FOUND, ResponseJson(error_response)).into_response();
    }

    info!("Job data found for job_id: {} {:?}", job_id, result);

    (StatusCode::OK, ResponseJson(GetDataResponse { result })).into_response()
}
