use crate::database::{DatabaseConn, DatabasePool};
use crate::schema::tables::users;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use frank_jwt::error::Error as JWTError;
use frank_jwt::{decode, encode, Algorithm};
use orion::errors::UnknownCryptoError;
use orion::pwhash::{hash_password, Password};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, State};
use serde_json::{self, Value as JsonValue};

pub const PWHASH_ITERATIONS: usize = 10000;
/// This struct defines a user object
///
/// It's used as a request guard: all routes with a User parameter will return
/// 401 UNAUTHORIZED if the client has not previously authenticated.
#[derive(Queryable, Insertable, Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub display_name: Option<String>,
    pub salted_pass: String,
    pub hash_iterations: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub username: String,
    pub last_seen: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JWTResponse {
    pub jwt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JWTPayload {
    pub user: String,
    pub expire: i64,
}

pub struct JWTConfig {
    pub secret: String,
    pub algorithm: Algorithm,
    pub token_lifetime: Duration,
}

#[cfg(test)]
impl JWTConfig {
    pub fn testing_config() -> Self {
        JWTConfig {
            secret: "secret".into(),
            algorithm: Algorithm::HS512,
            token_lifetime: Duration::days(2),
        }
    }
}

pub fn generate_jwt_session(user: &User, config: &JWTConfig) -> Result<String, JWTError> {
    let payload = serde_json::json!({
        "user": user.name.clone(),
        "expire": (Utc::now() + config.token_lifetime).timestamp(),
    });

    let header = serde_json::json!({});

    encode(header, &config.secret, &payload, config.algorithm)
}

fn get_user(user_name: String, connection: &DatabaseConn) -> Option<User> {
    use crate::schema::tables::users::dsl::*;
    users.find(user_name).first(connection).ok()
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let db_pool: State<DatabasePool> = request.guard()?;
        let jwt_config: State<JWTConfig> = request.guard()?;

        if let Some(header) = request
            .headers()
            .get("Authorization")
            .filter(|header| header.starts_with("Bearer "))
            .map(|header| header.trim_start_matches("Bearer "))
            .next()
        {
            let jwt =
                decode(&header.to_owned(), &jwt_config.secret, jwt_config.algorithm).map_err(|e| {
                    eprintln!("{}", e);
                    match e {
                        _ => Outcome::Failure((Status::InternalServerError, ())),
                    }
                });

            let (_header, payload): (JsonValue, JsonValue) = match jwt {
                Ok(jwt) => jwt,
                Err(failure) => {
                    return failure;
                }
            };

            let payload: JWTPayload = serde_json::from_value(payload)
                //.map_err(|e| Outcome::Forward(()))
                .unwrap(); // TODO

            let now: i64 = Utc::now().timestamp();
            if payload.expire <= now {
                return Outcome::Forward(());
            }

            if let Ok(connection) = db_pool.inner().get() {
                if let Some(user) = get_user(payload.user, &connection) {
                    Outcome::Success(user)
                } else {
                    Outcome::Forward(())
                }
            } else {
                Outcome::Failure((Status::InternalServerError, ()))
            }
        } else {
            Outcome::Forward(())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub name: String,
    pub pass: String,
}

pub fn generate_salted_hash<T: AsRef<[u8]>>(password: T) -> Result<String, UnknownCryptoError> {
    hash_password(&Password::from_slice(password.as_ref()), PWHASH_ITERATIONS)
        .map(|pwhash| hex::encode(&pwhash.unprotected_as_bytes()))
}
