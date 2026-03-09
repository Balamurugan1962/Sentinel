use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct UserInfo {
    pub name: String,
    pub reg: String,
}

impl UserInfo {
    pub fn new() -> UserInfo {
        UserInfo {
            name: "unknown".to_string(),
            reg: "unknown".to_string(),
        }
    }
}

pub type SharedUser = Arc<Mutex<UserInfo>>;
