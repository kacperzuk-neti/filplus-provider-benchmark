use rabbitmq::ResultMessage;
use std::error::Error;
use tracing::error;

pub fn aggregate(
    ping_result: String,
    latency_result: String,
    download_speed_result: String,
) -> Result<ResultMessage, Box<dyn Error + Send + Sync>> {
    // TODO: Implement the aggregation logic
    error!("Aggregation logic not implemented yet.");

    let bandwidth = 0; // TODO: make f64;
    let latency = 0;
    let status_code = 0;
    let content_length = 0;
    let duration = 0;
    let error = vec![ping_result, latency_result, download_speed_result]
        .join(", ")
        .to_string();
    let result = ResultMessage::new(
        bandwidth,
        latency,
        status_code,
        content_length,
        duration,
        error,
    );

    Ok(result)
}
