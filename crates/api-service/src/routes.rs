//! API routes configuration

use axum::{
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::{
    handlers::{auth, health, users},
    middleware::{auth::AuthMiddleware, logging::LoggingMiddleware, metrics::MetricsMiddleware},
    state::AppState,
};

/// Create application routes
pub fn create_routes(state: AppState) -> Router {
    let config = state.config();

    // Create middleware stack
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(config.server.client_timeout.into()))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
        .layer(MetricsMiddleware::new())
        .layer(LoggingMiddleware::new())
        .layer(CorsLayer::permissive()); // Configure CORS as needed

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check));

    // Authentication routes (no auth required)
    let auth_routes = Router::new()
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/logout", post(auth::logout));

    // Protected API routes (auth required)
    let api_routes = Router::new()
        .route("/users", get(users::list_users).post(users::create_user))
        .route("/users/:id", get(users::get_user).put(users::update_user).delete(users::delete_user))
        .route("/users/:id/profile", get(users::get_user_profile).put(users::update_user_profile))
        .layer(AuthMiddleware::new(state.clone()));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .nest("/api/v1", Router::new()
            .merge(auth_routes)
            .merge(api_routes)
        )
        .layer(middleware)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use shared::AppConfig;

    #[tokio::test]
    async fn test_health_routes() {
        let config = AppConfig::default();
        
        // This test would require running database and Redis instances
        // In a real test environment, you would use testcontainers
        // let state = AppState::new(config).await.unwrap();
        // let app = create_routes(state);
        // let server = TestServer::new(app).unwrap();
        
        // let response = server.get("/health").await;
        // assert_eq!(response.status_code(), StatusCode::OK);
    }
}