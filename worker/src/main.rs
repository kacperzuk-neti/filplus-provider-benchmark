use std::{env, error::Error};

use anyhow::Result;
use once_cell::sync::Lazy;
use queue::{job_consumer::JobConsumer, status_sender::StatusSender};
use rabbitmq::*;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod handlers;
mod queue;

// global worker name, initialized once on first access
pub static GLOBAL_WORKER_NAME: Lazy<String> =
    Lazy::new(|| env::var("WORKER_NAME").expect("WORKER_NAME not set"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env
    dotenvy::dotenv()
        .inspect_err(|_| eprintln!("Failed to read .env file, ignoring."))
        .ok();

    // Initialize logging
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    info!("Worker started {}", *GLOBAL_WORKER_NAME);

    let mut job_queue = QueueHandler::clone(&CONFIG_QUEUE_JOB);
    job_queue.setup().await?;
    info!("Successfully set up job queue");

    let mut data_queue = QueueHandler::clone(&CONFIG_QUEUE_RESULT);
    data_queue.setup().await?;
    info!("Successfully set up data queue");

    let mut status_queue = QueueHandler::clone(&CONFIG_QUEUE_STATUS);
    status_queue.setup().await?;
    info!("Successfully set up status queue");
    let status_sender = StatusSender::new(status_queue.clone());

    status_sender
        .send_lifecycle_status(WorkerStatus::Online)
        .await?;

    // Spawn the background task to send heartbeat status
    tokio::spawn(send_heartbeat_status(status_sender.clone()));

    let consumer = JobConsumer::new(data_queue.clone(), status_sender.clone());
    job_queue.subscribe(consumer).await?;
    info!("Successfully started job queue consumer");

    tokio::signal::ctrl_c().await?;
    info!("Received SIGINT signal, Shutting down...");

    // TODO: do not accept new jobs and wait for execution of existing ones
    // TODO: maybe lookup tokio::sync::Notify for this

    job_queue.close().await?;
    data_queue.close().await?;
    status_sender
        .send_lifecycle_status(WorkerStatus::Offline)
        .await?;
    status_queue.close().await?;
    info!("Worker shut down gracefully");

    Ok(())
}

/// Sends heartbeat status to scheduler every interval
async fn send_heartbeat_status(status_sender: StatusSender) {
    let interval_secs: u64 = env::var("HEARTBEAT_INTERVAL_SEC")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .expect("Invalid HEARTBEAT_INTERVAL value");

    let mut interval = interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;
        if let Err(e) = status_sender.send_heartbeat_status().await {
            error!("Error sending heartbeat status: {}", e);
        }
    }
}
