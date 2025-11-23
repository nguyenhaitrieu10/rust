//! Job processors

use async_trait::async_trait;
use shared::{AppResult, CorrelationId};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Job processor trait
#[async_trait]
pub trait Processor: Send + Sync {
    async fn process(&self, job_type: &str, payload: serde_json::Value, correlation_id: CorrelationId) -> AppResult<serde_json::Value>;
}

/// Default job processor implementation
pub struct DefaultProcessor;

#[async_trait]
impl Processor for DefaultProcessor {
    async fn process(&self, job_type: &str, payload: serde_json::Value, correlation_id: CorrelationId) -> AppResult<serde_json::Value> {
        info!("Processing job: type={}, correlation_id={}", job_type, correlation_id);
        
        match job_type {
            "send_email" => process_email_job(payload).await,
            "process_payment" => process_payment_job(payload).await,
            "generate_report" => process_report_job(payload).await,
            "cleanup_data" => process_cleanup_job(payload).await,
            _ => {
                warn!("Unknown job type: {}", job_type);
                Err(shared::AppError::BadRequest(format!("Unknown job type: {}", job_type)))
            }
        }
    }
}

/// Process email job
async fn process_email_job(payload: serde_json::Value) -> AppResult<serde_json::Value> {
    // TODO: Implement actual email sending logic
    info!("Processing email job with payload: {:?}", payload);
    
    // Simulate email processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    Ok(serde_json::json!({
        "status": "sent",
        "timestamp": chrono::Utc::now()
    }))
}

/// Process payment job
async fn process_payment_job(payload: serde_json::Value) -> AppResult<serde_json::Value> {
    // TODO: Implement actual payment processing logic
    info!("Processing payment job with payload: {:?}", payload);
    
    // Simulate payment processing
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    Ok(serde_json::json!({
        "status": "processed",
        "timestamp": chrono::Utc::now()
    }))
}

/// Process report generation job
async fn process_report_job(payload: serde_json::Value) -> AppResult<serde_json::Value> {
    // TODO: Implement actual report generation logic
    info!("Processing report job with payload: {:?}", payload);
    
    // Simulate report generation
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    Ok(serde_json::json!({
        "status": "generated",
        "timestamp": chrono::Utc::now()
    }))
}

/// Process data cleanup job
async fn process_cleanup_job(payload: serde_json::Value) -> AppResult<serde_json::Value> {
    // TODO: Implement actual cleanup logic
    info!("Processing cleanup job with payload: {:?}", payload);
    
    // Simulate cleanup processing
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    Ok(serde_json::json!({
        "status": "cleaned",
        "timestamp": chrono::Utc::now()
    }))
}

/// Job execution context
pub struct JobContext {
    pub job_id: uuid::Uuid,
    pub job_type: String,
    pub correlation_id: CorrelationId,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_duration: Duration,
}

/// Job executor with timeout and retry logic
pub struct JobExecutor<P: Processor> {
    processor: P,
}

impl<P: Processor> JobExecutor<P> {
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
    
    /// Execute job with timeout and error handling
    pub async fn execute(&self, context: JobContext, payload: serde_json::Value) -> AppResult<serde_json::Value> {
        let start_time = std::time::Instant::now();
        
        info!(
            "Executing job: id={}, type={}, retry={}/{}",
            context.job_id, context.job_type, context.retry_count, context.max_retries
        );
        
        // Execute with timeout
        let result = timeout(
            context.timeout_duration,
            self.processor.process(&context.job_type, payload, context.correlation_id)
        ).await;
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(Ok(result)) => {
                info!(
                    "Job completed successfully: id={}, duration={:?}",
                    context.job_id, duration
                );
                Ok(result)
            }
            Ok(Err(e)) => {
                error!(
                    "Job failed: id={}, error={}, duration={:?}",
                    context.job_id, e, duration
                );
                Err(e)
            }
            Err(_) => {
                error!(
                    "Job timed out: id={}, timeout={:?}, duration={:?}",
                    context.job_id, context.timeout_duration, duration
                );
                Err(shared::AppError::Internal("Job execution timed out".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_default_processor() {
        let processor = DefaultProcessor;
        let correlation_id = Uuid::new_v4();
        
        let result = processor.process(
            "send_email",
            serde_json::json!({"to": "test@example.com"}),
            correlation_id
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_job_executor() {
        let processor = DefaultProcessor;
        let executor = JobExecutor::new(processor);
        
        let context = JobContext {
            job_id: Uuid::new_v4(),
            job_type: "send_email".to_string(),
            correlation_id: Uuid::new_v4(),
            retry_count: 0,
            max_retries: 3,
            timeout_duration: Duration::from_secs(30),
        };
        
        let result = executor.execute(
            context,
            serde_json::json!({"to": "test@example.com"})
        ).await;
        
        assert!(result.is_ok());
    }
}