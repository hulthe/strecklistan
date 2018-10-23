use chrono::Local;
use database::DatabasePool;
use diesel::prelude::*;
use models::{Event, EventRange, EventWithSignups as EventWS, NewEvent};
use rocket::State;
use rocket_contrib::Json;
use schema::tables::events;
use util::StatusJson;

/// Route `GET /events?high=x&low=y`
///
/// Return all events in the range `low..high`, where `0..1` yields the
/// upcoming event and `-1..0` yields the most recently completed event.
#[get("/events?<range>")]
pub fn get_events(
    range: EventRange,
    db_pool: State<DatabasePool>,
) -> Result<Json<Vec<EventWS>>, StatusJson> {
    use schema::views::events_with_signups::dsl::*;

    range.validate()?;

    let now = Local::now().naive_local();
    let connection = db_pool.inner().get()?;

    let mut previous: Vec<EventWS> = if range.low < 0 {
        events_with_signups
            .filter(end_time.le(now))
            .order_by(start_time.desc())
            .limit(-range.low)
            .load(&connection)?
    } else {
        Vec::new()
    };

    let mut upcoming: Vec<EventWS> = if range.high > 0 {
        events_with_signups
            .filter(end_time.gt(now))
            .order_by(start_time.asc())
            .limit(range.high)
            .load(&connection)?
    } else {
        Vec::new()
    };

    if range.high < 0 {
        if (-range.high) as usize >= previous.len() {
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
pub fn get_event(event_id: i32, db_pool: State<DatabasePool>) -> Result<Json<EventWS>, StatusJson> {
    use schema::views::events_with_signups::dsl::*;

    let connection = db_pool.inner().get()?;
    let result = events_with_signups
        .find(event_id)
        .first::<EventWS>(&connection)?;
    Ok(Json(result))
}

/// Route `POST /event`
///
/// Post a new event.
#[post("/event", format = "application/json", data = "<event>")]
pub fn post_event(
    event: Json<NewEvent>,
    db_pool: State<DatabasePool>,
) -> Result<Json<Event>, StatusJson> {
    let event = event.into_inner();
    let connection = db_pool.inner().get()?;

    let result = diesel::insert_into(events::table)
        .values(event)
        .get_result(&connection)?;
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::{Event, EventWS};
    use diesel::RunQueryDsl;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use schema::tables::events;
    use util::catchers::catchers;
    use util::testing::{generate_new_events, DatabaseState};

    #[test]
    fn event_creation() {
        let (_state, db_pool) = DatabaseState::new();
        let rocket = rocket::ignite()
            .manage(db_pool)
            .catch(catchers())
            .mount("/", routes![super::post_event,]);
        let client = Client::new(rocket).expect("valid rocket instance");
        let events = generate_new_events(10, 10);

        for event in events {
            let mut response = client
                .post("/event")
                .body(serde_json::to_string(&event).expect("Could not serialize NewEvent"))
                .header(ContentType::JSON)
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            let body = response.body_string().expect("Response has no body");
            let event: Event =
                serde_json::from_str(&body).expect("Could not deserialize JSON into Event");
            assert_eq!(event.title, "My Event");
        }
    }

    #[test]
    fn get_event_list() {
        let (_state, db_pool) = DatabaseState::new();

        {
            let connection = db_pool.get().expect("Could not get database connection");
            for event in generate_new_events(10, 10).into_iter() {
                diesel::insert_into(events::table)
                    .values(event)
                    .execute(&connection)
                    .expect("Could not populate testing database");
            }
        }

        let rocket = rocket::ignite()
            .manage(db_pool)
            .mount("/", routes![super::get_events,]);

        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/events?low=-10&high=11").dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.body_string().expect("Response has no body");
        let events: Vec<EventWS> =
            serde_json::from_str(&body).expect("Could not deserialize JSON into Vec<EventWS>");
        println!("{:#?}", events);
        assert_eq!(events.len(), 20);
        assert!(events.iter().all(|event| event.title == "My Event"));
    }
}
