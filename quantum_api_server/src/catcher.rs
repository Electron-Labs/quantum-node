use rocket::{serde::json::Json, catch};

use crate::error::error::ErrorResponse;

#[catch(415)]
pub fn unsupported_media_type() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error_type: "Unsupported Media Type".to_string(),
        message: "May be missing payload".to_string(),
    })
}

#[catch(500)]
pub fn internal_server_error() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error_type: "Internal Server Error".to_string(),
        message: "May be invalid payload".to_string(),
    })
}