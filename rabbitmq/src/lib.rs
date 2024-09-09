use amqprs::{
    channel::{
         BasicConsumeArguments, BasicPublishArguments, Channel,
        ExchangeDeclareArguments, ExchangeType, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobMessage {
    pub url: String,
    pub start_time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultMessage { // TODO: ResultMessage needs complete overhaul !
    pub bandwidth: u32,
    pub latency: u32,
    pub status_code: u32,
    pub content_length: u32,
    pub duration: u32,
    pub error: String,
}

impl ResultMessage {
    pub fn new(
        bandwidth: u32,
        latency: u32,
        status_code: u32,
        content_length: u32,
        duration: u32,
        error: String,
    ) -> Self {
        Self {
            bandwidth,
            latency,
            status_code,
            content_length,
            duration,
            error,
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
    pub queue_name: &'static str,
    pub routing_key: &'static str,
    connection: Option<Connection>,
    channel: Option<Channel>,
}


impl QueueHandler {
    pub async fn setup(
        &mut self,
        addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Open connection
        let connection =
            Connection::open(&OpenConnectionArguments::new(&addr, 5672, "guest", "guest")).await?;

        // Open channel
        let channel = connection.open_channel(None).await?;

        // Declare exchange
        channel
            .exchange_declare(
                ExchangeDeclareArguments::new(
                    self.exchange_name,
                    &ExchangeType::Direct.to_string(),
                )
                .passive(false)
                .durable(true)
                .finish(),
            )
            .await?;

        // Declare queue
        channel
            .queue_declare(QueueDeclareArguments::durable_client_named(self.queue_name))
            .await?;

        // Bind queue to exchange
        channel
            .queue_bind(QueueBindArguments::new(
                self.queue_name,
                self.exchange_name,
                self.routing_key,
            ))
            .await?;

        self.connection = Some(connection);
        self.channel = Some(channel);

        Ok(())
    }

    pub async fn publish(
        &self,
        message: &Message,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_message = serde_json::to_vec(message)?;
        let args = BasicPublishArguments::new(self.exchange_name, self.routing_key);

        self.channel
            .as_ref()
            .unwrap()
            .basic_publish(BasicProperties::default(), serialized_message, args)
            .await?;

        Ok(())
    }

    pub async fn subscribe<C>(
        &self,
        consumer: C,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        C: AsyncConsumer + Send + Sync + 'static,
    {
        let args =
            BasicConsumeArguments::new(self.queue_name, "consumer_tag_somehow_take_from_consumer");

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
    exchange_name: "test_exchange",
    queue_name: "worker_job",
    routing_key: "worker_a_job",
    connection: None,
    channel: None,
};

pub const CONFIG_WORKER_A_RESULT: QueueHandler = QueueHandler {
    exchange_name: "test_exchange",
    queue_name: "worker_result",
    routing_key: "worker_a_result",
    connection: None,
    channel: None,
};

