use crate::{service, types::auth::AuthToken};
use rocket::get;

#[get("/ping")]
pub fn ping(_auth_token: AuthToken) -> &'static str {
    service::ping::ping()
}