use crate::state::AppState;
use amqprs::{
    channel::{BasicAckArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use async_trait::async_trait;
use rabbitmq::{Message, ResultMessage};
use serde_json;
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct DataConsumer {
    state: Arc<AppState>,
}

impl DataConsumer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    async fn parse_message(&self, content_str: &str) -> Option<(Uuid, ResultMessage)> {
        match serde_json::from_str::<Message>(content_str) {
            Ok(Message::WorkerResult { job_id, result }) => Some((job_id, result)),
            Ok(Message::WorkerJob { .. }) => {
                error!("Received unexpected WorkerResult message");
                None
            }

            Err(e) => {
                error!("Error parsing message: {:?}", e);
                None
            }
        }
    }

    async fn process_message(
        &self,
        job_id: Uuid,
        result_message: ResultMessage,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Handling message: {:?} {:?}", job_id, result_message);

        {
            let mut results = self.state.results.lock().unwrap();
            results.insert(job_id, result_message);
        }

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
        let mut no_ack = false;

        let content_str = String::from_utf8(content).unwrap();
        debug!("Received message: {}", content_str);

        if let Some((job_id, result_message)) = self.parse_message(&content_str).await {
            match self.process_message(job_id, result_message).await {
                Ok(_) => {
                    info!("Processed message successfully");
                }
                Err(e) => {
                    no_ack = true;
                    error!("Error processing message: {:?}", e);
                }
            }
        } else {
            error!("Error parsing message");
        }

        if !no_ack {
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            channel.basic_ack(args).await.unwrap();
            debug!("Acked message");
        }
    }
}
