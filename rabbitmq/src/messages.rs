use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobMessage {
    pub url: String,
    pub start_timestamp: Duration,
    pub start_range: u64,
    pub end_range: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultMessage {
    pub download_result: Result<DownloadResult, DownloadError>,
    pub ping_result: Result<PingResult, PingError>,
    pub head_result: Result<HeadResult, HeadError>,
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
pub struct DownloadError {
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResult {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingError {
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeadResult {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeadError {
    pub error: String,
}

impl ResultMessage {
    pub fn new(
        download_result: Result<DownloadResult, DownloadError>,
        ping_result: Result<PingResult, PingError>,
        head_result: Result<HeadResult, HeadError>,
    ) -> Self {
        Self {
            download_result,
            ping_result,
            head_result,
        }
    }
}
