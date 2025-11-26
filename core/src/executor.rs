use crate::config::NexusConfig;
use anyhow::{Context, Result};
use nexus_event_fabric::CloudEvent;
use nexus_runtime::WasmExecutor;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info, warn};

/// Manages function execution based on configuration
pub struct FunctionExecutor {
    config: Arc<NexusConfig>,
    wasm_executor: Arc<WasmExecutor>,
}

impl FunctionExecutor {
    pub fn new(config: Arc<NexusConfig>) -> Result<Self> {
        let wasm_executor = Arc::new(WasmExecutor::new()?);
        
        Ok(Self {
            config,
            wasm_executor,
        })
    }

    /// Execute a function by name with event data
    pub async fn execute_function(
        &self,
        function_name: &str,
        event: &CloudEvent,
    ) -> Result<Vec<u8>> {
        info!("Executing function: {}", function_name);

        // Find function in config
        let function = self
            .config
            .functions
            .iter()
            .find(|f| f.name == function_name)
            .with_context(|| format!("Function '{}' not found in configuration", function_name))?;

        // Load WASM module
        let module_path = PathBuf::from(&function.code);
        let module_bytes = fs::read(&module_path)
            .await
            .with_context(|| format!("Failed to read WASM module at {:?}", module_path))?;

        info!(
            "Loaded WASM module for '{}' ({} bytes)",
            function_name,
            module_bytes.len()
        );

        // Prepare input (serialize CloudEvent to JSON)
        let input = event
            .to_json_bytes()
            .context("Failed to serialize CloudEvent")?;

        // Execute WASM module
        let output = self
            .wasm_executor
            .execute(&module_bytes, &input)
            .await
            .with_context(|| format!("Failed to execute function '{}'", function_name))?;

        info!(
            "Function '{}' executed successfully, output size: {} bytes",
            function_name,
            output.len()
        );

        Ok(output)
    }

    /// Find functions that should be triggered by an event
    pub fn find_matching_functions(&self, event_type: &str) -> Vec<String> {
        self.config
            .functions
            .iter()
            .filter_map(|func| {
                // Check if trigger matches this event type
                let matches = if func.on.http.is_some() {
                    // HTTP triggers - match all for MVP (can be refined later)
                    true
                } else if let Some(nats) = &func.on.nats {
                    // NATS triggers match based on subject pattern
                    event_type.contains(&nats.subject) || nats.subject.contains(event_type)
                } else {
                    false
                };

                if matches {
                    Some(func.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Execute all functions that match an event
    pub async fn execute_matching_functions(&self, event: &CloudEvent) -> Result<Vec<(String, Vec<u8>)>> {
        let matching_functions = self.find_matching_functions(&event.event_type);

        if matching_functions.is_empty() {
            warn!("No functions matched event type: {}", event.event_type);
            return Ok(vec![]);
        }

        info!(
            "Found {} matching function(s) for event type: {}",
            matching_functions.len(),
            event.event_type
        );

        let mut results = Vec::new();

        for func_name in matching_functions {
            match self.execute_function(&func_name, event).await {
                Ok(output) => {
                    info!("Function '{}' executed successfully", func_name);
                    results.push((func_name, output));
                }
                Err(e) => {
                    error!("Function '{}' execution failed: {}", func_name, e);
                    // Continue with other functions
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FunctionConfig, TriggerConfig};

    fn create_test_config() -> NexusConfig {
        use crate::config::{HttpTrigger, TriggerConfig};
        
        NexusConfig {
            version: "v1".to_string(),
            functions: vec![FunctionConfig {
                name: "test-func".to_string(),
                on: TriggerConfig {
                    http: Some(HttpTrigger {
                        method: "POST".to_string(),
                        path: "/test".to_string(),
                    }),
                    nats: None,
                },
                runtime: "wasi-preview1".to_string(),
                code: "./test.wasm".to_string(),
                timeout: "5s".to_string(),
                memory: "128Mi".to_string(),
                env: std::collections::HashMap::new(),
            }],
        }
    }

    #[test]
    fn test_find_matching_functions() {
        let config = Arc::new(create_test_config());
        let executor = FunctionExecutor::new(config).unwrap();
        
        let matches = executor.find_matching_functions("com.nexus.test.event");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "test-func");
    }
}
