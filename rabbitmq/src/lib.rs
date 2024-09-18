use std::{collections::HashSet, env};

use amqprs::{
    channel::{
        BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments,
        QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod messages;

// re export messages
pub use messages::{
    AccumulatingBytes, DownloadError, DownloadResult, HeadError, HeadResult, IntervalBytes,
    JobMessage, PingError, PingResult, ResultMessage,
};

// Messages that can be sent or received
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    WorkerJob { job_id: Uuid, payload: JobMessage },
    WorkerResult { job_id: Uuid, result: ResultMessage },
}

// Configuration for RabbitMQ exchange, queue, and routing key
#[derive(Clone)]
pub struct QueueHandler {
    pub exchange_name: &'static str,
    pub queue_name: Option<&'static str>,
    pub routing_key: Option<&'static str>,
    exchange_type: &'static str,
    connection: Option<Connection>,
    channel: Option<Channel>,
}

impl QueueHandler {
    fn set_queue_name(&mut self, queue_name: &'static str) {
        self.queue_name = Some(queue_name);
    }

    fn set_routing_key(&mut self, routing_key: &'static str) {
        self.routing_key = Some(routing_key);
    }

    pub async fn setup(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Open connection
        let connection =
            Connection::open(&OpenConnectionArguments::new(addr, 5672, "guest", "guest")).await?;

        // Open channel
        let channel = connection.open_channel(None).await?;

        // Declare exchange
        channel
            .exchange_declare(
                ExchangeDeclareArguments::new(self.exchange_name, self.exchange_type)
                    .passive(false)
                    .durable(true)
                    .finish(),
            )
            .await?;

        if self.queue_name.is_none() || self.routing_key.is_none() {
            let worker_name: &'static str = Box::leak(
                std::env::var("WORKER_NAME")
                    .unwrap_or_else(|_| "default_worker".to_string())
                    .into_boxed_str(),
            );
            self.set_queue_name(worker_name);
            self.set_routing_key(worker_name);
        }

        let worker_topics: Vec<String> = env::var("WORKER_TOPICS")
            .unwrap_or_else(|_| "all".to_string())
            .split(',')
            .map(|s| s.to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Declare queue
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(
                self.queue_name.unwrap(),
            ))
            .await?;

        // Bind queue to exchange
        channel
            .queue_bind(QueueBindArguments::new(
                self.queue_name.unwrap(),
                self.exchange_name,
                self.routing_key.unwrap(),
            ))
            .await?;

        if self.exchange_type == "topic" {
            // Bind the queue to the exchange with each topic
            for topic in worker_topics {
                channel
                    .queue_bind(QueueBindArguments::new(
                        self.queue_name.unwrap(),
                        self.exchange_name,
                        &topic,
                    ))
                    .await?;
            }
        }

        self.connection = Some(connection);
        self.channel = Some(channel);

        Ok(())
    }

    pub async fn publish(
        &self,
        message: &Message,
        routing_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_message = serde_json::to_vec(message)?;
        let args = BasicPublishArguments::new(self.exchange_name, routing_key);

        self.channel
            .as_ref()
            .unwrap()
            .basic_publish(BasicProperties::default(), serialized_message, args)
            .await?;

        Ok(())
    }

    pub async fn subscribe<C>(&self, consumer: C) -> Result<(), Box<dyn std::error::Error>>
    where
        C: AsyncConsumer + Send + Sync + 'static,
    {
        let args = BasicConsumeArguments::new(
            self.queue_name.unwrap(),
            "consumer_tag_somehow_take_from_consumer",
        );

        self.channel
            .as_ref()
            .unwrap()
            .basic_consume(consumer, args)
            .await
            .unwrap();

        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(connection) = self.connection.take() {
            connection.close().await?;
        }
        if let Some(channel) = self.channel.take() {
            channel.close().await?;
        }

        Ok(())
    }
}

// RabbitMQ configurations for various services and use cases
pub const CONFIG_QUEUE_JOB: QueueHandler = QueueHandler {
    exchange_name: "job_exchange",
    queue_name: None,
    routing_key: None,
    exchange_type: "topic",
    connection: None,
    channel: None,
};

pub const CONFIG_QUEUE_RESULT: QueueHandler = QueueHandler {
    exchange_name: "result_exchange",
    queue_name: Some("result_queue"),
    routing_key: Some("worker_a_result"),
    exchange_type: "direct",
    connection: None,
    channel: None,
};
