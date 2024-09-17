use std::sync::Arc;

use crate::repository::data_repository::DataRepository;
use rabbitmq::QueueHandler;

pub struct AppState {
    pub job_queue: QueueHandler,
    pub data_repo: Arc<DataRepository>,
}

impl AppState {
    pub fn new(job_queue: QueueHandler, data_repo: Arc<DataRepository>) -> Self {
        AppState {
            job_queue,
            data_repo,
        }
    }
}
