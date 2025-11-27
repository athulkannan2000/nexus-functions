use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Metrics collector for monitoring system health
#[derive(Clone)]
pub struct MetricsCollector {
    data: Arc<RwLock<MetricsData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    // Event metrics
    pub events_published: u64,
    pub events_replayed: u64,
    pub events_failed: u64,
    
    // Function metrics
    pub functions_executed: u64,
    pub functions_succeeded: u64,
    pub functions_failed: u64,
    pub total_execution_time_ms: u64,
    
    // System metrics
    pub uptime_seconds: u64,
    pub nats_connected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub events: EventMetrics,
    pub functions: FunctionMetrics,
    pub system: SystemMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventMetrics {
    pub published: u64,
    pub replayed: u64,
    pub failed: u64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionMetrics {
    pub executed: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub avg_execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub nats_connected: bool,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(MetricsData {
                events_published: 0,
                events_replayed: 0,
                events_failed: 0,
                functions_executed: 0,
                functions_succeeded: 0,
                functions_failed: 0,
                total_execution_time_ms: 0,
                uptime_seconds: 0,
                nats_connected: false,
            })),
        }
    }

    pub async fn increment_events_published(&self) {
        let mut data = self.data.write().await;
        data.events_published += 1;
    }

    pub async fn increment_events_replayed(&self) {
        let mut data = self.data.write().await;
        data.events_replayed += 1;
    }

    pub async fn increment_events_failed(&self) {
        let mut data = self.data.write().await;
        data.events_failed += 1;
    }

    pub async fn record_function_execution(&self, duration_ms: u64, success: bool) {
        let mut data = self.data.write().await;
        data.functions_executed += 1;
        data.total_execution_time_ms += duration_ms;
        
        if success {
            data.functions_succeeded += 1;
        } else {
            data.functions_failed += 1;
        }
    }

    pub async fn set_nats_connected(&self, connected: bool) {
        let mut data = self.data.write().await;
        data.nats_connected = connected;
    }

    pub async fn update_uptime(&self, seconds: u64) {
        let mut data = self.data.write().await;
        data.uptime_seconds = seconds;
    }

    pub async fn get_metrics(&self) -> Metrics {
        let data = self.data.read().await;
        
        let event_total = data.events_published + data.events_replayed;
        let event_success_rate = if event_total > 0 {
            ((event_total - data.events_failed) as f64 / event_total as f64) * 100.0
        } else {
            100.0
        };

        let function_success_rate = if data.functions_executed > 0 {
            (data.functions_succeeded as f64 / data.functions_executed as f64) * 100.0
        } else {
            100.0
        };

        let avg_execution_time = if data.functions_executed > 0 {
            data.total_execution_time_ms as f64 / data.functions_executed as f64
        } else {
            0.0
        };

        Metrics {
            events: EventMetrics {
                published: data.events_published,
                replayed: data.events_replayed,
                failed: data.events_failed,
                success_rate: event_success_rate,
            },
            functions: FunctionMetrics {
                executed: data.functions_executed,
                succeeded: data.functions_succeeded,
                failed: data.functions_failed,
                success_rate: function_success_rate,
                avg_execution_time_ms: avg_execution_time,
            },
            system: SystemMetrics {
                uptime_seconds: data.uptime_seconds,
                nats_connected: data.nats_connected,
            },
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Function execution timing helper
pub struct ExecutionTimer {
    start: Instant,
}

impl ExecutionTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}
