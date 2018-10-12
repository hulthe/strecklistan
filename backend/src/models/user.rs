use database::establish_connection;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use schema::tables::users::dsl::*;

/// This struct defines a user object
///
/// It's used as a request guard: all routes with a User parameter will return
/// 401 UNAUTHORIZED if the client has not previously authenticated.
#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub display_name: Option<String>,
}

fn get_user(user_name: String) -> Option<User> {
    let connection = establish_connection().ok()?;
    users.find(user_name).first(&connection).ok()
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let mut cookies = request.cookies();
        let user = cookies
            .get_private("session")
            .map(|session| String::from(session.value()))
            .and_then(|user_name| get_user(user_name));

        if let Some(user) = user {
            return Outcome::Success(user);
        }

        return Outcome::Failure((Status::Unauthorized, ()));
    }
}
