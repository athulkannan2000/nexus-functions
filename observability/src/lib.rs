pub mod tracing_config;
pub mod context;

pub use tracing_config::{init_tracing, init_tracing_json};
pub use context::{RequestContext, with_context, get_trace_id};

/// Initialize observability for the application
pub fn setup() -> anyhow::Result<()> {
    init_tracing_json()?;
    tracing::info!("Observability initialized with structured JSON logging");
    Ok(())
}
