use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub enum ApiResponse<T> {
    BadRequest(Json<ErrorResponse>),
    InternalServerError(Json<ErrorResponse>),
    NotFound(Json<ErrorResponse>),
    OkResponse(Json<T>),
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            ApiResponse::BadRequest(json) => (StatusCode::BAD_REQUEST, json).into_response(),
            ApiResponse::InternalServerError(json) => {
                (StatusCode::INTERNAL_SERVER_ERROR, json).into_response()
            }
            ApiResponse::NotFound(json) => (StatusCode::NOT_FOUND, json).into_response(),
            ApiResponse::OkResponse(json) => (StatusCode::OK, json).into_response(),
        }
    }
}

pub fn bad_request<T: Into<String>>(msg: T) -> ApiResponse<()> {
    ApiResponse::BadRequest(Json(ErrorResponse { error: msg.into() }))
}

pub fn internal_server_error<T: Into<String>>(msg: T) -> ApiResponse<()> {
    ApiResponse::InternalServerError(Json(ErrorResponse { error: msg.into() }))
}

pub fn not_found<T: Into<String>>(msg: T) -> ApiResponse<()> {
    ApiResponse::NotFound(Json(ErrorResponse { error: msg.into() }))
}

pub fn ok_response<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse::OkResponse(Json(data))
}
