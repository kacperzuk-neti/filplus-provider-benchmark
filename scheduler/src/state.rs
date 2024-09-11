use rabbitmq::{QueueHandler, ResultMessage};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

// #[derive(Debug)]
pub struct AppState {
    pub results: Mutex<HashMap<Uuid, Vec<ResultMessage>>>,
    pub job_queue: QueueHandler,
}

impl AppState {
    pub fn new(job_queue: QueueHandler) -> Self {
        AppState {
            results: Mutex::new(HashMap::new()),
            job_queue,
        }
    }
}
