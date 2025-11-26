pub mod tracing_config;

pub use tracing_config::init_tracing;

/// Initialize observability for the application
pub fn setup() -> anyhow::Result<()> {
    init_tracing()?;
    Ok(())
}
