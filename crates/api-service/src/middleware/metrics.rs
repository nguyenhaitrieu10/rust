//! Metrics middleware

use axum::{extract::Request, response::Response};
use metrics::{counter, histogram};
use tower::{Layer, Service};

/// Metrics middleware
#[derive(Clone)]
pub struct MetricsMiddleware;

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for MetricsMiddleware {
    type Service = MetricsMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddlewareService { inner }
    }
}

#[derive(Clone)]
pub struct MetricsMiddlewareService<S> {
    inner: S,
}

impl<S> Service<Request> for MetricsMiddlewareService<S>
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
        let method = request.method().to_string();
        let path = request.uri().path().to_string();
        let start = std::time::Instant::now();

        Box::pin(async move {
            let response = inner.call(request).await?;
            let duration = start.elapsed();
            let status = response.status().as_u16().to_string();

            // Record metrics
            counter!("http_requests_total", 
                "method" => method.clone(), 
                "path" => path.clone(), 
                "status" => status.clone()
            ).increment(1);

            histogram!("http_request_duration_seconds",
                "method" => method,
                "path" => path,
                "status" => status
            ).record(duration.as_secs_f64());

            Ok(response)
        })
    }
}

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}