use rabbitmq::JobMessage;
use std::error::Error;
use std::net::{IpAddr, ToSocketAddrs};
use std::process::Command;
use std::str;
use tracing::{debug, error, info};
use url::Url;

struct PingResult {
    avg_latency: f64,
    min_latency: f64,
    max_latency: f64,
}

pub async fn process(payload: JobMessage) -> Result<String, Box<dyn Error + Send + Sync>> {
    info!("Processing PING job");

    // TODO: make proper URL parsing and error handling!
    // Parse the URL and extract the host
    let url = Url::parse(&payload.url)?;
    let host = url.host_str().ok_or("Failed to extract host from URL")?;

    // Resolve the host to an IP address
    let ip_addresses: Vec<IpAddr> = (host, 0)
        .to_socket_addrs()?
        .map(|socket_addr| socket_addr.ip())
        .collect();

    if ip_addresses.is_empty() {
        error!("Could not resolve host to IP addresses.");

        return Err("Could not resolve host to IP addresses.".into());
    }

    let ip_address = ip_addresses[0];
    debug!("Resolved IP address: {}", ip_address);

    let output = Command::new("ping")
        .arg("-c")
        .arg("10")
        .arg(ip_address.to_string())
        .output()?;

    if !output.status.success() {
        return Err("Ping command failed.".into());
    }

    let stdout = str::from_utf8(&output.stdout)?;
    debug!("Ping output:\n{}", stdout);

    // Parse the latency statistics from the output
    let mut latencies: Vec<f64> = Vec::new();
    for line in stdout.lines() {
        if line.contains("time=") {
            if let Some(time_str) = line.split("time=").nth(1) {
                if let Some(latency_str) = time_str.split_whitespace().next() {
                    if let Ok(latency) = latency_str.parse::<f64>() {
                        latencies.push(latency);
                    }
                }
            }
        }
    }

    if latencies.is_empty() {
        error!("Failed to parse latency values from ping output.");

        return Err("Failed to parse latency values from ping output.".into());
    }

    // Calculate the average, min, and max latencies
    let avg_latency: f64 = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let min_latency: f64 = *latencies
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_latency: f64 = *latencies
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    debug!("Latency Statistics:");
    debug!("Average: {:.2} ms", avg_latency);
    debug!("Min: {:.2} ms", min_latency);
    debug!("Max: {:.2} ms", max_latency);

    Ok("ping result".to_string())
}
