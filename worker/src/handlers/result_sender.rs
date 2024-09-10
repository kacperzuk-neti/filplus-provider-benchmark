use rabbitmq::{Message, QueueHandler, CONFIG_WORKER_A_RESULT};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

pub async fn handle(mut result_receiver: mpsc::Receiver<Message>, addr: Arc<String>) {
    while let Some(result) = result_receiver.recv().await {
        let mut data_queue = QueueHandler::clone(&CONFIG_WORKER_A_RESULT);
        match data_queue.setup(&addr).await {
            Ok(_) => info!("Successfully set up data queue"),
            Err(e) => error!("Failed to set up data queue: {}", e),
        }
        if let Err(e) = data_queue.publish(&result).await {
            error!("Error publishing result: {:?}", e);
        }
    }
}
