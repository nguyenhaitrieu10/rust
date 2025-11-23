//! Application constants

/// Default pagination limit
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Maximum pagination limit
pub const MAX_PAGE_SIZE: u32 = 1000;

/// Default cache TTL in seconds
pub const DEFAULT_CACHE_TTL: u64 = 3600;

/// JWT token header name
pub const JWT_HEADER: &str = "Authorization";

/// JWT token prefix
pub const JWT_PREFIX: &str = "Bearer ";

/// Correlation ID header name
pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Tenant ID header name
pub const TENANT_ID_HEADER: &str = "X-Tenant-ID";

/// User ID header name
pub const USER_ID_HEADER: &str = "X-User-ID";

/// API version header name
pub const API_VERSION_HEADER: &str = "X-API-Version";

/// Content type JSON
pub const CONTENT_TYPE_JSON: &str = "application/json";

/// Content type form
pub const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";

/// Health check endpoint
pub const HEALTH_ENDPOINT: &str = "/health";

/// Metrics endpoint
pub const METRICS_ENDPOINT: &str = "/metrics";

/// Ready endpoint
pub const READY_ENDPOINT: &str = "/ready";

/// Live endpoint
pub const LIVE_ENDPOINT: &str = "/live";

/// API base path
pub const API_BASE_PATH: &str = "/api/v1";

/// Default database schema
pub const DEFAULT_SCHEMA: &str = "public";

/// Default Kafka topic prefix
pub const KAFKA_TOPIC_PREFIX: &str = "app";

/// Event types
pub mod events {
    pub const USER_CREATED: &str = "user.created";
    pub const USER_UPDATED: &str = "user.updated";
    pub const USER_DELETED: &str = "user.deleted";
    pub const ORDER_CREATED: &str = "order.created";
    pub const ORDER_UPDATED: &str = "order.updated";
    pub const ORDER_CANCELLED: &str = "order.cancelled";
    pub const PAYMENT_PROCESSED: &str = "payment.processed";
    pub const PAYMENT_FAILED: &str = "payment.failed";
}

/// Job types
pub mod jobs {
    pub const SEND_EMAIL: &str = "send_email";
    pub const PROCESS_PAYMENT: &str = "process_payment";
    pub const GENERATE_REPORT: &str = "generate_report";
    pub const CLEANUP_DATA: &str = "cleanup_data";
    pub const SYNC_DATA: &str = "sync_data";
}

/// Cache key prefixes
pub mod cache_keys {
    pub const USER: &str = "user";
    pub const SESSION: &str = "session";
    pub const RATE_LIMIT: &str = "rate_limit";
    pub const CONFIG: &str = "config";
    pub const METRICS: &str = "metrics";
}

/// Database table names
pub mod tables {
    pub const USERS: &str = "users";
    pub const ORDERS: &str = "orders";
    pub const PAYMENTS: &str = "payments";
    pub const JOBS: &str = "jobs";
    pub const EVENTS: &str = "events";
}

/// Environment names
pub mod environments {
    pub const DEVELOPMENT: &str = "development";
    pub const STAGING: &str = "staging";
    pub const PRODUCTION: &str = "production";
    pub const TEST: &str = "test";
}

/// Service names
pub mod services {
    pub const API: &str = "api-service";
    pub const WORKER: &str = "worker-service";
    pub const EVENT: &str = "event-service";
}

/// Metric names
pub mod metrics {
    pub const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
    pub const HTTP_REQUEST_DURATION: &str = "http_request_duration_seconds";
    pub const DATABASE_CONNECTIONS: &str = "database_connections";
    pub const REDIS_CONNECTIONS: &str = "redis_connections";
    pub const KAFKA_MESSAGES_PRODUCED: &str = "kafka_messages_produced_total";
    pub const KAFKA_MESSAGES_CONSUMED: &str = "kafka_messages_consumed_total";
    pub const JOBS_PROCESSED: &str = "jobs_processed_total";
    pub const JOBS_FAILED: &str = "jobs_failed_total";
}

/// Log levels
pub mod log_levels {
    pub const TRACE: &str = "trace";
    pub const DEBUG: &str = "debug";
    pub const INFO: &str = "info";
    pub const WARN: &str = "warn";
    pub const ERROR: &str = "error";
}

/// HTTP status codes
pub mod status_codes {
    pub const OK: u16 = 200;
    pub const CREATED: u16 = 201;
    pub const NO_CONTENT: u16 = 204;
    pub const BAD_REQUEST: u16 = 400;
    pub const UNAUTHORIZED: u16 = 401;
    pub const FORBIDDEN: u16 = 403;
    pub const NOT_FOUND: u16 = 404;
    pub const CONFLICT: u16 = 409;
    pub const UNPROCESSABLE_ENTITY: u16 = 422;
    pub const TOO_MANY_REQUESTS: u16 = 429;
    pub const INTERNAL_SERVER_ERROR: u16 = 500;
    pub const BAD_GATEWAY: u16 = 502;
    pub const SERVICE_UNAVAILABLE: u16 = 503;
}

/// Timeouts in seconds
pub mod timeouts {
    pub const HTTP_CLIENT: u64 = 30;
    pub const DATABASE_QUERY: u64 = 30;
    pub const REDIS_OPERATION: u64 = 5;
    pub const KAFKA_PRODUCE: u64 = 10;
    pub const JOB_EXECUTION: u64 = 300;
}

/// Retry configurations
pub mod retries {
    pub const MAX_ATTEMPTS: u32 = 3;
    pub const INITIAL_DELAY_MS: u64 = 1000;
    pub const MAX_DELAY_MS: u64 = 30000;
    pub const BACKOFF_MULTIPLIER: f64 = 2.0;
}

/// Rate limiting
pub mod rate_limits {
    pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 60;
    pub const AUTH_REQUESTS_PER_MINUTE: u32 = 10;
    pub const API_REQUESTS_PER_MINUTE: u32 = 1000;
}

/// File size limits
pub mod file_limits {
    pub const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024; // 10MB
    pub const MAX_JSON_PAYLOAD: usize = 1024 * 1024; // 1MB
}

/// Validation rules
pub mod validation {
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    pub const MIN_USERNAME_LENGTH: usize = 3;
    pub const MAX_USERNAME_LENGTH: usize = 50;
    pub const MAX_EMAIL_LENGTH: usize = 254;
    pub const MAX_NAME_LENGTH: usize = 100;
}