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
mod util;

use rocket::Request;
use rocket::http::Status;
use rocket_contrib::Json;
use diesel::prelude::*;
use self::models::{Event, NewEvent, EventRange};
use self::database::establish_connection;
use self::schema::events;
use self::util::ErrorJson;


#[get("/events?<range>")]
fn get_events(range: EventRange) -> Result<Json<Vec<Event>>, ErrorJson> {
    use self::schema::events::dsl::*;

    let connection = establish_connection();
    let results: Vec<Event> = events.order_by(start_time.asc())
        .load(&connection)?;
    Ok(Json(results))
}

#[get("/event/<event_id>")]
fn get_event(event_id: i32) -> Result<Json<Event>, ErrorJson>{
    use self::schema::events::dsl::*;

    let connection = establish_connection();
    let result = events.find(event_id)
        .first::<Event>(&connection)?;
    Ok(Json(result))
}

#[post("/event", format = "application/json", data="<event>")]
fn post_event(event: Json<NewEvent>) -> Result<Json<Event>, ErrorJson>{
    let event = event.into_inner();
    let connection = establish_connection();

    let result = diesel::insert_into(events::table)
        .values(event)
        .get_result(&connection)?;
    Ok(Json(result))
}

#[catch(404)]
fn not_found(_: &Request) -> ErrorJson { Status::NotFound.into() }


fn main() {
    rocket::ignite()
        .catch(catchers![
            not_found,
        ])
        .mount("/", routes![
               get_events,
               get_event,
               post_event,
        ]).launch();
}
