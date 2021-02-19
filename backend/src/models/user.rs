use crate::database::{DatabaseConn, DatabasePool};
use crate::schema::tables::users;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use frank_jwt::error::Error as JWTError;
use frank_jwt::{decode, encode, Algorithm, ValidationOptions};
use orion::errors::UnknownCryptoError;
use orion::pwhash::{hash_password, Password};
use rocket::http::Status;
use rocket::outcome::{try_outcome, Outcome};
use rocket::request::{self, FromRequest, Request};
use rocket::response::{self, Responder};
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value as JsonValue};

pub const PWHASH_ITERATIONS: u32 = 10000;
pub const PWHASH_MEMORY: u32 = 8192;

/// This struct defines a user object
///
/// It's used as a request guard: all routes with a User parameter will return
/// 401 UNAUTHORIZED if the client cannot provide proof of authentication.
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

pub struct JWT<T> {
    pub header: JWTHeader,
    pub payload: JWTPayload,

    config: JWTConfig,
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
            config: config.clone(),
            wrapped_response: (),
        }
    }
}

impl<T> JWT<T> {
    /// The implementation of Responder for JWT wraps the Responder of this value with an
    /// `Authorization`-header containing a bearer token.
    pub fn wrap<N>(self, response: N) -> JWT<N>
    where
        N: for<'r> Responder<'r, 'static>,
    {
        JWT {
            header: self.header,
            payload: self.payload,
            config: self.config,
            wrapped_response: response,
        }
    }

    pub fn encode_jwt(&self) -> Result<String, JWTError> {
        let payload = serde_json::to_value(&self.payload).expect("Failed to serialize JWT payload");

        let header = serde_json::to_value(&self.header).expect("Failed to serialize JWT header");

        encode(header, &self.config.secret, &payload, self.config.algorithm)
    }
}

impl<'a, T> Responder<'a, 'static> for JWT<T>
where
    T: for<'r> Responder<'r, 'static>,
{
    fn respond_to(self, request: &Request) -> response::Result<'static> {
        let jwt = self.encode_jwt().map_err(|e| {
            eprintln!("Failed to encode JWT: {}", e);
            Status::InternalServerError
        })?;

        let mut response = self.wrapped_response.respond_to(request)?;
        response.set_raw_header("Authorization", format!("Bearer {}", jwt));

        Ok(response)
    }
}

#[derive(Clone)]
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

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let db_pool: State<DatabasePool> = try_outcome!(request.guard().await);
        let jwt_config: State<JWTConfig> = try_outcome!(request.guard().await);

        if let Some(header) = request
            .headers()
            .get("Authorization")
            .filter(|header| header.starts_with("Bearer "))
            .map(|header| header.trim_start_matches("Bearer "))
            .next()
        {
            let jwt = decode(
                &header.to_owned(),
                &jwt_config.secret,
                jwt_config.algorithm,
                &ValidationOptions::new(),
            )
            .map_err(|e| {
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

            let payload: JWTPayload = match serde_json::from_value(payload) {
                Ok(payload) => payload,
                Err(e) => {
                    eprintln!(
                        "Failed to deserialize JsonValue into struct: {}\n\
                         This is a programmer error.",
                        e
                    );
                    return Outcome::Failure((Status::InternalServerError, ()));
                }
            };

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
    hash_password(&Password::from_slice(password.as_ref())?, PWHASH_ITERATIONS, PWHASH_MEMORY)
        .map(|pwhash| hex::encode(&pwhash.unprotected_as_bytes()))
}
