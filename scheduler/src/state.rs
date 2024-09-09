use rabbitmq::ResultMessage;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug)]
pub struct AppState {
    pub results: Mutex<HashMap<Uuid, ResultMessage>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            results: Mutex::new(HashMap::new()),
        }
    }
}
