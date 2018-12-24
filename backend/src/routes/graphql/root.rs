use super::context::Context;
use crate::models::event::{Event, EventWithSignups as EventWS, NewEvent};
use crate::models::signup::{NewSignup, Signup};
use chrono::Local;
use diesel::prelude::*;
use juniper::{FieldError, FieldResult};

pub struct RootQuery;
graphql_object!(RootQuery: Context |&self| {

    field apiVersion() -> &str {
        env!("CARGO_PKG_VERSION")
    }

    field event(&executor, id: i32) -> FieldResult<EventWS>
        as "Get a specific event by ID" {
        use crate::schema::views::events_with_signups::dsl::{events_with_signups, published};
        let has_auth = executor.context().get_auth("event").is_ok();

        let connection = executor.context().pool.get()?;
        Ok(events_with_signups
            .find(id)
            .filter(published.eq(true).or(has_auth))
            .first(&connection)?)
    }

    field events(&executor, low: i32, high: i32) -> FieldResult<Vec<EventWS>>
        as "Get a number of past and/or future events" {
        use crate::schema::views::events_with_signups::dsl::*;
        let has_auth = executor.context().get_auth("events").is_ok();

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

    field signup(&executor, id: i32) -> FieldResult<Signup> {
        use crate::schema::tables::event_signups::dsl::{event_signups};
        executor.context().get_auth("signup")?;
        let connection = executor.context().pool.get()?;
        let result: Signup = event_signups.find(id).first(&connection)?;
        Ok(result)
    }
});

pub struct RootMutation;
graphql_object!(RootMutation: Context |&self| {
    field create_event(&executor, new_event: NewEvent) -> FieldResult<EventWS> {
        use crate::schema::tables::events;
        executor.context().get_auth("create_event")?;
        let connection = executor.context().pool.get()?;
        let event: Event = diesel::insert_into(events::table)
            .values(new_event)
            .get_result(&connection)?;
        Ok(event.into())
    }

    field create_signup(&executor, new_signup: NewSignup) -> FieldResult<Signup> {
        use crate::schema::tables::event_signups;
        // TODO: Some sort of captcha
        let connection = executor.context().pool.get()?;
        let signup: Signup = diesel::insert_into(event_signups::table)
            .values(new_signup)
            .get_result(&connection)?;
        Ok(signup.into())
    }
});

#[cfg(test)]
mod tests {
    use crate::models::{Event, NewEvent};
    use crate::routes::graphql;
    use crate::schema::tables::events;
    use crate::util::catchers::catchers;
    use crate::util::testing::{DatabaseState, UserSession};
    use chrono::naive::NaiveDateTime;
    use diesel::RunQueryDsl;
    use rocket::http::ContentType;
    use rocket::local::Client;

    #[test]
    fn test_create_event() {
        let (_state, db_pool) = DatabaseState::new();
        let user_session = UserSession::new(&db_pool);

        let rocket = rocket::ignite()
            .manage(db_pool)
            .manage(graphql::create_schema())
            .register(catchers())
            .mount(
                "/",
                routes![
                    graphql::post_graphql_handler_auth,
                    graphql::post_graphql_handler,
                ],
            );
        let client = Client::new(rocket).unwrap();

        let new_event = json!({
            "title": "Test Event",
            "background": "http://test.ru/jpg.png",
            "location": "Foobar CA",
            "startTime": 10_000_000_000i64,
            "endTime": 10_000_001_000i64,
        });
        println!("Request Data: {:#?}\n", &new_event);

        let mut response = client
            .post("/graphql")
            .header(ContentType::JSON)
            .body(
                json!({
                    "operationName": "CreateEvent",
                    "query": "mutation CreateEvent($ev:NewEvent!) {\n\
                            createEvent(newEvent: $ev) {\n\
                                id        \n\
                                title     \n\
                                background\n\
                                location  \n\
                                startTime \n\
                                endTime   \n\
                                price     \n\
                                published \n\
                            }\n\
                        }",
                    "variables": {
                        "ev": new_event,
                    }
                })
                .to_string(),
            )
            .private_cookie(user_session.cookie)
            .dispatch();

        let body = response.body_string().expect("Response has no body");
        let data: serde_json::Value =
            serde_json::from_str(&body).expect(&format!("Could not deserialize JSON: {}", body));

        assert!(data.is_object());
        let json = data.as_object().unwrap();
        println!("Response Data: {:#?}\n", json);
        assert!(json.contains_key("data"));
        let graphql_data = json.get("data").unwrap().as_object().unwrap();

        assert!(graphql_data.contains_key("createEvent"));
        let graphql_data = graphql_data
            .get("createEvent")
            .unwrap()
            .as_object()
            .unwrap();

        assert!(graphql_data.contains_key("id"));
        assert!(graphql_data.contains_key("title"));
        assert!(graphql_data.contains_key("background"));
        assert!(graphql_data.contains_key("location"));
        assert!(graphql_data.contains_key("startTime"));
        assert!(graphql_data.contains_key("endTime"));
        assert!(graphql_data.contains_key("price"));
        assert!(graphql_data.contains_key("published"));
    }

    #[test]
    fn test_get_event() {
        let (_state, db_pool) = DatabaseState::new();
        let user_session = UserSession::new(&db_pool);

        let new_event = NewEvent {
            title: "Test Event 2".into(),
            background: "http://image.ru/png.jpg".into(),
            location: "Fizzbuzz TX".into(),
            start_time: NaiveDateTime::from_timestamp(10_000_000_00i64, 0),
            end_time: NaiveDateTime::from_timestamp(10_000_001_00i64, 0),
            price: None,
        };
        println!("Request Data: {:#?}\n", &new_event);

        let connection = db_pool.get().expect("Could not get database connection");
        let event: Event = diesel::insert_into(events::table)
            .values(&new_event)
            .get_result(&connection)
            .expect("Could not add new user for testing");

        let rocket = rocket::ignite()
            .manage(db_pool)
            .manage(graphql::create_schema())
            .register(catchers())
            .mount(
                "/",
                routes![
                    graphql::post_graphql_handler_auth,
                    graphql::post_graphql_handler,
                ],
            );
        let client = Client::new(rocket).unwrap();

        let mut response = client
            .post("/graphql")
            .header(ContentType::JSON)
            .body(
                json!({
                "operationName": "GetEvent",
                "query": format!("query GetEvent {{\n\
                        event(id: {}) {{\n\
                            id        \n\
                            title     \n\
                            background\n\
                            startTime \n\
                            endTime   \n\
                            price     \n\
                        }}\n\
                    }}", event.id),
                })
                .to_string(),
            )
            .private_cookie(user_session.cookie)
            .dispatch();

        let body = response.body_string().expect("Response has no body");
        let data: serde_json::Value =
            serde_json::from_str(&body).expect(&format!("Could not deserialize JSON: {}", body));

        assert!(data.is_object());
        let json = data.as_object().unwrap();
        println!("Response Data: {:#?}\n", json);
        assert!(json.contains_key("data"));
        let graphql_data = json.get("data").unwrap().as_object().unwrap();

        assert!(graphql_data.contains_key("event"));
        let graphql_data = graphql_data.get("event").unwrap().as_object().unwrap();

        assert!(graphql_data.contains_key("id"));
        assert!(graphql_data.contains_key("title"));
        assert!(graphql_data.contains_key("background"));
        assert!(!graphql_data.contains_key("location"));
        assert!(graphql_data.contains_key("startTime"));
        assert!(graphql_data.contains_key("endTime"));
        assert!(graphql_data.contains_key("price"));
        assert!(!graphql_data.contains_key("published"));
    }
}
