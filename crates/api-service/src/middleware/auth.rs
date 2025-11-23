//! Authentication middleware

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tower::{Layer, Service};

use crate::state::AppState;

/// Authentication middleware
#[derive(Clone)]
pub struct AuthMiddleware {
    state: AppState,
}

impl AuthMiddleware {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for AuthMiddleware {
    type Service = AuthMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddlewareService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddlewareService<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request> for AuthMiddlewareService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();

        Box::pin(async move {
            // TODO: Implement JWT token validation
            // For now, just pass through
            inner.call(request).await
        })
    }
}

/// Authentication handler function (alternative approach)
pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement authentication logic
    // Extract and validate JWT token from Authorization header
    // Add user context to request extensions
    
    Ok(next.run(request).await)
}