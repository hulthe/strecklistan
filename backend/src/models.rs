use diesel::prelude::*;
use rocket::request::FromForm;
use chrono::NaiveDateTime;
use super::schema::events;

#[derive(FromForm)]
pub struct EventRange {
    pub low: i32,
    pub high: i32,
}

#[derive(Queryable, Serialize)]
pub struct Event {
    pub id: i32,
    pub title: String,
    pub background: String,
    pub location: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub price: i32,
    pub published: bool,
}

#[derive(Insertable, Deserialize)]
#[table_name="events"]
pub struct NewEvent {
    pub title: String,
    pub background: String,
    pub location: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub price: Option<i32>,
}
