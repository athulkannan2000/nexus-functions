use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusConfig {
    pub version: String,
    pub functions: Vec<FunctionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionConfig {
    pub name: String,
    pub on: TriggerConfig,
    pub runtime: String,
    pub code: String,
    #[serde(default = "default_timeout")]
    pub timeout: String,
    #[serde(default = "default_memory")]
    pub memory: String,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nats: Option<NatsTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTrigger {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsTrigger {
    pub subject: String,
}

fn default_timeout() -> String {
    "5s".to_string()
}

fn default_memory() -> String {
    "128Mi".to_string()
}

impl NexusConfig {
    /// Load configuration from a YAML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read configuration file")?;
        
        Self::from_str(&content)
    }
    
    /// Parse configuration from a YAML string
    pub fn from_str(content: &str) -> Result<Self> {
        let config: NexusConfig = serde_yaml::from_str(content)
            .context("Failed to parse configuration YAML")?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Check version
        if self.version != "v1" {
            anyhow::bail!("Unsupported configuration version: {}", self.version);
        }
        
        // Check for duplicate function names
        let mut names = std::collections::HashSet::new();
        for func in &self.functions {
            if !names.insert(&func.name) {
                anyhow::bail!("Duplicate function name: {}", func.name);
            }
        }
        
        // Validate each function
        for func in &self.functions {
            func.validate()?;
        }
        
        Ok(())
    }
}

impl FunctionConfig {
    fn validate(&self) -> Result<()> {
        // Validate runtime
        let valid_runtimes = ["wasi-preview1", "wasi-preview2"];
        if !valid_runtimes.contains(&self.runtime.as_str()) {
            anyhow::bail!(
                "Invalid runtime '{}' for function '{}'. Valid options: {}",
                self.runtime,
                self.name,
                valid_runtimes.join(", ")
            );
        }
        
        // Validate code path
        if self.code.is_empty() {
            anyhow::bail!("Function '{}' has empty code path", self.name);
        }
        
        // Validate trigger
        if self.on.http.is_none() && self.on.nats.is_none() {
            anyhow::bail!(
                "Function '{}' must have at least one trigger (http or nats)",
                self.name
            );
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
version: v1
functions:
  - name: hello-world
    on:
      http:
        method: POST
        path: /events/hello
    runtime: wasi-preview1
    code: ./build/handler.wasm
    timeout: 5s
    memory: 128Mi
"#;
        let config = NexusConfig::from_str(yaml).unwrap();
        assert_eq!(config.version, "v1");
        assert_eq!(config.functions.len(), 1);
        assert_eq!(config.functions[0].name, "hello-world");
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
version: v2
functions: []
"#;
        let result = NexusConfig::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_function_names() {
        let yaml = r#"
version: v1
functions:
  - name: test
    on:
      http:
        method: POST
        path: /test
    runtime: wasi-preview1
    code: ./test.wasm
  - name: test
    on:
      http:
        method: POST
        path: /test2
    runtime: wasi-preview1
    code: ./test2.wasm
"#;
        let result = NexusConfig::from_str(yaml);
        assert!(result.is_err());
    }
}
