use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use nexus_event_fabric::CloudEvent;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, error};

use crate::state::AppState;

pub struct Server {
    port: u16,
    state: AppState,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    nats_connected: bool,
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
    event_type: String,
}

impl Server {
    pub fn new(port: u16, state: AppState) -> Self {
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

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let nats_connected = state.nats_client.read().await.is_connected();
    
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        nats_connected,
    })
}

async fn event_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(payload): Json<EventPayload>,
) -> Result<Json<EventResponse>, StatusCode> {
    info!("Received event on path: {}", path);

    // Extract event type from path (e.g., /events/user.created -> com.nexus.user.created)
    let event_type = format!("com.nexus.{}", path.replace('/', "."));
    
    // Create CloudEvent
    let cloud_event = CloudEvent::new(&event_type, "/api/webhook")
        .with_data(payload.data);

    let event_id = cloud_event.id.clone();

    // Publish to NATS
    match state.event_publisher.publish(&cloud_event).await {
        Ok(_) => {
            info!("Event {} published successfully", event_id);
            Ok(Json(EventResponse {
                event_id,
                status: "published".to_string(),
                event_type,
            }))
        }
        Err(e) => {
            error!("Failed to publish event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
