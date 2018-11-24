use super::context::Context;
use chrono::Local;
use diesel::prelude::*;
use juniper::{FieldError, FieldResult};
use models::event::{Event, EventWithSignups as EventWS, NewEvent};
use models::signup::{Signup, NewSignup};

pub struct RootQuery;
graphql_object!(RootQuery: Context |&self| {

    field apiVersion() -> &str {
        env!("CARGO_PKG_VERSION")
    }

    field event(&executor, id: i32) -> FieldResult<EventWS>
        as "Get a specific event by ID" {
        use schema::views::events_with_signups::dsl::*;
        let has_auth = executor.context().get_auth("signups").is_ok();

        let connection = executor.context().pool.get()?;
        Ok(events_with_signups
            .find(id)
            .filter(published.eq(true).or(has_auth))
            .first(&connection)?)
    }

    field events(&executor, low: i32, high: i32) -> FieldResult<Vec<EventWS>>
        as "Get a number of past and/or future events" {
        use schema::views::events_with_signups::dsl::*;
        let has_auth = executor.context().get_auth("signups").is_ok();

        let low: i64 = low.into();
        let high: i64 = high.into();

        if low >= high {
            return Err(FieldError::new(
                "high must be higher than low",
                graphql_value!({ "bad_request": "Invalid range" })
            ));
        }

        let now = Local::now().naive_local();
        let connection = executor.context().pool.get()?;

        let mut previous: Vec<EventWS> = if low < 0 {
            events_with_signups
                .filter(end_time.le(now))
                .filter(published.eq(true).or(has_auth))
                .order_by(start_time.desc())
                .limit(-low)
                .load(&connection)?
        } else {
            Vec::new()
        };

        let mut upcoming: Vec<EventWS> = if high > 0 {
            events_with_signups
                .filter(end_time.gt(now))
                .filter(published.eq(true).or(has_auth))
                .order_by(start_time.asc())
                .limit(high)
                .load(&connection)?
        } else {
            Vec::new()
        };

        if high < 0 {
            if (-high) as usize >= previous.len() {
                previous = Vec::new();
            } else {
                previous.drain(..(-high as usize));
            }
        }

        if low > 0 {
            if low as usize >= upcoming.len() {
                upcoming = Vec::new();
            } else {
                upcoming.drain(..(low as usize));
            }
        }

        upcoming.reverse();

        upcoming.append(&mut previous);
        Ok(upcoming)
    }
});

pub struct RootMutation;
graphql_object!(RootMutation: Context |&self| {
    field create_event(&executor, new_event: NewEvent) -> FieldResult<EventWS> {
        use schema::tables::events;
        executor.context().get_auth("create_event")?;
        let connection = executor.context().pool.get()?;
        let event: Event = diesel::insert_into(events::table)
            .values(new_event)
            .get_result(&connection)?;
        Ok(event.into())
    }
});
