//! Job scheduler

use crate::{config::WorkerConfig, processors::{DefaultProcessor, JobExecutor, JobContext, Processor}};
use database::{DatabaseManager, JobRepository};
use shared::{AppResult, CorrelationId};
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::interval};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Job scheduler for managing background job processing
pub struct JobScheduler {
    config: WorkerConfig,
    database: DatabaseManager,
    job_repository: JobRepository,
    executor: JobExecutor<DefaultProcessor>,
    running: Arc<RwLock<bool>>,
    worker_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub async fn new(
        config: shared::AppConfig,
        worker_threads: usize,
        job_types: Vec<String>,
    ) -> AppResult<Self> {
        let database = DatabaseManager::new(&config.database).await?;
        let job_repository = JobRepository::new(database.pool().clone());
        let executor = JobExecutor::new(DefaultProcessor);
        
        let worker_config = WorkerConfig {
            app: config,
            worker: crate::config::WorkerSettings {
                worker_threads,
                job_types,
                ..Default::default()
            },
        };

        Ok(Self {
            config: worker_config,
            database,
            job_repository,
            executor,
            running: Arc::new(RwLock::new(false)),
            worker_handles: Vec::new(),
        })
    }

    /// Start the job scheduler
    pub async fn start(&mut self) -> AppResult<()> {
        info!("Starting job scheduler with {} worker threads", self.config.worker.worker_threads);
        
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start worker threads
        for worker_id in 0..self.config.worker.worker_threads {
            let handle = self.spawn_worker(worker_id).await;
            self.worker_handles.push(handle);
        }

        // Start cron scheduler if enabled
        if self.config.worker.scheduler.enable_cron {
            let handle = self.spawn_cron_scheduler().await;
            self.worker_handles.push(handle);
        }

        // Start cleanup task
        let handle = self.spawn_cleanup_task().await;
        self.worker_handles.push(handle);

        info!("Job scheduler started successfully");
        Ok(())
    }

    /// Stop the job scheduler
    pub async fn shutdown(&self) -> AppResult<()> {
        info!("Shutting down job scheduler");
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Wait for all workers to finish
        for handle in &self.worker_handles {
            handle.abort();
        }

        info!("Job scheduler stopped");
        Ok(())
    }

    /// Spawn a worker thread
    async fn spawn_worker(&self, worker_id: usize) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let job_repository = self.job_repository.clone();
        let executor = JobExecutor::new(DefaultProcessor);
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("Worker {} started", worker_id);
            
            let mut poll_interval = interval(config.poll_interval_duration());
            
            loop {
                // Check if we should continue running
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        break;
                    }
                }

                poll_interval.tick().await;

                // Fetch pending jobs
                match job_repository.find_pending(config.worker.batch_size as i64).await {
                    Ok(jobs) => {
                        for job in jobs {
                            // Check if we should process this job type
                            if !config.should_process_job_type(&job.job_type) {
                                continue;
                            }

                            // Mark job as started
                            if let Err(e) = job_repository.mark_started(&job.id).await {
                                error!("Failed to mark job as started: {}", e);
                                continue;
                            }

                            let context = JobContext {
                                job_id: job.id,
                                job_type: job.job_type.clone(),
                                correlation_id: Uuid::new_v4(), // TODO: Use actual correlation ID
                                retry_count: job.retry_count as u32,
                                max_retries: job.max_retries as u32,
                                timeout_duration: config.job_timeout_duration(),
                            };

                            // Execute the job
                            match executor.execute(context, job.payload).await {
                                Ok(result) => {
                                    if let Err(e) = job_repository.mark_completed(&job.id, Some(result)).await {
                                        error!("Failed to mark job as completed: {}", e);
                                    }
                                }
                                Err(e) => {
                                    let error_msg = e.to_string();
                                    if job.retry_count < job.max_retries {
                                        // Schedule retry
                                        warn!("Job failed, will retry: id={}, error={}", job.id, error_msg);
                                        // TODO: Implement retry scheduling with delay
                                    } else {
                                        // Mark as failed
                                        error!("Job failed permanently: id={}, error={}", job.id, error_msg);
                                        if let Err(e) = job_repository.mark_failed(&job.id, &error_msg).await {
                                            error!("Failed to mark job as failed: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch pending jobs: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }

            info!("Worker {} stopped", worker_id);
        })
    }

    /// Spawn cron scheduler
    async fn spawn_cron_scheduler(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("Cron scheduler started");
            
            let mut check_interval = interval(Duration::from_secs(60)); // Check every minute
            
            loop {
                // Check if we should continue running
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        break;
                    }
                }

                check_interval.tick().await;

                // TODO: Implement cron job scheduling
                // Check each cron job definition and schedule if due
                for cron_job in &config.worker.scheduler.cron_jobs {
                    if cron_job.enabled {
                        // Parse cron expression and check if job should run
                        // Create job entry in database if due
                    }
                }
            }

            info!("Cron scheduler stopped");
        })
    }

    /// Spawn cleanup task
    async fn spawn_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("Cleanup task started");
            
            let mut cleanup_interval = interval(Duration::from_secs(3600)); // Run every hour
            
            loop {
                // Check if we should continue running
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        break;
                    }
                }

                cleanup_interval.tick().await;

                // TODO: Implement job cleanup logic
                // Remove old completed/failed jobs based on configuration
                info!("Running job cleanup task");
            }

            info!("Cleanup task stopped");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_job_scheduler_creation() {
        let config = shared::AppConfig::default();
        
        // This test would require a running database
        // let scheduler = JobScheduler::new(config, 2, vec!["test".to_string()]).await;
        // assert!(scheduler.is_ok());
    }
}