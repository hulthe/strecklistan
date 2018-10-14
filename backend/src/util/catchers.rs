use rocket::http::Status;
use rocket::Request;
use util::StatusJson;

#[catch(404)]
pub fn not_found(_: &Request) -> StatusJson {
    StatusJson {
        status: Status::NotFound.into(),
        description: "Route Not Found".into(),
    }
}

#[catch(401)]
pub fn unauthorized(_: &Request) -> StatusJson {
    Status::Unauthorized.into()
}

#[catch(400)]
pub fn bad_request(_: &Request) -> StatusJson {
    Status::BadRequest.into()
}
