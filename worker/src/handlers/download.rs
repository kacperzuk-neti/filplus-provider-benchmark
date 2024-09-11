use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rabbitmq::{AccumulatingBytes, DownloadError, DownloadResult, IntervalBytes, JobMessage};
use reqwest::{
    header::{ACCEPT, RANGE, USER_AGENT},
    Client,
};
use tracing::{debug, info};

/// Prepare the HTTP request
fn prepare_request(url: &str, range_start: usize, range_end: usize) -> reqwest::RequestBuilder {
    const USER_AGENT_STR: &str = "curl/7.68.0";
    const ACCEPT_TYPE: &str = "*/*";

    Client::new()
        .get(url)
        .header(RANGE, format!("bytes={}-{}", range_start, range_end))
        .header(USER_AGENT, USER_AGENT_STR)
        .header(ACCEPT, ACCEPT_TYPE)
}

/// Calculates the next even second from the given `SystemTime`.
fn calculate_next_even_second(current: SystemTime) -> SystemTime {
    let duration_since_epoch = current
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));

    let millis = duration_since_epoch.as_millis() % 1000;
    let remaining_millis = 1000 - millis;

    current + Duration::from_millis(remaining_millis as u64)
}

/// Benchmark the download speed of the given URL
pub async fn process(payload: JobMessage) -> Result<DownloadResult, DownloadError> {
    let range_start = 0;
    let range_end = 100 * 1024 * 1024; // 100 MB

    let request = prepare_request(&payload.url, range_start, range_end);

    let start_time = SystemTime::now();
    let mut bytes: usize = 0;
    let mut total_bytes: usize = 0;
    let mut second_by_second_logs: Vec<(SystemTime, IntervalBytes, AccumulatingBytes)> = Vec::new();

    let mut response = request
        .send()
        .await
        .map_err(|e| DownloadError(format!("RequestError: {}", e)))?;

    if !response.status().is_success() {
        return Err(DownloadError(format!(
            "RequestFailed: {}",
            response.status()
        )));
    }

    let time_to_first_byte = start_time.elapsed().unwrap().as_secs_f64() * 1000.0;
    info!("Time to first byte: {} ms", time_to_first_byte);

    let mut next_log_time = calculate_next_even_second(start_time);

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| DownloadError(format!("ChunkError: {}", e)))?
    {
        let chunk_size = chunk.len();
        bytes += chunk_size;
        total_bytes += chunk_size;

        let current_time = SystemTime::now();
        // Save the data for each interval, close to each even second
        if current_time >= next_log_time {
            // Save the stats for the interval
            second_by_second_logs.push((
                current_time,
                IntervalBytes(bytes),
                AccumulatingBytes(total_bytes),
            ));
            debug!(
                "Time: {:?}, Bytes downloaded: {}",
                current_time, total_bytes
            );

            // Reset the interval byte counter
            bytes = 0;
            // Increment next log time to the next even second
            next_log_time = calculate_next_even_second(current_time);
            debug!(
                "Duration from current time {:?}",
                next_log_time
                    .duration_since(current_time)
                    .unwrap()
                    .as_millis()
            );
        }
    }

    let elapsed_secs = start_time.elapsed().unwrap().as_secs_f64();
    let end_time = SystemTime::now();
    // Convert to bits and then to kilo and mega bits per second
    let download_speed = (total_bytes as f64 * 8.0) / (elapsed_secs * 1024.0 * 1024.0);

    info!(
        "Downloaded {} bytes in {:.2} seconds ({:.2} Mbps, {:.2} MBps)",
        total_bytes,
        elapsed_secs,
        download_speed,
        download_speed / 8.0
    );

    Ok(DownloadResult {
        total_bytes,
        elapsed_secs,
        download_speed,
        start_time,
        end_time,
        second_by_second_logs,
    })
}
