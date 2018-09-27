use rocket_contrib::Json;
use chrono::Local;
use diesel::prelude::*;
use super::super::models::{Event, NewEvent, EventRange};
use super::super::database::establish_connection;
use super::super::schema::events;
use super::super::util::ErrorJson;

/// Route `GET /events?high=x&low=y`
///
/// Return all events in the range `low..high`, where `0..1` yields the
/// upcoming event and `-1..0` yields the most recently completed event.
#[get("/events?<range>")]
pub fn get_events(range: EventRange) -> Result<Json<Vec<Event>>, ErrorJson> {
    use super::super::schema::events::dsl::*;

    range.validate()?;

    let now = Local::now().naive_local();
    let connection = establish_connection();

    let mut previous: Vec<Event> = if range.low < 0 {
        events.filter(end_time.le(now))
            .order_by(start_time.desc())
            .limit(-range.low)
            .load(&connection)?
    } else { Vec::new() };

    let mut upcoming: Vec<Event> = if range.high > 0 {
        events.filter(end_time.gt(now))
            .order_by(start_time.asc())
            .limit(range.high)
            .load(&connection)?
    } else { Vec::new() };

    if range.high < 0 {
        if (-range.high) as usize>= previous.len() {
            previous = Vec::new();
        } else {
            previous.drain(..(-range.high as usize));
        }
    }

    if range.low > 0 {
        if range.low as usize >= upcoming.len() {
            upcoming = Vec::new();
        } else {
            upcoming.drain(..(range.low as usize));
        }
    }

    upcoming.reverse();

    upcoming.append(&mut previous);
    Ok(Json(upcoming))
}

/// Route `GET /event/<event_id>`
///
/// Get a specific event by its id parameter.
#[get("/event/<event_id>")]
pub fn get_event(event_id: i32) -> Result<Json<Event>, ErrorJson>{
    use super::super::schema::events::dsl::*;

    let connection = establish_connection();
    let result = events.find(event_id)
        .first::<Event>(&connection)?;
    Ok(Json(result))
}

/// Route `POST /event`
///
/// Post a new event.
#[post("/event", format = "application/json", data="<event>")]
pub fn post_event(event: Json<NewEvent>) -> Result<Json<Event>, ErrorJson>{
    let event = event.into_inner();
    let connection = establish_connection();

    let result = diesel::insert_into(events::table)
        .values(event)
        .get_result(&connection)?;
    Ok(Json(result))
}
