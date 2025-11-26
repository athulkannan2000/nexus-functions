use crate::config::NexusConfig;
use nexus_event_fabric::{EventPublisher, EventStore, NatsClient};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<NexusConfig>,
    pub nats_client: Arc<RwLock<NatsClient>>,
    pub event_publisher: Arc<EventPublisher>,
    pub event_store: Arc<EventStore>,
}

impl AppState {
    pub fn new(config: NexusConfig, nats_client: Arc<RwLock<NatsClient>>) -> Self {
        let config = Arc::new(config);
        let event_publisher = Arc::new(EventPublisher::new(nats_client.clone()));
        let event_store = Arc::new(EventStore::new(nats_client.clone(), "events".to_string()));
        
        Self {
            config,
            nats_client,
            event_publisher,
            event_store,
        }
    }
}
