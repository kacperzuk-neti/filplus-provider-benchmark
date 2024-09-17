use std::sync::Arc;

use crate::state::AppState;
use amqprs::{
    channel::{BasicAckArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rabbitmq::{Message, ResultMessage};
use serde_json;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct DataConsumer {
    state: Arc<AppState>,
}

impl DataConsumer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    async fn parse_message(&self, content_str: &str) -> Result<(Uuid, ResultMessage)> {
        match serde_json::from_str::<Message>(content_str) {
            Ok(Message::WorkerResult { job_id, result }) => Ok((job_id, result)),
            Ok(Message::WorkerJob { .. }) => {
                Err(anyhow!("Received unexpected WorkerResult message"))
            }
            Err(e) => {
                error!("Error parsing message: {:?}", e);
                Err(e.into())
            }
        }
    }

    async fn process_message(&self, job_id: Uuid, result_message: ResultMessage) -> Result<()> {
        info!("Handling message: {:?} {:?}", job_id, result_message);

        self.state
            .data_repo
            .save_data(job_id, result_message)
            .await?;

        Ok(())
    }

    async fn run(&self, content: Vec<u8>) -> Result<()> {
        let content_str = String::from_utf8(content)?;

        debug!("Received message: {}", content_str);

        let (job_id, result_message) = self.parse_message(&content_str).await?;

        self.process_message(job_id, result_message).await?;

        Ok(())
    }
}

#[async_trait]
impl AsyncConsumer for DataConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        match self.run(content).await {
            Ok(_) => {
                info!("Processed message successfully");
                // Ack message only if processed successfully
                let args = BasicAckArguments::new(deliver.delivery_tag(), false);
                channel.basic_ack(args).await.unwrap();
                debug!("Acked message");
            }
            Err(e) => {
                error!("Error processing message: {:?}", e);
            }
        }
    }
}
