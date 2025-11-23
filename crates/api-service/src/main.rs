//! API Service - REST API microservice with Axum

use anyhow::Result;
use clap::Parser;
use shared::{AppConfig, ValidateConfig};
use std::net::SocketAddr;
use tracing::{info, warn};

mod config;
mod handlers;
mod middleware;
mod routes;
mod services;
mod state;

use config::ApiConfig;
use state::AppState;

/// API Service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "api-service")]
#[command(about = "REST API microservice")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config")]
    config: String,

    /// Server host
    #[arg(long, env = "API_HOST")]
    host: Option<String>,

    /// Server port
    #[arg(short, long, env = "API_PORT")]
    port: Option<u16>,

    /// Environment
    #[arg(short, long, env = "ENVIRONMENT")]
    environment: Option<String>,
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
    if let Some(host) = args.host {
        config.server.host = host;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(environment) = args.environment {
        config.environment = environment;
    }

    // Validate configuration
    config.validate()?;

    // Initialize logging
    init_logging(&config)?;

    info!("Starting API service");
    info!("Environment: {}", config.environment);
    info!("Version: {}", config.version);

    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;

    // Run database migrations if enabled
    if config.database.migrate_on_start {
        info!("Running database migrations");
        app_state.database().migrate().await?;
    }

    // Build application routes
    let app = routes::create_routes(app_state.clone());

    // Create server address
    let addr: SocketAddr = config.server_address().parse()?;
    info!("Server listening on {}", addr);

    // Start metrics server if enabled
    if config.metrics.enabled {
        let metrics_addr: SocketAddr = config.metrics_address().parse()?;
        tokio::spawn(start_metrics_server(metrics_addr));
        info!("Metrics server listening on {}", metrics_addr);
    }

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("API service stopped");
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

/// Start metrics server
async fn start_metrics_server(addr: SocketAddr) -> Result<()> {
    use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
    use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(handle);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Metrics endpoint handler
async fn metrics_handler(State(handle): State<metrics_exporter_prometheus::PrometheusHandle>) -> impl IntoResponse {
    match handle.render() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
    }
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