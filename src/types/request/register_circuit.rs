use rocket::{data::{self, FromData, ToByteUnit}, http::{ContentType, Status}, outcome::Outcome, Data, Request};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct RegisterCircuitRequest {
    pub name: String
}

#[derive(Debug)]
pub enum Error {
    TooLarge,
    Io(std::io::Error)
}

#[rocket::async_trait]
impl<'r> FromData<'r> for RegisterCircuitRequest {
    type Error = Error;
    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        use Error::*;
        // Content type must be json
        let json_ct = ContentType::JSON;
        if req.content_type() != Some(&json_ct) {
            return Outcome::Forward((data, Status::UnsupportedMediaType));
        }
        // Deserialise the request body into RegisterCircuitRequest
        let stream = match data.open(1024.kibibytes()).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Error((Status::PayloadTooLarge, TooLarge)),
            Err(e) => return Outcome::Error((Status::InternalServerError, Io(e))),
        };
        println!("request data {:?}", stream);
        let register_circuit_request = serde_json::from_str(&stream).unwrap();

        Outcome::Success(register_circuit_request)
    }
}

