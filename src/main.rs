use types::request::register_circuit::RegisterCircuitRequest;
use types::response::regsiter_circuit::RegisterCircuitResponse;
use requests::register_circuit::register_circuit;
use rocket::serde::json::Json;
mod types;
mod requests;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ping")]
fn ping() -> &'static str {
    requests::ping::ping()
}

#[post("/regsiter_circuit", data = "<data>")]
fn regsiter_circuit(data: RegisterCircuitRequest) -> Json<RegisterCircuitResponse> {
    Json(register_circuit(data))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, ping, regsiter_circuit])
}