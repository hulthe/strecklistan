use rocket::Request;
use rocket::response::{Response, Responder};
use rocket::http::Status;
use rocket_contrib::Json;
use diesel::result::Error as DieselError;

#[derive(Debug)]
pub struct ErrorJson {
    pub status: Status,
    pub description: String,
}

impl Responder<'static> for ErrorJson {
    fn respond_to(self, req: &Request) -> Result<Response<'static>, Status> {
        Json(json!({
            "status": self.status.code,
            "description": self.description,
        })).respond_to(req)
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
                description: "Not Found".into(),
            },
            err => ErrorJson {
                status: Status::InternalServerError,
                description: err.to_string(),
            },
        }
    }
}
