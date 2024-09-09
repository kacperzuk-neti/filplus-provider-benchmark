
use rabbitmq::JobMessage;
use reqwest::header::{ACCEPT, RANGE, USER_AGENT};
use reqwest::Client;
use std::error::Error;
use tokio::time::Instant;
use tracing::{debug, error, info};

pub async fn process(payload: JobMessage) -> Result<String, Box<dyn Error + Send + Sync>> {
    info!("Processing DOWNLOAD job");

    let client = Client::new();
    let url = payload.url.clone();
    let download_mb = 100; // 100 MB
    let range_start = 0; // in bytes //TODO: randomize this 
    let range_end = range_start + download_mb * 1024 * 1024;

    // Make a range request
    let request = client
        .get(url)
        .header(RANGE, format!("bytes={}-{}", range_start, range_end))
        .header(USER_AGENT, "curl/7.68.0") // Mimic `curl` user agent
        .header(ACCEPT, "*/*"); // Set `Accept` header to any content type

    let start_time = Instant::now();

    let mut elapsed_secs = 0.0;
    let mut total_bytes: usize = 0;

    match request.send().await {
        Ok(mut response) => {
            if !response.status().is_success() {
                error!("Request failed with status: {}", response.status());

                // TODO: this is critical part of the worker checks, we should make sure that this bubbles up to consumer and doesnt allow the job to be marked as success
                // TODO: or make a retry mechanism ?
                // TODO: or compensate somehow for a failed job maybe with more nodes ?
                return Ok("download speed result".to_string());
            }

            while let Some(chunk) = response.chunk().await? {
                total_bytes += chunk.len();
                debug!("Received chunk of size: {} bytes", chunk.len());

                // Optional: Stop if a certain size is reached (e.g., 5 MB)
                if total_bytes >= range_end {
                    debug!("Downloaded {} MB, stopping.", total_bytes / 1024 / 1024);
                    break;
                }
            }

            let duration = Instant::now() - start_time;
            elapsed_secs = duration.as_secs_f64();

            // debug!(
            //     "Downloaded {} bytes in {:.2} seconds ({:.2} MB/s)",
            //     total_bytes,
            //     elapsed_secs,
            //     total_bytes as f64 / elapsed_secs / 1024.0 / 1024.0
            // );
        }
        Err(err) => {
            error!("Request error: {:?}", err);
        }
    }

    debug!(
        "Downloaded {} bytes in {:.2} seconds ({:.2} MB/s)",
        total_bytes,
        elapsed_secs,
        total_bytes as f64 / elapsed_secs / 1024.0 / 1024.0
    );

    info!("Finished processing DOWNLOAD job");

    Ok("download speed result {}".to_string())
}
