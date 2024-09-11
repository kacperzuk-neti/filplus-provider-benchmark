use crate::handlers::{create_job, get_data};
use crate::state::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/data", get(get_data::handle))
        .route("/job", post(create_job::handle))
}
