use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub name: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWT {
    pub header: String,
    pub payload: String,
}
