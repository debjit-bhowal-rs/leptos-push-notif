use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
}
