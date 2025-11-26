use crate::{CloudEvent, NatsClient};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Publishes events to NATS JetStream
pub struct EventPublisher {
    nats_client: Arc<RwLock<NatsClient>>,
}

impl EventPublisher {
    pub fn new(nats_client: Arc<RwLock<NatsClient>>) -> Self {
        Self { nats_client }
    }

    /// Publish a CloudEvent to NATS
    pub async fn publish(&self, event: &CloudEvent) -> Result<()> {
        let subject = format!("events.{}", event.event_type.replace('.', "_"));
        self.publish_to(&subject, event).await
    }

    /// Publish to a specific subject/stream
    pub async fn publish_to(&self, subject: &str, event: &CloudEvent) -> Result<()> {
        tracing::debug!("Publishing event {} to subject: {}", event.id, subject);
        
        let client = self.nats_client.read().await;
        
        if !client.is_connected() {
            anyhow::bail!("NATS client not connected");
        }

        let payload = event.to_json_bytes()?;
        client.publish(subject, payload).await?;

        tracing::info!("Published event {} to {}", event.id, subject);
        Ok(())
    }
}
