use axum::Router;
use dotenv::dotenv;
use handlers::data_consumer::DataConsumer;
use rabbitmq::*;
use state::AppState;
use std::{error::Error, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod handlers;
mod routes;
mod state;

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

    info!("Scheduler started");

    let addr = Arc::new(std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()));

    debug!("RabbitMQ host: {}", addr);

    let mut job_queue = QueueHandler::clone(&CONFIG_QUEUE_JOB);
    match job_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up job queue"),
        Err(e) => panic!("Failed to set up job queue: {}", e),
    }

    // Initialize in memory data store
    let app_state = Arc::new(AppState::new(job_queue));

    let mut data_queue = QueueHandler::clone(&CONFIG_QUEUE_RESULT);
    match data_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up data queue"),
        Err(e) => panic!("Failed to set up data queue: {}", e),
    }
    let consumer = DataConsumer::new(app_state.clone());
    match data_queue.subscribe(consumer).await {
        Ok(_) => info!("Successfully started data queue consumer"),
        Err(e) => panic!("Failed to start data queue consumer: {}", e),
    }

    let app = Router::new()
        .merge(routes::create_routes())
        .layer(
            ServiceBuilder::new().layer(TraceLayer::new_for_http()),
            // TODO: add something to authenticate requests
        )
        .with_state(app_state.clone());

    let server_addr = "0.0.0.0:3000".to_string();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    info!("Listening on http://{}", &server_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    // Close the connection gracefully
    // job_queue.close().await.unwrap(); // TODO: fix this !
    // TODO: make sure that connection is closed properly !
    info!("Scheduler shut down gracefully");

    Ok(())
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(_) => info!("Received SIGINT signal, Shutting down..."),
        Err(e) => panic!("Failed to listen for SIGINT signal: {}", e),
    }
}
