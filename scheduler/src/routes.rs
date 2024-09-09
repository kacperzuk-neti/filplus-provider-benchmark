use axum::Router;
use axum::routing::{get, post};
use crate::handlers::{get_data, create_job};

pub fn create_routes() -> Router {
    Router::new()
        .route("/data", get(get_data::handle))
        .route("/job", post(create_job::handle))
}