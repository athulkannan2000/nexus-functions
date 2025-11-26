use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::NexusConfig;

#[derive(Clone)]
pub struct ServerState {
    pub config: Arc<NexusConfig>,
}

pub struct Server {
    port: u16,
    state: ServerState,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Deserialize)]
struct EventPayload {
    #[serde(flatten)]
    data: serde_json::Value,
}

#[derive(Serialize)]
struct EventResponse {
    event_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<String>,
}

impl Server {
    pub fn new(port: u16, config: NexusConfig) -> Self {
        let state = ServerState {
            config: Arc::new(config),
        };

        Self { port, state }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/events/*path", post(event_handler))
            .layer(TraceLayer::new_for_http())
            .with_state(self.state);

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Starting server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn event_handler(
    State(_state): State<ServerState>,
    Json(payload): Json<EventPayload>,
) -> Result<Json<EventResponse>, StatusCode> {
    info!("Received event: {:?}", payload.data);

    // Generate event ID
    let event_id = uuid::Uuid::new_v4().to_string();

    // TODO: Publish to NATS
    // TODO: Trigger function execution

    Ok(Json(EventResponse {
        event_id,
        status: "published".to_string(),
        function: None,
    }))
}
