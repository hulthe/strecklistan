use chrono::NaiveDateTime;
use rocket::http::Status;
use schema::tables::events;
use util::StatusJson;

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

/// Metadata about an Event
#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: i32,
    /// The title of the event
    pub title: String,
    /// An URL pointing to an image associated with the event
    pub background: String,
    /// The location where the event will take place
    pub location: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    /// The entry price for the event
    pub price: i32,
    /// Whether the event has been published or not
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
