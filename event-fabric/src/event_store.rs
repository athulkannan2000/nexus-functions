use crate::{CloudEvent, NatsClient};
use anyhow::{Context, Result};
use async_nats::jetstream;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Event store for querying and retrieving events from JetStream
pub struct EventStore {
    nats_client: Arc<RwLock<NatsClient>>,
    stream_name: String,
}

impl EventStore {
    pub fn new(nats_client: Arc<RwLock<NatsClient>>, stream_name: String) -> Self {
        Self {
            nats_client,
            stream_name,
        }
    }

    /// Retrieve a single event by its ID
    pub async fn get_event_by_id(&self, event_id: &str) -> Result<Option<CloudEvent>> {
        debug!("Retrieving event by ID: {}", event_id);

        let client = self.nats_client.read().await;
        if !client.is_connected() {
            anyhow::bail!("NATS client not connected");
        }

        let nats_client = client
            .client()
            .context("NATS client not available")?;
        
        let jetstream = jetstream::new(nats_client.clone());

        // Get the stream
        let stream = jetstream
            .get_stream(&self.stream_name)
            .await
            .context("Failed to get stream")?;

        // Create a temporary consumer to fetch messages
        let consumer = stream
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                durable_name: None,
                filter_subject: format!("{}.*", self.stream_name),
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
                ack_policy: async_nats::jetstream::consumer::AckPolicy::None,
                ..Default::default()
            })
            .await
            .context("Failed to create consumer")?;

        // Fetch messages and search for the matching event ID
        let mut messages = consumer.fetch().max_messages(1000).messages().await?;

        while let Some(Ok(msg)) = messages.next().await {
            if let Ok(event) = serde_json::from_slice::<CloudEvent>(&msg.payload) {
                if event.id == event_id {
                    info!("Found event: {}", event_id);
                    return Ok(Some(event));
                }
            }
        }

        warn!("Event not found: {}", event_id);
        Ok(None)
    }

    /// List events by type with optional limit
    pub async fn list_events(
        &self,
        event_type: Option<String>,
        limit: usize,
    ) -> Result<Vec<CloudEvent>> {
        debug!(
            "Listing events: type={:?}, limit={}",
            event_type, limit
        );

        let client = self.nats_client.read().await;
        if !client.is_connected() {
            anyhow::bail!("NATS client not connected");
        }

        let nats_client = client
            .client()
            .context("NATS client not available")?;
        
        let jetstream = jetstream::new(nats_client.clone());

        let stream = jetstream
            .get_stream(&self.stream_name)
            .await
            .context("Failed to get stream")?;

        // Determine filter subject
        let filter_subject = match event_type {
            Some(ref et) => format!("{}.{}", self.stream_name, et.replace('.', "_")),
            None => format!("{}.*", self.stream_name),
        };

        let consumer = stream
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                durable_name: None,
                filter_subject,
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
                ack_policy: async_nats::jetstream::consumer::AckPolicy::None,
                ..Default::default()
            })
            .await
            .context("Failed to create consumer")?;

        let mut events = Vec::new();
        let mut messages = consumer
            .fetch()
            .max_messages(limit.min(1000))
            .messages()
            .await?;

        while let Some(Ok(msg)) = messages.next().await {
            if let Ok(event) = serde_json::from_slice::<CloudEvent>(&msg.payload) {
                events.push(event);
                if events.len() >= limit {
                    break;
                }
            }
        }

        info!(
            "Retrieved {} events (type={:?})",
            events.len(),
            event_type
        );
        Ok(events)
    }

    /// Get the count of messages in the stream
    pub async fn get_event_count(&self) -> Result<u64> {
        let client = self.nats_client.read().await;
        if !client.is_connected() {
            anyhow::bail!("NATS client not connected");
        }

        let nats_client = client
            .client()
            .context("NATS client not available")?;
        
        let jetstream = jetstream::new(nats_client.clone());

        let mut stream = jetstream
            .get_stream(&self.stream_name)
            .await
            .context("Failed to get stream")?;

        let info = stream.info().await.context("Failed to get stream info")?;
        Ok(info.state.messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_store_creation() {
        let nats_client = Arc::new(RwLock::new(NatsClient::new()));
        let store = EventStore::new(nats_client, "test_events".to_string());
        assert_eq!(store.stream_name, "test_events");
    }
}
