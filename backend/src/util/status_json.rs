use diesel::result::Error as DieselError;
use diesel::ConnectionError as DieselConnectionError;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::Request;
use rocket_contrib::json::Json;

/// An error message which can be serialized as JSON.
///
/// #### Example JSON
/// ```json
/// {
///   "status": 404,
///   "description": "Not Found"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct StatusJson {
    pub status: Status,
    pub description: String,
}

impl Responder<'static> for StatusJson {
    fn respond_to(self, req: &Request) -> Result<Response<'static>, Status> {
        let mut response = Json(json!({
            "status": self.status.code,
            "description": self.description,
        }))
        .respond_to(req)?;
        response.set_status(self.status);
        Ok(response)
    }
}

impl<T: ToString> From<T> for StatusJson {
    default fn from(e: T) -> StatusJson {
        StatusJson {
            status: Status::BadRequest,
            description: e.to_string(),
        }
    }
}

impl From<Status> for StatusJson {
    fn from(status: Status) -> StatusJson {
        StatusJson {
            description: status.reason.to_string(),
            status: status,
        }
    }
}

impl From<DieselError> for StatusJson {
    fn from(e: DieselError) -> StatusJson {
        match e {
            DieselError::NotFound => StatusJson {
                status: Status::NotFound,
                description: "Not Found in Database".into(),
            },
            err => StatusJson {
                status: Status::InternalServerError,
                description: err.to_string(),
            },
        }
    }
}

impl From<DieselConnectionError> for StatusJson {
    fn from(e: DieselConnectionError) -> StatusJson {
        StatusJson {
            status: Status::InternalServerError,
            description: e.to_string(),
        }
    }
}
