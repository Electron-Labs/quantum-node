#[macro_use] extern crate rocket;

use rocket_db_pools::{Database, Connection};
use rocket_db_pools::diesel::{MysqlPool, prelude::*};
use service::register_circuit::register_circuit_exec;
use types::register_circuit::RegisterCircuitRequest;
use types::register_circuit::RegisterCircuitResponse;
use rocket::serde::json::Json;
mod types;
mod service;

#[derive(Database)]
#[database("diesel_mysql")]
struct Db(MysqlPool);

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
    rocket::build()
        .attach(Db::init())
        .mount("/", routes![index, ping, register_circuit])
}