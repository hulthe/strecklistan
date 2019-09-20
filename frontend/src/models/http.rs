use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusJson {
    pub status: i32,
    pub description: String,
}
