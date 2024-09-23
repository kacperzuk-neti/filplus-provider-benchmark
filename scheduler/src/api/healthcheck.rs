use axum::debug_handler;
use serde::Serialize;

use crate::api::api_response::*;

#[derive(Serialize)]
pub struct HealthcheckResponse {
    pub status: String,
}

/// GET /healthcheck
/// Return simple healthcheck response
#[debug_handler]
pub async fn handle() -> Result<ApiResponse<HealthcheckResponse>, ApiResponse<()>> {
    Ok(ok_response(HealthcheckResponse {
        status: "ok".to_string(),
    }))
}
