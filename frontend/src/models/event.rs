use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub title: String,
    pub background: String,
    pub location: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub price: i32,
    pub published: bool,
    pub signups: i64,
}
