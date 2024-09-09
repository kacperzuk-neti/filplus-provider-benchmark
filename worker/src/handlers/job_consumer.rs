use crate::handlers::*;
use amqprs::{
    channel::{BasicAckArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use async_trait::async_trait;
use rabbitmq::{JobMessage, Message, ResultMessage};
use serde_json;
use std::error::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct JobConsumer {
    result_sender: mpsc::Sender<Message>,
}

impl JobConsumer {
    pub fn new(result_sender: mpsc::Sender<Message>) -> Self {
        Self { result_sender }
    }
    
    async fn parse_message(&self, content_str: &str) -> Option<(Uuid, JobMessage)> {
        match serde_json::from_str::<Message>(content_str) {
            Ok(Message::WorkerJob { job_id, payload }) => Some((job_id, payload)),
            Ok(Message::WorkerResult { .. }) => {
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
        job_message: JobMessage,
    ) -> Result<ResultMessage, Box<dyn Error + Send + Sync>> {
        info!("Handling message: {:?} {:?}", job_id, job_message);

        let (download_result, ping_result, latency_result) = tokio::try_join!(
            download::process(job_message.clone()),
            ping::process(job_message.clone()),
            head::process(job_message.clone()),
        )?;

        info!(
            "Results: {:?} {:?} {:?}",
            ping_result, latency_result, download_result
        );

        results::aggregate(ping_result, latency_result, download_result)
    }
}

#[async_trait]
impl AsyncConsumer for JobConsumer {
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

        if let Some((job_id, job_message)) = self.parse_message(&content_str).await {
            // React to the received data
            let result = self.process_message(job_id, job_message).await.unwrap();

            let result_message = Message::WorkerResult { job_id, result };
            if let Err(e) = self.result_sender.send(result_message).await {
                no_ack = true;
                error!("Error sending result: {:?}", e);
            }
        } else {
            // no_ack = true; // TODO: we probably want to take this out, maybe move to different queue
            error!("Error parsing message");
        }

        // if !self.no_ack {
        if !no_ack {
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            channel.basic_ack(args).await.unwrap();
            debug!("Acked message");
        }
    }
}
