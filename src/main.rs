#![feature(proc_macro_hygiene, decl_macro, core_intrinsics)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate mqtt;

mod config;
use config::{read_config, Config};

use std::env;
use std::fs::File;
use std::thread;

use rocket_contrib::json::Json;

mod model;
use model::hero::Hero;

mod libs;
use libs::mqtt_lib;
use libs::utils;

#[get("/hello")]
fn hello() -> &'static str {
  "Hello, world!"
}

#[get("/am-i-up")]
fn am_i_up() -> &'static str {
  "OK"
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

fn parse_config() -> Config {
  const CONFIG_FILENAME: &'static str = "config.toml";
  let mut f = File::open(CONFIG_FILENAME).expect(&format!(
    "Can't open configuration file: {}",
    CONFIG_FILENAME
  ));
  read_config(&mut f).expect("Can't read configuration file.")
}

fn main() {
  env::set_var(
    "RUST_LOG",
    env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
  );
  env_logger::init();
  println!("{:?}", utils::get_unique_name().unwrap());

  let settings = parse_config();

  let mut stream = mqtt_lib::connect(
    settings.mqtt.broker_address,
    settings.mqtt.username,
    settings.mqtt.password,
    settings.mqtt.client_id,
    &settings.mqtt.topic,
  );

  mqtt_lib::publish(&mut stream, "Hai".to_string(), settings.mqtt.topic);

  info!("Hai from log");

  let _listen = thread::spawn(move || mqtt_lib::mqtt_subscribe_worker(stream));
  rocket::ignite()
    .mount("/", routes![hello])
    .mount("/status", routes![am_i_up])
    .mount("/heroes", routes![heroes])
    .launch();
}
