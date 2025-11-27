use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Application error types
#[derive(Debug)]
pub enum NexusError {
    NotFound { resource: String, id: String },
    InvalidInput { field: String, message: String },
    ConfigError { message: String },
    NatsError { message: String },
    WasmError { function: String, message: String },
    InternalError { message: String },
}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::NotFound { resource, id } => {
                write!(f, "{} not found: {}", resource, id)
            }
            NexusError::InvalidInput { field, message } => {
                write!(f, "Invalid input for {}: {}", field, message)
            }
            NexusError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            NexusError::NatsError { message } => {
                write!(f, "NATS error: {}", message)
            }
            NexusError::WasmError { function, message } => {
                write!(f, "WASM execution error in {}: {}", function, message)
            }
            NexusError::InternalError { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for NexusError {}

impl NexusError {
    pub fn to_response(&self, trace_id: Option<String>) -> ErrorResponse {
        let (code, message, details) = match self {
            NexusError::NotFound { resource, id } => (
                "NOT_FOUND".to_string(),
                format!("{} not found: {}", resource, id),
                Some(serde_json::json!({
                    "resource": resource,
                    "id": id
                })),
            ),
            NexusError::InvalidInput { field, message } => (
                "INVALID_INPUT".to_string(),
                message.clone(),
                Some(serde_json::json!({
                    "field": field
                })),
            ),
            NexusError::ConfigError { message } => (
                "CONFIG_ERROR".to_string(),
                message.clone(),
                None,
            ),
            NexusError::NatsError { message } => (
                "NATS_ERROR".to_string(),
                message.clone(),
                None,
            ),
            NexusError::WasmError { function, message } => (
                "WASM_ERROR".to_string(),
                message.clone(),
                Some(serde_json::json!({
                    "function": function
                })),
            ),
            NexusError::InternalError { message } => (
                "INTERNAL_ERROR".to_string(),
                message.clone(),
                None,
            ),
        };

        ErrorResponse {
            error: ErrorDetail {
                code,
                message,
                details,
            },
            trace_id,
        }
    }

    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            NexusError::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            NexusError::InvalidInput { .. } => axum::http::StatusCode::BAD_REQUEST,
            NexusError::ConfigError { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            NexusError::NatsError { .. } => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            NexusError::WasmError { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            NexusError::InternalError { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Helper to create error responses
pub fn error_response(
    error: NexusError,
    trace_id: Option<String>,
) -> (axum::http::StatusCode, axum::Json<ErrorResponse>) {
    let status = error.status_code();
    let response = error.to_response(trace_id);
    (status, axum::Json(response))
}
