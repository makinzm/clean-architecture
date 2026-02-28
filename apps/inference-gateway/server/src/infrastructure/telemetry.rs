use anyhow::Result;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

pub fn init_tracer() -> Result<()> {
    let format_layer = tracing_subscriber::fmt::layer().with_level(true).json();

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = Registry::default().with(filter_layer).with(format_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    Ok(())
}
