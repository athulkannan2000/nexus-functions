use axum::{
    extract::{Path, Query, State},
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

#[derive(Deserialize)]
struct ListEventsQuery {
    #[serde(rename = "type")]
    event_type: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Serialize)]
struct EventListResponse {
    events: Vec<CloudEvent>,
    count: usize,
    total: u64,
}

#[derive(Serialize)]
struct ReplayResponse {
    event_id: String,
    status: String,
    message: String,
}

#[derive(Serialize)]
struct FunctionExecutionResponse {
    event_id: String,
    status: String,
    functions_executed: Vec<FunctionResult>,
}

#[derive(Serialize)]
struct FunctionResult {
    function_name: String,
    status: String,
    output_size: usize,
    output: Option<String>,
}

impl Server {
    pub fn new(port: u16, state: AppState) -> Self {
        Self { port, state }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/events", get(list_events_handler).post(event_handler_root))
            .route("/events/:event_id", get(get_event_handler))
            .route("/replay/:event_id", post(replay_handler))
            .route("/execute/:event_id", post(execute_handler))
            .route("/webhook/*path", post(event_handler))
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

    // Extract event type from path (e.g., /webhook/user.created -> com.nexus.user.created)
    let event_type = format!("com.nexus.{}", path.replace('/', "."));
    
    // Create CloudEvent
    let cloud_event = CloudEvent::new(&event_type, "/api/webhook")
        .with_data(payload.data);

    let event_id = cloud_event.id.clone();

    // Publish to NATS
    match state.event_publisher.publish(&cloud_event).await {
        Ok(_) => {
            info!("Event {} published successfully", event_id);
            
            // Execute matching functions asynchronously (fire and forget)
            let executor = state.function_executor.clone();
            let event_clone = cloud_event.clone();
            tokio::spawn(async move {
                match executor.execute_matching_functions(&event_clone).await {
                    Ok(results) => {
                        info!("Executed {} function(s) for event {}", results.len(), event_clone.id);
                    }
                    Err(e) => {
                        error!("Function execution failed for event {}: {}", event_clone.id, e);
                    }
                }
            });
            
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

async fn event_handler_root(
    State(state): State<AppState>,
    Json(mut payload): Json<EventPayload>,
) -> Result<Json<EventResponse>, StatusCode> {
    info!("Received event on root /events endpoint");

    // Extract event type from payload if provided, otherwise use generic
    let event_type = payload
        .data
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("generic.event")
        .to_string();
    
    // Remove event_type from data if it exists
    payload.data.as_object_mut().map(|obj| obj.remove("event_type"));
    
    let full_event_type = format!("com.nexus.{}", event_type);
    
    // Create CloudEvent
    let cloud_event = CloudEvent::new(&full_event_type, "/api/events")
        .with_data(payload.data);

    let event_id = cloud_event.id.clone();

    // Publish to NATS
    match state.event_publisher.publish(&cloud_event).await {
        Ok(_) => {
            info!("Event {} published successfully", event_id);
            Ok(Json(EventResponse {
                event_id,
                status: "published".to_string(),
                event_type: full_event_type,
            }))
        }
        Err(e) => {
            error!("Failed to publish event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_event_handler(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<CloudEvent>, StatusCode> {
    info!("Retrieving event: {}", event_id);

    match state.event_store.get_event_by_id(&event_id).await {
        Ok(Some(event)) => {
            info!("Event {} retrieved", event_id);
            Ok(Json(event))
        }
        Ok(None) => {
            info!("Event {} not found", event_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to retrieve event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_events_handler(
    State(state): State<AppState>,
    Query(params): Query<ListEventsQuery>,
) -> Result<Json<EventListResponse>, StatusCode> {
    info!(
        "Listing events: type={:?}, limit={}",
        params.event_type, params.limit
    );

    let total = match state.event_store.get_event_count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get event count: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match state
        .event_store
        .list_events(params.event_type, params.limit)
        .await
    {
        Ok(events) => {
            let count = events.len();
            info!("Retrieved {} events", count);
            Ok(Json(EventListResponse {
                events,
                count,
                total,
            }))
        }
        Err(e) => {
            error!("Failed to list events: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn replay_handler(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<ReplayResponse>, StatusCode> {
    info!("Replaying event: {}", event_id);

    // First, retrieve the event
    let event = match state.event_store.get_event_by_id(&event_id).await {
        Ok(Some(event)) => event,
        Ok(None) => {
            return Ok(Json(ReplayResponse {
                event_id,
                status: "not_found".to_string(),
                message: "Event not found".to_string(),
            }));
        }
        Err(e) => {
            error!("Failed to retrieve event for replay: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Re-publish the event to NATS
    match state.event_publisher.publish(&event).await {
        Ok(_) => {
            info!("Event {} replayed successfully", event_id);
            
            // Execute functions asynchronously
            let executor = state.function_executor.clone();
            let event_clone = event.clone();
            tokio::spawn(async move {
                match executor.execute_matching_functions(&event_clone).await {
                    Ok(results) => {
                        info!("Replayed event {} triggered {} function(s)", event_clone.id, results.len());
                    }
                    Err(e) => {
                        error!("Function execution failed for replayed event {}: {}", event_clone.id, e);
                    }
                }
            });
            
            Ok(Json(ReplayResponse {
                event_id,
                status: "replayed".to_string(),
                message: format!("Event type: {}", event.event_type),
            }))
        }
        Err(e) => {
            error!("Failed to replay event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn execute_handler(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<FunctionExecutionResponse>, StatusCode> {
    info!("Executing functions for event: {}", event_id);

    // Retrieve the event
    let event = match state.event_store.get_event_by_id(&event_id).await {
        Ok(Some(event)) => event,
        Ok(None) => {
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Failed to retrieve event: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Execute matching functions
    match state.function_executor.execute_matching_functions(&event).await {
        Ok(results) => {
            let function_results: Vec<FunctionResult> = results
                .into_iter()
                .map(|(name, output)| {
                    let output_str = String::from_utf8(output.clone()).ok();
                    FunctionResult {
                        function_name: name,
                        status: "success".to_string(),
                        output_size: output.len(),
                        output: output_str,
                    }
                })
                .collect();

            info!("Executed {} function(s) for event {}", function_results.len(), event_id);

            Ok(Json(FunctionExecutionResponse {
                event_id,
                status: "executed".to_string(),
                functions_executed: function_results,
            }))
        }
        Err(e) => {
            error!("Function execution failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
