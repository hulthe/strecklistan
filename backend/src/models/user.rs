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
use rocket::response::{self, Responder};
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
pub struct JWT<T> {
    pub header: JWTHeader,
    pub payload: JWTPayload,

    wrapped_response: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JWTHeader {}

#[derive(Serialize, Deserialize, Debug)]
pub struct JWTPayload {
    /// [Subject](https://tools.ietf.org/html/rfc7519#section-4.1.2) Claim.
    /// Equal to the unique name of the issuing user.
    pub sub: String,

    /// [Expiration Time](https://tools.ietf.org/html/rfc7519#section-4.1.4) Claim.
    /// The time where the JWT ceases to be valid.
    /// Stored as POSIX time.
    pub exp: i64,
}

impl JWT<()> {
    pub fn new(user: &User, config: &JWTConfig) -> JWT<()> {
        JWT {
            header: JWTHeader {},
            payload: JWTPayload {
                sub: user.name.clone(),
                exp: (Utc::now() + config.token_lifetime).timestamp(),
            },
            wrapped_response: (),
        }
    }
}

impl<T> JWT<T> {
    /// The implementation of Responder for JWT wraps the Responder of this value with an
    /// `Authorization`-header containing a bearer token.
    pub fn wrap<N>(self, response: N) -> JWT<N>
    where
        N: for<'r> Responder<'r>,
    {
        JWT {
            header: self.header,
            payload: self.payload,
            wrapped_response: response,
        }
    }

    pub fn encode_jwt(&self, config: &JWTConfig) -> Result<String, JWTError> {
        let payload = serde_json::to_value(&self.payload).expect("Failed to serialize JWT payload");

        let header = serde_json::to_value(&self.header).expect("Failed to serialize JWT header");

        encode(header, &config.secret, &payload, config.algorithm)
    }
}

impl<'a, T> Responder<'a> for JWT<T>
where
    T: for<'r> Responder<'r>,
{
    fn respond_to(self, request: &Request) -> response::Result<'a> {
        let jwt_config = request.guard::<State<JWTConfig>>().success_or_else(|| {
            eprintln!("Failed to acquire JWT configuration");
            Status::InternalServerError
        })?;

        let jwt = self.encode_jwt(&jwt_config).map_err(|e| {
            eprintln!("Failed to encode JWT: {}", e);
            Status::InternalServerError
        })?;

        let mut response = self.wrapped_response.respond_to(request)?;
        response.set_raw_header("Authorization", format!("Bearer {}", jwt));

        Ok(response)
    }
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
                        // TODO
                        _ => Outcome::Failure((Status::InternalServerError, ())),
                    }
                });

            let (_header, payload): (JsonValue, JsonValue) = match jwt {
                Ok(jwt) => jwt,
                Err(failure) => {
                    return failure;
                }
            };

            let payload: JWTPayload = serde_json::from_value(payload).map_err(|e| {
                eprintln!(
                    "Failed to deserialize JsonValue into struct: {}\n\
                     This is a programmer error.",
                    e
                );
                Err((Status::InternalServerError, ()))
            })?;

            let now: i64 = Utc::now().timestamp();
            if payload.exp <= now {
                return Outcome::Forward(());
            }

            if let Ok(connection) = db_pool.inner().get() {
                if let Some(user) = get_user(payload.sub, &connection) {
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
