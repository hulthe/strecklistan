use crate::schema::tables::events;
use crate::util::StatusJson;
use chrono::NaiveDateTime;
use rocket::http::Status;

#[derive(FromForm)]
pub struct EventRange {
    pub low: i64,
    pub high: i64,
}

impl EventRange {
    pub fn validate(&self) -> Result<(), StatusJson> {
        match self.low >= self.high {
            false => Ok(()),
            true => Err(StatusJson {
                status: Status::BadRequest,
                description: "EventRange: high must be greater than low".into(),
            }),
        }
    }
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct EventWithSignups {
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

#[derive(Queryable, Serialize, Deserialize, Debug)]
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

#[derive(Insertable, GraphQLInputObject, Serialize, Deserialize, Debug)]
#[table_name = "events"]
pub struct NewEvent {
    pub title: String,
    pub background: String,
    pub location: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub price: Option<i32>,
}

impl From<Event> for EventWithSignups {
    fn from(event: Event) -> EventWithSignups {
        EventWithSignups {
            id: event.id,
            title: event.title,
            background: event.background,
            location: event.location,
            start_time: event.start_time,
            end_time: event.end_time,
            price: event.price,
            published: event.published,
            signups: 0,
        }
    }
}
