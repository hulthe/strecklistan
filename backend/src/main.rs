#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

// Disable warnings caused by nightly rust phasing out this feature
#![allow(proc_macro_derive_resolution_fallback)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate dotenv;
extern crate chrono;

mod schema;
mod models;
mod database;

use rocket_contrib::Json;
use rocket::response::status;
use rocket::http::Status;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use self::models::{Event, NewEvent, EventRange};
use self::database::establish_connection;
use self::schema::events;

fn map_diesel_error(err: DieselError) -> status::Custom<Json> {
    match err {
        DieselError::NotFound => status::Custom(
            Status::NotFound,
            Json(json!({
                "status": 404,
                "description": "Not Found",
            })),
        ),
        err => status::Custom(
            Status::InternalServerError,
            Json(json!({
                "status": 500,
                "description": err.to_string(),
            })),
        ),
    }
}

#[get("/events?<range>")]
fn get_events(range: EventRange) -> Result<Json, status::Custom<Json>> {
    use self::schema::events::dsl::*;

    let connection = establish_connection();
    let results: Vec<Event> = events.order_by(start_time.asc())
        .load(&connection)
        .map_err(map_diesel_error)?;
    Ok(Json(json!({
        "events": results,
    })))
}

#[get("/event/<event_id>")]
fn get_event(event_id: i32) -> Result<Json<Event>, status::Custom<Json>>{
    use self::schema::events::dsl::*;

    let connection = establish_connection();
    let result = events.find(event_id)
        .first::<Event>(&connection)
        .map_err(map_diesel_error);

    match result {
        Ok(event) => Ok(Json(event)),
        Err(err) => Err(err),
    }
}

#[post("/event", format = "application/json", data="<event>")]
fn post_event(event: Json<NewEvent>) -> Result<Json<Event>, status::Custom<Json>>{
    let event = event.into_inner();
    let connection = establish_connection();

    let result: Result<Event, _> = diesel::insert_into(events::table)
        .values(event)
        .get_result(&connection)
        .map_err(map_diesel_error);
    match result {
        Ok(event) => Ok(Json(event)),
        Err(err) => Err(err),
    }
}


fn main() {
    rocket::ignite()
        .mount("/", routes![
               get_events,
               get_event,
               post_event
        ]).launch();
}

