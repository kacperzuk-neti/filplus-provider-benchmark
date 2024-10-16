use std::sync::Arc;

use rabbitmq::Publisher;
use tokio::sync::Mutex;

use crate::repository::*;

pub struct AppState {
    pub job_queue: Arc<Mutex<Publisher>>,
    pub data_repo: Arc<DataRepository>,
    pub worker_repo: Arc<WorkerRepository>,
    pub job_repo: Arc<JobRepository>,
    pub topic_repo: Arc<TopicRepository>,
    pub sub_job_repo: Arc<SubJobRepository>,
}

impl AppState {
    pub fn new(
        job_queue: Arc<Mutex<Publisher>>,
        data_repo: Arc<DataRepository>,
        worker_repo: Arc<WorkerRepository>,
        job_repo: Arc<JobRepository>,
        topic_repo: Arc<TopicRepository>,
        sub_job_repo: Arc<SubJobRepository>,
    ) -> Self {
        AppState {
            job_queue,
            data_repo,
            worker_repo,
            job_repo,
            topic_repo,
            sub_job_repo,
        }
    }
}
