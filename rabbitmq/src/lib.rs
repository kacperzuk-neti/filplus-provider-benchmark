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
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobMessage {
    pub url: String,
    pub start_time: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultMessage {
    pub download_result: Option<DownloadResult>,
    pub download_error: Option<DownloadError>,
    pub ping_result: Option<PingResult>,
    pub ping_error: Option<PingError>,
    pub head_result: Option<HeadResult>,
    pub head_error: Option<HeadError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadResult {
    pub total_bytes: usize,
    pub elapsed_secs: f64,
    pub download_speed: f64,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub second_by_second_logs: Vec<(SystemTime, IntervalBytes, AccumulatingBytes)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntervalBytes(pub usize);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccumulatingBytes(pub usize);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadError(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResult {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub mean_dev: f64, // Mean deviation
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingError(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeadResult {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeadError(pub String);

impl ResultMessage {
    pub fn new(
        download_result: Result<DownloadResult, DownloadError>,
        ping_result: Result<PingResult, PingError>,
        head_result: Result<HeadResult, HeadError>,
    ) -> Self {
        let mut result_message = Self {
            download_result: None,
            download_error: None,
            ping_result: None,
            ping_error: None,
            head_result: None,
            head_error: None,
        };
        result_message.set_download_result(download_result);
        result_message.set_ping_result(ping_result);
        result_message.set_head_result(head_result);
        result_message
    }
    pub fn set_download_result(&mut self, download_result: Result<DownloadResult, DownloadError>) {
        match download_result {
            Ok(download_result) => self.download_result = Some(download_result),
            Err(download_error) => self.download_error = Some(download_error),
        }
    }
    pub fn set_ping_result(&mut self, ping_result: Result<PingResult, PingError>) {
        match ping_result {
            Ok(ping_result) => self.ping_result = Some(ping_result),
            Err(ping_error) => self.ping_error = Some(ping_error),
        }
    }
    pub fn set_head_result(&mut self, head_result: Result<HeadResult, HeadError>) {
        match head_result {
            Ok(head_result) => self.head_result = Some(head_result),
            Err(head_error) => self.head_error = Some(head_error),
        }
    }
}

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
    topics: &'static [&'static str],
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

        if None == self.queue_name || None == self.routing_key {
            let worker_name: &'static str = Box::leak(
                std::env::var("WORKER_NAME")
                    .unwrap_or_else(|_| "default_worker".to_string())
                    .into_boxed_str(),
            );
            self.set_queue_name(worker_name);
            self.set_routing_key(worker_name);
        }

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

        if (self.exchange_type == "topic") {
            // Bind the queue to the exchange with each topic
            for topic in self.topics {
                channel
                    .queue_bind(QueueBindArguments::new(
                        self.queue_name.unwrap(),
                        self.exchange_name,
                        topic,
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
            .unwrap() // TODO: consider returning error instead of unwrap
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
pub const CONFIG_WORKER_A_JOB: QueueHandler = QueueHandler {
    exchange_name: "job_exchange",
    queue_name: None,
    routing_key: None,
    topics: &["all"],
    exchange_type: &"topic",
    connection: None,
    channel: None,
};

pub const CONFIG_WORKER_A_RESULT: QueueHandler = QueueHandler {
    exchange_name: "result_exchange",
    queue_name: Some("result_queue"),
    routing_key: Some("worker_a_result"),
    topics: &[],
    exchange_type: &"direct",
    connection: None,
    channel: None,
};
