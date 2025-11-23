//! Logging middleware

use axum::{extract::Request, response::Response};
use tower::{Layer, Service};
use tracing::{info, warn};

/// Logging middleware
#[derive(Clone)]
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LoggingMiddleware {
    type Service = LoggingMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddlewareService { inner }
    }
}

#[derive(Clone)]
pub struct LoggingMiddlewareService<S> {
    inner: S,
}

impl<S> Service<Request> for LoggingMiddlewareService<S>
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
        let method = request.method().clone();
        let uri = request.uri().clone();
        let start = std::time::Instant::now();

        Box::pin(async move {
            let response = inner.call(request).await?;
            let duration = start.elapsed();
            let status = response.status();

            if status.is_success() {
                info!(
                    method = %method,
                    uri = %uri,
                    status = %status,
                    duration_ms = duration.as_millis(),
                    "Request completed"
                );
            } else {
                warn!(
                    method = %method,
                    uri = %uri,
                    status = %status,
                    duration_ms = duration.as_millis(),
                    "Request failed"
                );
            }

            Ok(response)
        })
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}