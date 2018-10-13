use diesel::result::Error as DieselError;
use diesel::ConnectionError as DieselConnectionError;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::Request;
use rocket_contrib::Json;

/// An error message which can be serialized as JSON.
///
/// #### Example JSON
/// ```json
/// {
///   "status": 404,
///   "description": "Not Found"
/// }
/// ```
#[derive(Debug)]
pub struct ErrorJson {
    pub status: Status,
    pub description: String,
}

impl Responder<'static> for ErrorJson {
    fn respond_to(self, req: &Request) -> Result<Response<'static>, Status> {
        let mut response = Json(json!({
            "status": self.status.code,
            "description": self.description,
        })).respond_to(req)?;
        response.set_status(self.status);
        Ok(response)
    }
}

impl<T: ToString> From<T> for ErrorJson {
    default fn from(e: T) -> ErrorJson {
        ErrorJson {
            status: Status::BadRequest,
            description: e.to_string(),
        }
    }
}

impl From<Status> for ErrorJson {
    fn from(status: Status) -> ErrorJson {
        ErrorJson {
            description: status.reason.to_string(),
            status: status,
        }
    }
}

impl From<DieselError> for ErrorJson {
    fn from(e: DieselError) -> ErrorJson {
        match e {
            DieselError::NotFound => ErrorJson {
                status: Status::NotFound,
                description: "Not Found in Database".into(),
            },
            err => ErrorJson {
                status: Status::InternalServerError,
                description: err.to_string(),
            },
        }
    }
}

impl From<DieselConnectionError> for ErrorJson {
    fn from(e: DieselConnectionError) -> ErrorJson {
        ErrorJson {
            status: Status::InternalServerError,
            description: e.to_string(),
        }
    }
}
