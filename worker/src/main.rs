use std::{env, error::Error};

use anyhow::Result;
use queue::job_consumer::JobConsumer;
use rabbitmq::*;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod handlers;
mod queue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env
    dotenvy::dotenv()?;

    // Initialize logging
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    info!("Worker started");

    let mut job_queue = QueueHandler::clone(&CONFIG_QUEUE_JOB);
    job_queue.setup().await?;
    info!("Successfully set up job queue");

    let mut data_queue = QueueHandler::clone(&CONFIG_QUEUE_RESULT);
    data_queue.setup().await?;
    info!("Successfully set up data queue");

    let consumer = JobConsumer::new(data_queue.clone());
    job_queue.subscribe(consumer).await?;
    info!("Successfully started job queue consumer");

    tokio::signal::ctrl_c().await?;
    info!("Received SIGINT signal, Shutting down...");

    // TODO: do not accept new jobs and wait for execution of existing ones
    // TODO: maybe lookup tokio::sync::Notify for this

    job_queue.close().await?;
    data_queue.close().await?;
    info!("Worker shut down gracefully");

    Ok(())
}
