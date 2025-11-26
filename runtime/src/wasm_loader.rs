use anyhow::{Context, Result};
use std::path::Path;

/// Loads WASM modules from the filesystem
#[derive(Debug, Clone)]
pub struct WasmLoader;

impl WasmLoader {
    pub fn new() -> Self {
        Self
    }

    /// Load a WASM module from a file path
    pub fn load(&self, path: &str) -> Result<Vec<u8>> {
        let path = Path::new(path);
        
        if !path.exists() {
            anyhow::bail!("WASM module not found: {}", path.display());
        }

        if !path.extension().map(|e| e == "wasm").unwrap_or(false) {
            anyhow::bail!("File must have .wasm extension: {}", path.display());
        }

        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read WASM module: {}", path.display()))?;

        // Validate it's a valid WASM module (check magic number)
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM module: missing magic number");
        }

        Ok(bytes)
    }
}

impl Default for WasmLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_path() {
        let loader = WasmLoader::new();
        let result = loader.load("/nonexistent/module.wasm");
        assert!(result.is_err());
    }
}
