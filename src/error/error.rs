use serde::{Serialize, Deserialize};
use std::io::Cursor;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{content, status};
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;
//use core::resp::Error;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

//#[derive(Error)]
#[derive( Debug, Clone)]
pub enum CustomError {
    //#[resp("{0}")]
    Internal(String),

    //#[resp("{0}")]
    NotFound(String),

    //#[resp("{0}")]
    BadRequest(String),
}

impl CustomError {
    fn get_http_status(&self) -> Status {
        match self {
            CustomError::Internal(_) => Status::InternalServerError,
            CustomError::NotFound(_) => Status::NotFound,
            _ => Status::BadRequest,
        }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "Error {}.", self.get_http_status())
    }
}

impl std::error::Error for CustomError {}

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // serialize struct into json string
        let err_response = serde_json::to_string(&ErrorResponse{
            message: self.to_string()
        }).unwrap();

        Response::build()
            .status(self.get_http_status())
            .header(ContentType::JSON)
            .sized_body(err_response.len(), Cursor::new(err_response))
            .ok()
    }
}
