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
 
#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, ping])
}