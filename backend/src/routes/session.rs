use crate::database::DatabasePool;
use crate::models::user::{
    generate_salted_hash, Credentials, JWTConfig, User, JWT, PWHASH_ITERATIONS,
};
use crate::util::StatusJson as SJ;
use diesel::prelude::*;
use hex;
use orion::pwhash::{hash_password_verify, Password, PasswordHash};
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

/// Route `GET /me`
///
/// Get metadata regarding the currently authenticated user
#[get("/me")]
pub fn user_info(user: User) -> JsonValue {
    // TODO: Return useful information
    json!({
        "user": {
            "name": user.name,
            "display_name": user.display_name,
        },
    })
}

#[get("/me", rank = 2)]
pub fn no_user() -> SJ {
    Status::Unauthorized.into()
}

/// Route `POST /login`
///
/// Authenticate a user
#[post("/login", data = "<credentials>")]
pub fn login(
    credentials: Json<Credentials>,
    db_pool: State<DatabasePool>,
    jwt_config: State<JWTConfig>,
) -> Result<JWT<SJ>, SJ> {
    use crate::schema::tables::users::dsl::*;
    let connection = db_pool.inner().get()?;
    let unauthorized_error = SJ {
        status: Status::Unauthorized,
        description: "Invalid Credentials".into(),
    };
    let user: User = users
        .find(&credentials.name)
        .first(&connection)
        /* Convert database errors into 403:s since we don't want to
         * leak information about whether the user exists or not */
        .map_err(|_| unauthorized_error.clone())?;

    let hash: Vec<u8> = hex::decode(user.salted_pass.clone()).map_err(|e| {
        eprintln!("Could not decode hex for user [{}] pass: {}", user.name, e);
        Status::InternalServerError
    })?;
    let hash = PasswordHash::from_slice(&hash)?;

    let valid = hash_password_verify(
        &hash,
        &Password::from_slice(credentials.pass.as_ref()),
        user.hash_iterations as usize,
        /* If the validation errors the password is wrong */
    )
    .map_err(|_| unauthorized_error.clone())?;

    if valid {
        Ok(JWT::new(&user, &jwt_config).wrap(SJ::new(Status::Ok, "Login successful")))
    } else {
        Err(unauthorized_error)
    }
}

/// Route `POST /register`
///
/// Create a new user
#[post("/register", data = "<credentials>")]
pub fn register(credentials: Json<Credentials>, db_pool: State<DatabasePool>) -> Result<SJ, SJ> {
    use crate::schema::tables::users;
    let connection = db_pool.inner().get()?;

    let user = User {
        name: credentials.name.clone(),
        display_name: None,
        salted_pass: generate_salted_hash(&credentials.pass).map_err(|_| SJ {
            status: Status::BadRequest,
            description: "Password needs to be longer than 13 characters".into(),
        })?,
        hash_iterations: PWHASH_ITERATIONS as i32,
    };

    let affected_rows: usize = diesel::insert_into(users::table)
        .values(user)
        .execute(&connection)?;

    if affected_rows == 1 {
        Ok(SJ {
            status: Status::Ok,
            description: "Registration successful".into(),
        })
    } else {
        Err(SJ {
            status: Status::InternalServerError,
            description: "Registration failed".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::JWTConfig;
    use crate::schema::tables::users;
    use crate::util::catchers::catchers;
    use crate::util::testing::DatabaseState;
    use diesel::RunQueryDsl;
    use rocket::http::{ContentType, Header, Status};
    use rocket::local::Client;

    #[test]
    fn log_in() {
        let (_state, db_pool) = DatabaseState::new();
        let jwt_config = JWTConfig::testing_config();

        let credentials = Credentials {
            name: "new_test_user".into(),
            pass: "My Extremely Secure Password".into(),
        };

        let user = User {
            name: credentials.name.clone(),
            display_name: Some("Bob Alicesson".into()),
            salted_pass: generate_salted_hash(&credentials.pass)
                .expect("Could not create password hash"),
            hash_iterations: PWHASH_ITERATIONS as i32,
        };
        let connection = db_pool.get().expect("Could not get database connection");
        diesel::insert_into(users::table)
            .values(user)
            .execute(&connection)
            .expect("Could not add new user for testing");

        let rocket = rocket::ignite()
            .manage(db_pool)
            .manage(jwt_config)
            .register(catchers())
            .mount("/", routes![login, user_info]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&credentials).expect("Could not serialize NewEvent"))
            .dispatch();

        let jwt_header = response
            .headers()
            .get_one("Authorization")
            .unwrap()
            .to_owned();
        assert_eq!(response.status(), Status::Ok);

        let mut response = client
            .get("/me")
            .header(Header::new("Authorization", jwt_header))
            .dispatch();

        let body = response.body_string().expect("Response has no body");
        let data: serde_json::Value =
            serde_json::from_str(&body).expect(&format!("Could not deserialize JSON: {}", body));
        assert!(data.is_object());
        let json = data.as_object().unwrap();
        assert!(json.contains_key("user"));
        assert_eq!(
            json.get("user").unwrap().get("name").unwrap(),
            &credentials.name
        );
        assert_eq!(response.status(), Status::Ok);
    }
}
