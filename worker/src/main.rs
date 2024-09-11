use dotenv::dotenv;
use handlers::job_consumer::JobConsumer;
use rabbitmq::*;
use std::{error::Error, sync::Arc};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod handlers;

// TODO: make sure that the worker can and will exit gracefully
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let addr = Arc::new(std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()));

    debug!("RabbitMQ host: {}", addr);

    let mut job_queue = QueueHandler::clone(&CONFIG_WORKER_A_JOB);
    match job_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up job queue"),
        Err(e) => panic!("Failed to set up job queue: {}", e),
    }

    let mut data_queue = QueueHandler::clone(&CONFIG_WORKER_A_RESULT);
    match data_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up data queue"),
        Err(e) => panic!("Failed to set up data queue: {}", e),
    }

    let consumer = JobConsumer::new(data_queue.clone());
    match job_queue.subscribe(consumer).await {
        Ok(_) => info!("Successfully started job queue consumer"),
        Err(e) => panic!("Failed to start job queue consumer: {}", e),
    }

    match tokio::signal::ctrl_c().await {
        Ok(_) => info!("Received SIGINT signal, Shutting down..."),
        Err(e) => panic!("Failed to listen for SIGINT signal: {}", e),
    }

    job_queue.close().await.unwrap();
    data_queue.close().await.unwrap();
    info!("Worker shut down gracefully");

    Ok(())
}
