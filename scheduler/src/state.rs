use std::sync::Arc;

use crate::repository::{data_repository::DataRepository, worker_repository::WorkerRepository};
use rabbitmq::QueueHandler;
use tokio::sync::Mutex;

pub struct AppState {
    pub job_queue: Arc<Mutex<QueueHandler>>,
    pub data_repo: Arc<DataRepository>,
    pub worker_repo: Arc<WorkerRepository>,
}

impl AppState {
    pub fn new(
        job_queue: Arc<Mutex<QueueHandler>>,
        data_repo: Arc<DataRepository>,
        worker_repo: Arc<WorkerRepository>,
    ) -> Self {
        AppState {
            job_queue,
            data_repo,
            worker_repo,
        }
    }
}
