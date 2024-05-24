use quantum_types::enums::proving_schemes::ProvingSchemes;
use rocket::{data::{self, FromData, ToByteUnit}, http::{ContentType, Status}, outcome::Outcome, Data, Request};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Debug, Deserialize)]
pub struct SubmitProofRequest {
    pub proof: Vec<u8>, // borsh serialised vkey
    pub pis: Vec<u8>,  // borsh serialised vkey
    pub circuit_hash: String,
    pub proof_type: ProvingSchemes
}

#[derive(Debug)]
pub enum Error {
    TooLarge,
    Io(std::io::Error)
}

#[rocket::async_trait]
impl<'r> FromData<'r> for SubmitProofRequest {
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
        info!("request data {:?}", stream);

        // TODO: we can convert types here only

        let submit_proof_request = serde_json::from_str(&stream).unwrap();
        Outcome::Success(submit_proof_request)
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SubmitProofResponse {
    pub proof_id: String,
}
