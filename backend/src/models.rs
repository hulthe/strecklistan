use rocket::http::Status;
use chrono::NaiveDateTime;
use super::schema::events;
use super::util::ErrorJson;

#[derive(FromForm)]
pub struct EventRange {
    pub low: i64,
    pub high: i64,
}

impl EventRange {
    pub fn validate(&self) -> Result<(), ErrorJson> {
        match self.low >= self.high {
            false => Ok(()),
            true => Err(ErrorJson {
                status: Status::BadRequest,
                description: "EventRange: high must be greater than low".into(),
            }),
        }
    }
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
