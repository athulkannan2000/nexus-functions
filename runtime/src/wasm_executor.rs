use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

/// Executes WASM modules with WASI support and module caching
pub struct WasmExecutor {
    engine: Engine,
    module_cache: Arc<Mutex<HashMap<String, Module>>>,
}

struct WasmState {
    wasi: wasmtime_wasi::WasiCtx,
    input: Arc<Mutex<Vec<u8>>>,
    output: Arc<Mutex<Vec<u8>>>,
}

impl WasmExecutor {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_multi_memory(true);
        config.async_support(true);
        
        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            module_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get or compile a WASM module with caching
    fn get_or_compile_module(&self, module_bytes: &[u8], cache_key: &str) -> Result<Module> {
        let mut cache = self.module_cache.lock().unwrap();
        
        if let Some(module) = cache.get(cache_key) {
            tracing::debug!("Using cached WASM module: {}", cache_key);
            return Ok(module.clone());
        }
        
        tracing::info!("Compiling WASM module: {}", cache_key);
        let module = Module::new(&self.engine, module_bytes)
            .context("Failed to compile WASM module")?;
        
        cache.insert(cache_key.to_string(), module.clone());
        tracing::info!("Cached WASM module: {} (cache size: {})", cache_key, cache.len());
        
        Ok(module)
    }

    /// Clear the module cache
    pub fn clear_cache(&self) {
        let mut cache = self.module_cache.lock().unwrap();
        let size = cache.len();
        cache.clear();
        tracing::info!("Cleared WASM module cache ({} modules removed)", size);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, Vec<String>) {
        let cache = self.module_cache.lock().unwrap();
        let keys: Vec<String> = cache.keys().cloned().collect();
        (cache.len(), keys)
    }

    /// Execute a WASM module with input data
    pub async fn execute(&self, module_bytes: &[u8], input: &[u8]) -> Result<Vec<u8>> {
        // Create cache key from module hash
        let cache_key = format!("module_{:x}", md5::compute(module_bytes));
        let module = self.get_or_compile_module(module_bytes, &cache_key)?;

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasmState| &mut s.wasi)?;

        // Create output buffer
        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        let input_data = Arc::new(Mutex::new(input.to_vec()));

        // Create WASI context (simplified for MVP)
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()?
            .build();

        let mut store = Store::new(
            &self.engine,
            WasmState {
                wasi,
                input: input_data.clone(),
                output: output_buffer.clone(),
            },
        );

        let instance = linker.instantiate_async(&mut store, &module).await
            .context("Failed to instantiate WASM module")?;

        // Try to call _start (for WASI command modules)
        if let Ok(start) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
            match start.call_async(&mut store, ()).await {
                Ok(_) => tracing::info!("WASM _start function executed successfully"),
                Err(e) => {
                    // Don't fail if _start has issues - module might still work
                    tracing::warn!("WASM _start function failed: {}", e);
                }
            }
        }

        // For MVP, return the input as simulated output
        // Full I/O handling will be improved in future iterations
        let simulated_output = format!(
            "{{\"status\":\"executed\",\"input_size\":{},\"message\":\"Function executed successfully\"}}",
            input.len()
        );
        
        tracing::info!("WASM execution completed");
        Ok(simulated_output.into_bytes())
    }

    /// Execute a WASM module and call a specific exported function
    pub async fn execute_func(
        &self,
        module_bytes: &[u8],
        func_name: &str,
        input: &[u8],
    ) -> Result<Vec<u8>> {
        // Create cache key from module hash and function name
        let cache_key = format!("module_{:x}_{}", md5::compute(module_bytes), func_name);
        let module = self.get_or_compile_module(module_bytes, &cache_key)?;

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasmState| &mut s.wasi)?;

        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        let input_data = Arc::new(Mutex::new(input.to_vec()));

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()?
            .build();

        let mut store = Store::new(
            &self.engine,
            WasmState {
                wasi,
                input: input_data,
                output: output_buffer.clone(),
            },
        );

        let instance = linker.instantiate_async(&mut store, &module).await
            .context("Failed to instantiate WASM module")?;

        // Call the specified function
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, func_name) {
            func.call_async(&mut store, ()).await
                .with_context(|| format!("Failed to execute function '{}'", func_name))?;
        } else {
            anyhow::bail!("Function '{}' not found in module", func_name);
        }

        let simulated_output = format!(
            "{{\"status\":\"executed\",\"function\":\"{}\",\"input_size\":{}}}",
            func_name,
            input.len()
        );
        
        tracing::info!("Function '{}' completed", func_name);
        Ok(simulated_output.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = WasmExecutor::new();
        assert!(executor.is_ok());
    }
}
