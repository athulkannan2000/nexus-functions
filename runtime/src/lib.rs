pub mod wasm_loader;
pub mod wasm_executor;

pub use wasm_loader::WasmLoader;
pub use wasm_executor::WasmExecutor;

use anyhow::Result;

/// WASM runtime interface
#[derive(Debug, Clone)]
pub struct Runtime {
    loader: WasmLoader,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            loader: WasmLoader::new(),
        }
    }

    /// Load a WASM module from a file path
    pub fn load_module(&self, path: &str) -> Result<Vec<u8>> {
        self.loader.load(path)
    }

    /// Execute a WASM function with input data
    pub async fn execute(&self, module_bytes: &[u8], input: &[u8]) -> Result<Vec<u8>> {
        let executor = WasmExecutor::new()?;
        executor.execute(module_bytes, input).await
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
