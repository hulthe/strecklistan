use super::context::Context;
use crate::database::event::{get_event_ws, get_event_ws_range};
use crate::models::event::{Event, EventWithSignups as EventWS, NewEvent};
use crate::models::signup::{NewSignup, Signup};
use diesel::prelude::*;
use juniper::{graphql_object, graphql_value, FieldError, FieldResult};

pub struct RootQuery;
graphql_object!(RootQuery: Context |&self| {

    field apiVersion() -> &str {
        env!("CARGO_PKG_VERSION")
    }

    field event(&executor, id: i32) -> FieldResult<EventWS>
        as "Get a specific event by ID" {
        let has_auth = gql_auth!(executor, Events(List(Read))).is_ok();

        Ok(get_event_ws(
            executor.context().pool.get()?,
            id,
            !has_auth,
        )?)
    }

    field events(&executor, low: i32, high: i32) -> FieldResult<Vec<EventWS>>
        as "Get a number of past and/or future events" {
        let has_auth = gql_auth!(executor, Events(List(Read))).is_ok();

        if low >= high {
            return Err(FieldError::new(
                "high must be higher than low",
                graphql_value!({ "bad_request": "Invalid range" })
            ));
        }

        Ok(get_event_ws_range(
            executor.context().pool.get()?,
            low.into(),
            high.into(),
            !has_auth,
        )?)
    }

    field signup(&executor, id: i32) -> FieldResult<Signup> {
        use crate::schema::tables::event_signups::dsl::{event_signups};
        gql_auth!(executor, Events(SignupRead))?;
        let connection = executor.context().pool.get()?;
        let result: Signup = event_signups.find(id).first(&connection)?;
        Ok(result)
    }
});

pub struct RootMutation;
graphql_object!(RootMutation: Context |&self| {
    field create_event(&executor, new_event: NewEvent) -> FieldResult<EventWS> {
        use crate::schema::tables::events;
        gql_auth!(executor, Events(List(Write)))?;
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
    use crate::models::{user::JWTConfig, Event, NewEvent};
    use crate::routes::graphql;
    use crate::schema::tables::events;
    use crate::util::catchers::catchers;
    use crate::util::testing::{DatabaseState, UserSession};
    use chrono::naive::NaiveDateTime;
    use diesel::RunQueryDsl;
    use rocket::http::{ContentType, Header};
    use rocket::local::Client;
    use rocket::routes;
    use rocket_contrib::json;

    #[test]
    fn test_create_event() {
        let (_state, db_pool) = DatabaseState::new();
        let jwt_config = JWTConfig::testing_config();
        let user_session = UserSession::new(&db_pool, &jwt_config);

        let rocket = rocket::ignite()
            .manage(db_pool)
            .manage(graphql::create_schema())
            .manage(jwt_config)
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
            .header(Header::new("Authorization", user_session.bearer))
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
        let jwt_config = JWTConfig::testing_config();
        let user_session = UserSession::new(&db_pool, &jwt_config);

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
            .manage(jwt_config)
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
            .header(Header::new("Authorization", user_session.bearer))
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
