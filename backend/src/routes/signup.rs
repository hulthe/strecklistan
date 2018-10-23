use diesel::prelude::*;
use models::signup::{NewSignup, Signup};
use rocket::State;
use rocket_contrib::Json;
use database::DatabasePool;
use util::StatusJson;

/// Route `GET /signup/<signup_id>`
///
/// Get a specific event signup by its id parameter.
#[get("/signup/<signup_id>")]
pub fn get_signup(signup_id: i32, db_pool: State<DatabasePool>) -> Result<Json<Signup>, StatusJson> {
    use schema::tables::event_signups::dsl::*;
    let connection = db_pool.inner().get()?;
    let result: Signup = event_signups.find(signup_id).first(&connection)?;
    Ok(Json(result))
}

/// Route `GET /event/<event_id>/signups`
///
/// Get all signups for a specific event
#[get("/event/<event_id>/signups")]
pub fn get_event_signups(
    event_id: i32, db_pool: State<DatabasePool>,
) -> Result<Json<Vec<Signup>>, StatusJson> {
    use schema::tables::event_signups::dsl::*;
    let connection = db_pool.inner().get()?;
    let result: Vec<Signup> =
        event_signups.filter(event.eq(event_id)).load(&connection)?;
    // TODO: return 404 on non-existent event
    Ok(Json(result))
}

/// Route `POST /signup`
///
/// Post a new event.
#[post("/signup", format = "application/json", data = "<signup>")]
pub fn post_signup(
    signup: Json<NewSignup>, db_pool: State<DatabasePool>,
) -> Result<Json<Signup>, StatusJson> {
    use schema::tables::event_signups;
    let signup = signup.into_inner();
    let connection = db_pool.inner().get()?;

    let result = diesel::insert_into(event_signups::table)
        .values(signup)
        .get_result(&connection)?;
    // TODO: improve error message for non-existent event
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use diesel::RunQueryDsl;
    use models::{Event, Signup};
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use schema::tables::events;
    use util::catchers::catchers;
    use util::testing::{generate_event_signups, generate_new_events,
                        DatabaseState};

    #[test]
    fn create_signup() {
        let (_state, db_pool)= DatabaseState::new();
        let connection = db_pool.get().expect("Could not get database connection");
        let rocket = rocket::ignite()
            .manage(db_pool)
            .catch(catchers())
            .mount("/",
            routes![
                super::post_signup,
                super::get_signup,
                super::get_event_signups,
            ],
        );
        let client = Client::new(rocket).expect("valid rocket instance");

        for new_event in generate_new_events(5, 5) {
            let event: Event = diesel::insert_into(events::table)
                .values(new_event)
                .get_result(&connection)
                .unwrap();

            let signups = generate_event_signups(5, event.id);
            for new_signup in signups {
                let mut response = client
                    .post("/signup")
                    .body(serde_json::to_string(&new_signup).expect(
                        "Could not serialize NewSignup",
                    ))
                    .header(ContentType::JSON)
                    .dispatch();

                assert_eq!(response.status(), Status::Ok);

                let body =
                    response.body_string().expect("Response has no body");
                let signup: Signup = serde_json::from_str(&body).expect(
                    "Could not deserialize JSON into Signup",
                );
                assert_eq!(signup.name, new_signup.name);
                assert_eq!(signup.email, new_signup.email);
                assert_eq!(signup.event, new_signup.event);

                let mut response =
                    client.get(format!("/signup/{}", signup.id)).dispatch();
                let body =
                    response.body_string().expect("Response has no body");
                let signup2: Signup = serde_json::from_str(&body).expect(
                    "Could not deserialize JSON into Signup",
                );
                assert_eq!(signup, signup2);
            }
        }
    }
}
