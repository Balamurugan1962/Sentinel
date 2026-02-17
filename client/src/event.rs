use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Event {
    pub module: String,
    pub event_type: String,
    pub metadata: String,
    pub timestamp: String,
}
