use axum::http::HeaderMap;
use serde_json::Value;

/// Handles HTTP webhook ingestion and converts to CloudEvents
pub struct WebhookHandler;

impl WebhookHandler {
    pub fn new() -> Self {
        Self
    }

    /// Convert HTTP request to CloudEvent format
    pub fn to_cloud_event(
        &self,
        path: &str,
        _headers: &HeaderMap,
        body: Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        // Extract event type from path
        let event_type = path
            .strip_prefix("/events/")
            .unwrap_or("unknown")
            .replace('/', ".");

        // Build CloudEvent
        let event = serde_json::json!({
            "specversion": "1.0",
            "type": format!("com.nexus.{}", event_type),
            "source": "/api/webhook",
            "id": uuid::Uuid::new_v4().to_string(),
            "time": chrono::Utc::now().to_rfc3339(),
            "datacontenttype": "application/json",
            "data": body
        });

        Ok(event)
    }
}

impl Default for WebhookHandler {
    fn default() -> Self {
        Self::new()
    }
}
