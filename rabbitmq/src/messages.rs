use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

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
