//! Worker Service - Background job processor

use anyhow::Result;
use clap::Parser;
use shared::{AppConfig, ValidateConfig};
use tracing::{info, warn};

mod config;
mod jobs;
mod processors;
mod scheduler;

use config::WorkerConfig;
use scheduler::JobScheduler;

/// Worker Service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "worker-service")]
#[command(about = "Background job processor")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config")]
    config: String,

    /// Number of worker threads
    #[arg(short, long, env = "WORKER_THREADS")]
    workers: Option<usize>,

    /// Environment
    #[arg(short, long, env = "ENVIRONMENT")]
    environment: Option<String>,

    /// Job types to process (comma-separated)
    #[arg(long, env = "JOB_TYPES")]
    job_types: Option<String>,
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

    // Validate configuration
    config.validate()?;

    // Initialize logging
    init_logging(&config)?;

    info!("Starting Worker service");
    info!("Environment: {}", config.environment);
    info!("Version: {}", config.version);

    // Initialize job scheduler
    let worker_threads = args.workers.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    });

    let job_types = args.job_types
        .map(|types| types.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| vec!["*".to_string()]); // Process all job types by default

    info!("Worker threads: {}", worker_threads);
    info!("Processing job types: {:?}", job_types);

    let scheduler = JobScheduler::new(config, worker_threads, job_types).await?;

    // Start the scheduler
    scheduler.start().await?;

    // Wait for shutdown signal
    shutdown_signal().await;

    // Graceful shutdown
    info!("Shutting down worker service");
    scheduler.shutdown().await?;

    info!("Worker service stopped");
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