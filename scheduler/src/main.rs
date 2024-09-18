use std::{env, error::Error, sync::Arc};

use axum::Router;
use color_eyre::Result;
use queue::data_consumer::DataConsumer;
use rabbitmq::*;
use repository::data_repository::DataRepository;
use sqlx::{migrate::Migrator, PgPool};
use state::AppState;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod api;
mod queue;
mod repository;
mod routes;
mod state;

static MIGRATOR: Migrator = sqlx::migrate!("./src/migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    info!("Scheduler is starting...");

    // Initialize color_eyre panic and error handlers
    color_eyre::install()?;

    // Load .env
    dotenvy::dotenv()?;

    // Initialize logging
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    // Initialize database connection pool & run migrations
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    debug!("PostgreSQL url: {}", db_url);
    let pool = PgPool::connect(&db_url).await?;
    MIGRATOR.run(&pool).await?;

    let addr = Arc::new(env::var("RABBITMQ_HOST").expect("RABBITMQ_HOST must be set"));

    debug!("RabbitMQ host: {}", addr);

    let mut job_queue = QueueHandler::clone(&CONFIG_QUEUE_JOB);
    job_queue.setup(&addr).await?;
    info!("Successfully set up job queue");

    // Initialize data repository
    let data_repo = Arc::new(DataRepository::new(pool.clone()));

    // Initialize app state
    let app_state = Arc::new(AppState::new(job_queue, data_repo));

    let mut data_queue = QueueHandler::clone(&CONFIG_QUEUE_RESULT);
    data_queue.setup(&addr).await?;
    info!("Successfully set up data queue");

    let consumer = DataConsumer::new(app_state.clone());
    data_queue.subscribe(consumer).await?;
    info!("Successfully started data queue consumer");

    let app = Router::new()
        .merge(routes::create_routes())
        .layer(
            ServiceBuilder::new().layer(TraceLayer::new_for_http()),
            // TODO: add something to authenticate requests
        )
        .with_state(app_state.clone());

    let server_addr = "0.0.0.0:3000".to_string();
    let listener = TcpListener::bind(&server_addr).await?;
    info!("Listening on http://{}", &server_addr);

    info!("Scheduler started successfully, waiting for requests...");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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
