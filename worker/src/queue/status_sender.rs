use std::env;

use chrono::Utc;
use rabbitmq::{
    Message, QueueHandler, StatusMessage, WorkerStatus, WorkerStatusDetails,
    WorkerStatusJobDetails, CONFIG_QUEUE_STATUS,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct StatusSender {
    status_queue: QueueHandler,
}

impl StatusSender {
    pub fn new(status_queue: QueueHandler) -> Self {
        StatusSender { status_queue }
    }

    pub async fn send_lifecycle_status(
        &self,
        status: WorkerStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::WorkerStatus {
            status: StatusMessage {
                status: WorkerStatusDetails::Lifecycle(status),
                timestamp: Utc::now(),
                worker_name: env::var("WORKER_NAME").unwrap(),
            },
        };

        self.send_status(message).await?;

        Ok(())
    }

    pub async fn send_job_status(
        &self,
        job_id: Option<Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let job_details = job_id.map(|job_id| WorkerStatusJobDetails { job_id });

        let message = Message::WorkerStatus {
            status: StatusMessage {
                status: WorkerStatusDetails::Job(job_details),
                timestamp: Utc::now(),
                worker_name: env::var("WORKER_NAME").unwrap(),
            },
        };

        self.send_status(message).await?;

        Ok(())
    }

    pub async fn send_heartbeat_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::WorkerStatus {
            status: StatusMessage {
                status: WorkerStatusDetails::Heartbeat,
                timestamp: Utc::now(),
                worker_name: env::var("WORKER_NAME").unwrap(),
            },
        };

        self.send_status(message).await?;

        Ok(())
    }

    async fn send_status(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        self.status_queue
            .publish(&message, CONFIG_QUEUE_STATUS.routing_key.unwrap())
            .await?;

        Ok(())
    }
}
