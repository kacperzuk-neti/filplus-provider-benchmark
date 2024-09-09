use dotenv::dotenv;
use handlers::{job_consumer::JobConsumer, result_sender};
use rabbitmq::*;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod handlers;

// TODO: make sure that the worker can and will exit gracefully
#[tokio::main]
async fn main() {
    // Load .env
    dotenv().ok();

    // Initialize logging
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    info!("Worker started");

    // TODO: i need to make sure that before exit the worker, all the messages are processed
    let (result_sender, result_receiver) = mpsc::channel::<Message>(100); // TODO: not a string !

    let addr = Arc::new(std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()));

    let mut job_queue = QueueHandler::clone(&CONFIG_WORKER_A_JOB);
    match job_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up job queue"),
        Err(e) => error!("Failed to set up job queue: {}", e), // TODO: panic ?!
    }

    // Handle results in a separate task
    tokio::spawn(result_sender::handle(result_receiver, Arc::clone(&addr)));

    let consumer = JobConsumer::new(result_sender);
    if let Err(e) = job_queue.subscribe(consumer).await {
        // TODO: panic ?
        error!("Failed to start consumer: {}", e);
    }

    tokio::signal::ctrl_c().await.unwrap();
    info!("Shutting down...");

    job_queue.close().await.unwrap();
    info!("Worker shut down gracefully");
}
