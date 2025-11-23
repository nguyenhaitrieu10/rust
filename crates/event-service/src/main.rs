//! Event Service - Kafka event streaming service

use anyhow::Result;
use clap::Parser;
use shared::{AppConfig, ValidateConfig};
use tracing::{info, warn};

mod config;
mod consumers;
mod producers;
mod handlers;

use config::EventConfig;
use consumers::EventConsumerManager;
use producers::EventProducerManager;

/// Event Service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "event-service")]
#[command(about = "Kafka event streaming service")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config")]
    config: String,

    /// Consumer group ID
    #[arg(long, env = "KAFKA_GROUP_ID")]
    group_id: Option<String>,

    /// Topics to consume (comma-separated)
    #[arg(long, env = "KAFKA_TOPICS")]
    topics: Option<String>,

    /// Environment
    #[arg(short, long, env = "ENVIRONMENT")]
    environment: Option<String>,

    /// Enable producer mode
    #[arg(long)]
    producer: bool,

    /// Enable consumer mode
    #[arg(long)]
    consumer: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Initialize configuration
    let mut config = if args.config == "config" {
        AppConfig::load()?
    } else {
        AppConfig::load_from_path(&args.config)?
    };

    // Override config with CLI arguments
    if let Some(environment) = args.environment {
        config.environment = environment;
    }

    if let Some(group_id) = args.group_id {
        config.kafka.group_id = group_id;
    }

    // Validate configuration
    config.validate()?;

    // Initialize logging
    init_logging(&config)?;

    info!("Starting Event service");
    info!("Environment: {}", config.environment);
    info!("Version: {}", config.version);

    let topics = args.topics
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| vec!["events".to_string()]);

    info!("Kafka topics: {:?}", topics);

    // Determine mode
    let enable_producer = args.producer || (!args.consumer && !args.producer);
    let enable_consumer = args.consumer || (!args.consumer && !args.producer);

    let mut handles = Vec::new();

    // Start producer if enabled
    if enable_producer {
        info!("Starting Kafka producer");
        let producer_manager = EventProducerManager::new(&config.kafka).await?;
        let producer_handle = tokio::spawn(async move {
            if let Err(e) = producer_manager.start().await {
                tracing::error!("Producer manager failed: {}", e);
            }
        });
        handles.push(producer_handle);
    }

    // Start consumer if enabled
    if enable_consumer {
        info!("Starting Kafka consumer for topics: {:?}", topics);
        let consumer_manager = EventConsumerManager::new(&config.kafka, topics).await?;
        let consumer_handle = tokio::spawn(async move {
            if let Err(e) = consumer_manager.start().await {
                tracing::error!("Consumer manager failed: {}", e);
            }
        });
        handles.push(consumer_handle);
    }

    // Wait for shutdown signal
    shutdown_signal().await;

    // Graceful shutdown
    info!("Shutting down event service");
    for handle in handles {
        handle.abort();
    }

    info!("Event service stopped");
    Ok(())
}

/// Initialize logging and tracing
fn init_logging(config: &AppConfig) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match config.logging.format.as_str() {
        "json" => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true);
            subscriber.with(json_layer).init();
        }
        _ => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true);
            subscriber.with(fmt_layer).init();
        }
    }

    Ok(())
}

/// Graceful shutdown signal
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }

    info!("Starting graceful shutdown");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }
}