use serde::Serialize;
use tracing::info;
use std::io::Cursor;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;
//use core::resp::Error;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_type: String,
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
        let error_message = match &self{
            Self::Internal(str) | Self::BadRequest(str) | Self::NotFound(str) => {
                info!("Error is: {}", str);
                str
            }
        };
        write!(fmt, "{}", error_message)
    }
}

impl std::error::Error for CustomError {}

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // serialize struct into json string
        
        let err_response = serde_json::to_string(&ErrorResponse{
            error_type: self.get_http_status().to_string(),
            message: self.to_string(), 
        }).unwrap();

        Response::build()
            .status(self.get_http_status())
            .header(ContentType::JSON)
            .sized_body(err_response.len(), Cursor::new(err_response))
            .ok()
    }
}