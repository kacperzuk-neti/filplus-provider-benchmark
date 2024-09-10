use axum::Router;
use dotenv::dotenv;
use handlers::data_consumer::DataConsumer;
use rabbitmq::*;
use state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod handlers;
mod routes;
mod state;

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

    info!("Scheduler started");

    let addr = Arc::new(std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()));

    let mut job_queue = QueueHandler::clone(&CONFIG_WORKER_A_JOB);
    match job_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up job queue"),
        Err(e) => error!("Failed to set up job queue: {}", e), // TODO: panic ?!
    }

    // Initialize in memory data store
    let app_state = Arc::new(AppState::new(job_queue));

    let mut data_queue = QueueHandler::clone(&CONFIG_WORKER_A_RESULT);
    match data_queue.setup(&addr).await {
        Ok(_) => info!("Successfully set up data queue"),
        Err(e) => error!("Failed to set up data queue: {}", e), // TODO: panic ?!
    }
    let consumer = DataConsumer::new(app_state.clone());
    if let Err(e) = data_queue.subscribe(consumer).await {
        // TODO: panic ?
        error!("Failed to start consumer: {}", e);
    }

    let app = Router::new()
        .merge(routes::create_routes())
        .layer(
            ServiceBuilder::new().layer(TraceLayer::new_for_http()),
            // TODO: add something to authenticate requests
        )
        .with_state(app_state);

    let server_addr = "0.0.0.0:3000".to_string();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    println!("Listening on http://{}", &server_addr); // TODO: fix

    axum::serve(listener, app).await.unwrap();

    // TODO: this doesnt seem to work anymore
    tokio::signal::ctrl_c().await.unwrap();
    info!("Shutting down...");

    // Close the connection gracefully
    // job_queue.close().await.unwrap(); // TODO: fix this !
    // TODO: make sure that connection is closed properly !
    info!("Scheduler shut down gracefully");
}
