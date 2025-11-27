use crate::config::NexusConfig;
use crate::executor::FunctionExecutor;
use crate::metrics::MetricsCollector;
use nexus_event_fabric::{EventPublisher, EventStore, NatsClient};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<NexusConfig>,
    pub nats_client: Arc<RwLock<NatsClient>>,
    pub event_publisher: Arc<EventPublisher>,
    pub event_store: Arc<EventStore>,
    pub function_executor: Arc<FunctionExecutor>,
    pub metrics: MetricsCollector,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(config: NexusConfig, nats_client: Arc<RwLock<NatsClient>>) -> Result<Self, anyhow::Error> {
        let config = Arc::new(config);
        let event_publisher = Arc::new(EventPublisher::new(nats_client.clone()));
        let event_store = Arc::new(EventStore::new(nats_client.clone(), "events".to_string()));
        let function_executor = Arc::new(FunctionExecutor::new(config.clone())?);
        let metrics = MetricsCollector::new();
        let start_time = Instant::now();
        
        Ok(Self {
            config,
            nats_client,
            event_publisher,
            event_store,
            function_executor,
            metrics,
            start_time,
        })
    }
}
