#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::env;

use rocket_contrib::json::{Json, JsonValue};

mod model;
use model::hero::Hero;

#[get("/hello")]
fn hello() -> &'static str {
  "Hello, world!"
}

#[get("/")]
fn heroes() -> Json<Hero> {
  let data = Hero {
    id: Some(1234),
    name: "John Doe",
    identity: "Hai",
    hometown: "Bandung",
    age: 32,
  };

  serde_json::to_string(&data).unwrap();
  Json(data)
}

fn main() {
  env::set_var(
    "RUST_LOG",
    env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
  );
  env_logger::init();

  info!("Hai from log");
  rocket::ignite()
    .mount("/", routes![hello])
    .mount("/heroes", routes![heroes])
    .launch();
}
