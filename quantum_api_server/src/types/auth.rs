use quantum_db::repository::auth::check_if_auth_token_registered_and_is_master;
use quantum_db::repository::protocol::get_protocol_by_auth_token;
use rocket::http::Status;
use rocket::request::{Request, FromRequest, Outcome};
use tracing::error;
use crate::connection::get_pool;
pub struct AuthToken(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            let mut auth_token = auth_header;
            let mut itr = auth_token.split_whitespace();
            itr.next();
            let token = itr.next();
            let is_present: Result<bool, anyhow::Error>;
            if token.is_some() {
                auth_token = token.unwrap();
            }
            if request.uri().path() == "/auth/protocol" {
                is_present =  check_if_auth_token_registered_and_is_master(get_pool().await, auth_token).await;
            } else {
                is_present = match get_protocol_by_auth_token(get_pool().await, auth_token).await {
                        Ok(p) => match p {
                            Some(_) => Ok(true),
                            None => Ok(false),
                        },
                        Err(e) => Err(e),
                    };
            }
            println!("{:?}", is_present);
            if is_present.is_err() {
                error!("unauthorized api access with access toekn: {:?}", auth_token);
                return Outcome::Error((Status::InternalServerError, ()));
            }
            if is_present.is_ok_and(|x| x == true) {
                return Outcome::Success(AuthToken(auth_token.to_string()));
            }
            error!("unauthorized api access with access toekn: {:?}", auth_token);
            Outcome::Error((Status::Unauthorized, ()))
        } else {
            error!("authorized token not present in the request");
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

