use axum::{http::StatusCode, response::{IntoResponse, Response, Json}};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CloudEventError {
    #[error("Invalid CloudEvents JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing required CloudEvent field: {0}")]
    MissingField(&'static str),

    #[error("Invalid CloudEvents specversion: {0} (expected 1.0)")]
    InvalidSpecVersion(String),

    #[error("Failed to read request body: {0}")]
    BodyError(#[from] axum::BoxError),

    #[error("CloudEvent data deserialization failed: {0}")]
    DataDeserialization(serde_json::Error),

    #[error("Invalid UTF-8 in request body: {0}")]
    InvalidUtf8(String),

    #[error("Invalid HTTP header value")]
    InvalidHeaderValue,

    #[error("CloudEvents HTTP header '{0}' is missing")]
    MissingHeader(&'static str),

    #[error("Invalid HTTP Content-Type: {0}")]
    InvalidContentType(String),

    #[error("Failed to decode percent-encoded header value: {0}")]
    PercentDecodeError(String),

    #[error("Event does not match any variant: {0}")]
    NoMatchingVariant(String),
}

pub type Result<T> = std::result::Result<T, CloudEventError>;

impl IntoResponse for CloudEventError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            CloudEventError::Json(e) => {
                (StatusCode::BAD_REQUEST, "invalid_json", e.to_string())
            }
            CloudEventError::MissingField(field) => {
                (StatusCode::BAD_REQUEST, "missing_field", format!("Missing required field: {}", field))
            }
            CloudEventError::InvalidSpecVersion(v) => {
                (StatusCode::BAD_REQUEST, "invalid_specversion", format!("Unsupported specversion: {} (expected 1.0)", v))
            }
            CloudEventError::BodyError(e) => {
                (StatusCode::BAD_REQUEST, "body_error", e.to_string())
            }
            CloudEventError::DataDeserialization(e) => {
                (StatusCode::BAD_REQUEST, "data_deserialization", format!("Data deserialization failed: {}", e))
            }
            CloudEventError::InvalidUtf8(e) => {
                (StatusCode::BAD_REQUEST, "invalid_utf8", e.clone())
            }
            CloudEventError::InvalidHeaderValue => {
                (StatusCode::BAD_REQUEST, "invalid_header", "Invalid HTTP header value".to_string())
            }
            CloudEventError::MissingHeader(header) => {
                (StatusCode::BAD_REQUEST, "missing_header", format!("Missing required header: {}", header))
            }
            CloudEventError::InvalidContentType(ct) => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, "invalid_content_type", format!("Invalid Content-Type: {}", ct))
            }
            CloudEventError::PercentDecodeError(e) => {
                (StatusCode::BAD_REQUEST, "percent_decode_error", e.clone())
            }
            CloudEventError::NoMatchingVariant(t) => {
                (StatusCode::BAD_REQUEST, "no_matching_variant", format!("No matching variant for event type: {}", t))
            }
        };

        #[cfg(feature = "tracing")]
        tracing::error!("CloudEvent error: {} - {}", error_type, message);

        (
            status,
            Json(json!({
                "error": error_type,
                "message": message
            }))
        ).into_response()
    }
}
