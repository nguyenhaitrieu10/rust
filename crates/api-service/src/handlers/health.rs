//! Health check handlers

use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use shared::{HealthStatus, ServiceStatus};

use crate::state::AppState;

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config();
    
    // Check database health
    let db_health = match state.database().health_check().await {
        Ok(health) => health,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };
    
    // Check Redis health
    let redis_health = match state.cache().health_check().await {
        Ok(health) => health,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };
    
    // Determine overall status
    let overall_status = if matches!(db_health.status, shared::database::HealthStatus::Healthy) 
        && matches!(redis_health.status, shared::cache::HealthStatus::Healthy) {
        ServiceStatus::Healthy
    } else {
        ServiceStatus::Unhealthy
    };
    
    let response = json!({
        "service": config.service_name,
        "version": config.version,
        "status": overall_status,
        "timestamp": chrono::Utc::now(),
        "dependencies": {
            "database": {
                "status": db_health.status,
                "response_time_ms": db_health.response_time_ms
            },
            "redis": {
                "status": redis_health.status,
                "response_time_ms": redis_health.response_time_ms
            }
        }
    });
    
    match overall_status {
        ServiceStatus::Healthy => Ok(Json(response)),
        _ => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Readiness check endpoint
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Check if service is ready to accept traffic
    let config = state.config();
    
    let response = json!({
        "service": config.service_name,
        "status": "ready",
        "timestamp": chrono::Utc::now()
    });
    
    Ok(Json(response))
}

/// Liveness check endpoint
pub async fn liveness_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Check if service is alive
    let config = state.config();
    
    let response = json!({
        "service": config.service_name,
        "status": "alive",
        "timestamp": chrono::Utc::now()
    });
    
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_response_format() {
        // Test would require mock state
        // This is a placeholder for the response format test
    }
}