use std::sync::Arc;

use crate::repository::*;
use rabbitmq::QueueHandler;
use tokio::sync::Mutex;

pub struct AppState {
    pub job_queue: Arc<Mutex<QueueHandler>>,
    pub data_repo: Arc<DataRepository>,
    pub worker_repo: Arc<WorkerRepository>,
    pub job_repo: Arc<JobRepository>,
    pub topic_repo: Arc<TopicRepository>,
}

impl AppState {
    pub fn new(
        job_queue: Arc<Mutex<QueueHandler>>,
        data_repo: Arc<DataRepository>,
        worker_repo: Arc<WorkerRepository>,
        job_repo: Arc<JobRepository>,
        topic_repo: Arc<TopicRepository>,
    ) -> Self {
        AppState {
            job_queue,
            data_repo,
            worker_repo,
            job_repo,
            topic_repo,
        }
    }
}
