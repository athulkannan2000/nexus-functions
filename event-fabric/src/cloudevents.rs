use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CloudEvents v1.0 specification
/// https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent {
    /// CloudEvents version
    pub specversion: String,
    
    /// Event type identifier
    #[serde(rename = "type")]
    pub event_type: String,
    
    /// Event source (URI)
    pub source: String,
    
    /// Unique event ID
    pub id: String,
    
    /// Event timestamp
    pub time: DateTime<Utc>,
    
    /// Content type of data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datacontenttype: Option<String>,
    
    /// Event payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    
    /// Additional extension attributes
    #[serde(flatten)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
}

impl CloudEvent {
    /// Create a new CloudEvent with defaults
    pub fn new(event_type: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            specversion: "1.0".to_string(),
            event_type: event_type.into(),
            source: source.into(),
            id: Uuid::new_v4().to_string(),
            time: Utc::now(),
            datacontenttype: Some("application/json".to_string()),
            data: None,
            extensions: std::collections::HashMap::new(),
        }
    }

    /// Set the event data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Add an extension attribute
    pub fn with_extension(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extensions.insert(key.into(), value);
        self
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cloud_event() {
        let event = CloudEvent::new("com.example.test", "/api/test")
            .with_data(serde_json::json!({"test": true}));

        assert_eq!(event.specversion, "1.0");
        assert_eq!(event.event_type, "com.example.test");
        assert_eq!(event.source, "/api/test");
        assert!(event.data.is_some());
    }

    #[test]
    fn test_serialize_deserialize() {
        let event = CloudEvent::new("test.event", "/source")
            .with_data(serde_json::json!({"key": "value"}));

        let json = event.to_json().unwrap();
        let deserialized = CloudEvent::from_json(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.event_type, deserialized.event_type);
    }
}
