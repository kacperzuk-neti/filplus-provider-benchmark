use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobMessage {
    pub url: String,
    pub start_time: DateTime<Utc>,
    pub start_range: u64,
    pub end_range: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultMessage {
    pub run_id: Uuid,
    pub worker_name: String,
    pub download_result: Result<DownloadResult, DownloadError>,
    pub ping_result: Result<PingResult, PingError>,
    pub head_result: Result<HeadResult, HeadError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadResult {
    pub total_bytes: usize,
    pub elapsed_secs: f64,
    pub download_speed: f64,
    pub job_start_time: DateTime<Utc>,
    pub download_start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub time_to_first_byte_ms: f64,
    pub second_by_second_logs: Vec<(DateTime<Utc>, IntervalBytes, AccumulatingBytes)>,
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
        run_id: Uuid,
        worker_name: String,
        download_result: Result<DownloadResult, DownloadError>,
        ping_result: Result<PingResult, PingError>,
        head_result: Result<HeadResult, HeadError>,
    ) -> Self {
        Self {
            download_result,
            ping_result,
            head_result,
            run_id,
            worker_name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerStatusJobDetails {
    pub run_id: Uuid,
    pub job_id: Uuid,
    pub worker_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerStatusDetails {
    Lifecycle(WorkerDetails),
    Job(Option<WorkerStatusJobDetails>),
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerDetails {
    pub worker_topics: Vec<String>,
    pub worker_status: WorkerStatus,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerStatus {
    Online,
    Offline,
}
impl WorkerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerStatus::Online => "online",
            WorkerStatus::Offline => "offline",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusMessage {
    pub worker_name: String,
    pub status: WorkerStatusDetails,
    pub timestamp: DateTime<Utc>,
}
