use anyhow::{Context, Result};
use async_nats::jetstream;
use std::time::Duration;
use tracing::{info, warn};

/// NATS JetStream client for event streaming
pub struct NatsClient {
    client: Option<async_nats::Client>,
    jetstream: Option<jetstream::Context>,
}

impl NatsClient {
    /// Create a new NATS client (not connected)
    pub fn new() -> Self {
        Self {
            client: None,
            jetstream: None,
        }
    }

    /// Connect to NATS server
    pub async fn connect(&mut self, url: &str) -> Result<()> {
        info!("Connecting to NATS at {}...", url);
        
        let client = async_nats::connect(url)
            .await
            .context("Failed to connect to NATS server")?;
        
        let jetstream = jetstream::new(client.clone());
        
        self.client = Some(client);
        self.jetstream = Some(jetstream);
        
        info!("Connected to NATS successfully");
        Ok(())
    }

    /// Connect with retry logic (for embedded NATS that may take time to start)
    pub async fn connect_with_retry(&mut self, url: &str, max_retries: u32) -> Result<()> {
        for attempt in 1..=max_retries {
            match self.connect(url).await {
                Ok(_) => return Ok(()),
                Err(e) if attempt < max_retries => {
                    warn!("NATS connection attempt {}/{} failed: {}", attempt, max_retries, e);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => return Err(e),
            }
        }
        anyhow::bail!("Failed to connect to NATS after {} retries", max_retries)
    }

    /// Create or get a JetStream stream
    pub async fn create_stream(&self, stream_name: &str) -> Result<()> {
        let jetstream = self.jetstream.as_ref()
            .context("Not connected to NATS")?;

        // Check if stream already exists
        match jetstream.get_stream(stream_name).await {
            Ok(_) => {
                info!("Stream '{}' already exists", stream_name);
                return Ok(());
            }
            Err(_) => {
                info!("Creating new stream '{}'", stream_name);
            }
        }

        // Create the stream
        jetstream
            .create_stream(jetstream::stream::Config {
                name: stream_name.to_string(),
                subjects: vec![format!("{}.*", stream_name)],
                retention: jetstream::stream::RetentionPolicy::Limits,
                max_messages: 100_000,
                max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
                storage: jetstream::stream::StorageType::File,
                ..Default::default()
            })
            .await
            .context("Failed to create stream")?;

        info!("Stream '{}' created successfully", stream_name);
        Ok(())
    }

    /// Publish a message to a subject
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<()> {
        let jetstream = self.jetstream.as_ref()
            .context("Not connected to NATS")?;

        jetstream
            .publish(subject.to_string(), payload.into())
            .await
            .context("Failed to publish message")?
            .await
            .context("Failed to get publish acknowledgment")?;

        Ok(())
    }

    /// Get the underlying NATS client
    pub fn client(&self) -> Option<&async_nats::Client> {
        self.client.as_ref()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

impl Default for NatsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nats_client_creation() {
        let client = NatsClient::new();
        assert!(!client.is_connected());
    }
}
