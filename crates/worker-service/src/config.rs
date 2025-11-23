//! Worker service configuration

use shared::AppConfig;
use serde::{Deserialize, Serialize};

/// Worker service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Base application configuration
    #[serde(flatten)]
    pub app: AppConfig,
    
    /// Worker specific settings
    pub worker: WorkerSettings,
}

/// Worker specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSettings {
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Job polling interval in seconds
    pub poll_interval: u64,
    
    /// Maximum jobs to process per batch
    pub batch_size: u32,
    
    /// Job timeout in seconds
    pub job_timeout: u64,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Retry delay in seconds
    pub retry_delay: u64,
    
    /// Enable job metrics
    pub enable_metrics: bool,
    
    /// Job types to process
    pub job_types: Vec<String>,
    
    /// Scheduler settings
    pub scheduler: SchedulerSettings,
}

/// Scheduler settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerSettings {
    /// Enable cron job scheduling
    pub enable_cron: bool,
    
    /// Cron job definitions
    pub cron_jobs: Vec<CronJobConfig>,
    
    /// Cleanup old jobs after days
    pub cleanup_after_days: u32,
    
    /// Enable job history
    pub enable_history: bool,
}

/// Cron job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobConfig {
    /// Job name
    pub name: String,
    
    /// Cron expression
    pub cron: String,
    
    /// Job type
    pub job_type: String,
    
    /// Job payload
    pub payload: serde_json::Value,
    
    /// Enabled flag
    pub enabled: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            worker: WorkerSettings::default(),
        }
    }
}

impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            worker_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            poll_interval: 5,
            batch_size: 10,
            job_timeout: 300,
            max_retries: 3,
            retry_delay: 60,
            enable_metrics: true,
            job_types: vec!["*".to_string()],
            scheduler: SchedulerSettings::default(),
        }
    }
}

impl Default for SchedulerSettings {
    fn default() -> Self {
        Self {
            enable_cron: true,
            cron_jobs: Vec::new(),
            cleanup_after_days: 30,
            enable_history: true,
        }
    }
}

impl WorkerConfig {
    /// Load worker configuration
    pub fn load() -> Result<Self, figment::Error> {
        use figment::{providers::{Env, Format, Yaml}, Figment};
        
        Figment::new()
            .merge(Yaml::file("config/worker.yml"))
            .merge(Yaml::file(format!("config/worker-{}.yml", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()))))
            .merge(Env::prefixed("WORKER_"))
            .extract()
    }
    
    /// Get job timeout as Duration
    pub fn job_timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.worker.job_timeout)
    }
    
    /// Get poll interval as Duration
    pub fn poll_interval_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.worker.poll_interval)
    }
    
    /// Get retry delay as Duration
    pub fn retry_delay_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.worker.retry_delay)
    }
    
    /// Check if job type should be processed
    pub fn should_process_job_type(&self, job_type: &str) -> bool {
        self.worker.job_types.contains(&"*".to_string()) || 
        self.worker.job_types.contains(&job_type.to_string())
    }
    
    /// Validate worker configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate base app config
        self.app.validate()?;
        
        // Validate worker threads
        if self.worker.worker_threads == 0 {
            return Err("Worker threads cannot be zero".to_string());
        }
        
        // Validate batch size
        if self.worker.batch_size == 0 {
            return Err("Batch size cannot be zero".to_string());
        }
        
        // Validate timeouts
        if self.worker.job_timeout == 0 {
            return Err("Job timeout cannot be zero".to_string());
        }
        
        if self.worker.poll_interval == 0 {
            return Err("Poll interval cannot be zero".to_string());
        }
        
        // Validate cron expressions
        for cron_job in &self.worker.scheduler.cron_jobs {
            if cron_job.enabled {
                cron::Schedule::from_str(&cron_job.cron)
                    .map_err(|e| format!("Invalid cron expression '{}': {}", cron_job.cron, e))?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert!(config.worker.worker_threads > 0);
        assert_eq!(config.worker.poll_interval, 5);
        assert_eq!(config.worker.batch_size, 10);
    }

    #[test]
    fn test_worker_config_validation() {
        let mut config = WorkerConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Invalid worker threads should fail
        config.worker.worker_threads = 0;
        assert!(config.validate().is_err());
        
        // Reset and test batch size
        config = WorkerConfig::default();
        config.worker.batch_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_job_type_filtering() {
        let mut config = WorkerConfig::default();
        
        // Should process all job types by default
        assert!(config.should_process_job_type("send_email"));
        assert!(config.should_process_job_type("process_payment"));
        
        // Should process only specific job types
        config.worker.job_types = vec!["send_email".to_string()];
        assert!(config.should_process_job_type("send_email"));
        assert!(!config.should_process_job_type("process_payment"));
    }
}