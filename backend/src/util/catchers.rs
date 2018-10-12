use rocket::http::Status;
use rocket::Request;
use util::ErrorJson;

#[catch(404)]
pub fn not_found(_: &Request) -> ErrorJson {
    ErrorJson {
        status: Status::NotFound.into(),
        description: "Route Not Found".into(),
    }
}

#[catch(401)]
pub fn unauthorized(_: &Request) -> ErrorJson {
    Status::Unauthorized.into()
}
