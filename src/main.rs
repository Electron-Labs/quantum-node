use service::register_circuit::register_circuit_exec;
use types::register_circuit::RegisterCircuitRequest;
use types::register_circuit::RegisterCircuitResponse;
use rocket::serde::json::Json;
mod types;
mod service;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ping")]
fn ping() -> &'static str {
    service::ping::ping()
}

#[post("/register_circuit", data = "<data>")]
fn register_circuit(data: RegisterCircuitRequest) -> Json<RegisterCircuitResponse> {
    Json(register_circuit_exec(data))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, ping, register_circuit])
}