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
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate tokio;

mod config;
use config::{read_config, Config};

use std::env;
use std::fs::File;
use std::net::TcpStream;
use std::sync::Mutex;
use std::thread;

use rocket_contrib::json::Json;

mod model;
use model::hero::Hero;

mod libs;
use libs::{mqtt_lib, utils};

lazy_static! {
  static ref CONFIG: Config = parse_config();
  static ref MQTT_STREAMS: Mutex<Vec<TcpStream>> = Mutex::new(vec![]);
}

#[get("/hello")]
fn hello() -> &'static str {
  "Hello, world!"
}

#[get("/am-i-up")]
fn am_i_up() -> &'static str {
  hai();
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

#[get("/broadcast/<msg>")]
fn publish(msg: String) -> &'static str {
  mqtt_lib::publish(
    &mut MQTT_STREAMS.lock().unwrap()[1].try_clone().unwrap(),
    msg.to_string(),
    CONFIG.mqtt.topic.clone(),
  );
  "OK"
}

fn parse_config() -> Config {
  const CONFIG_FILENAME: &'static str = "config.toml";
  let mut f = File::open(CONFIG_FILENAME).expect(&format!(
    "Can't open configuration file: {}",
    CONFIG_FILENAME
  ));
  read_config(&mut f).expect("Can't read configuration file.")
}

fn hai() {
  info!("Hai");
}

fn main() {
  env::set_var(
    "RUST_LOG",
    env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
  );
  env_logger::init();
  println!("{:?}", utils::get_unique_name().unwrap());

  let settings = parse_config();

  MQTT_STREAMS.lock().unwrap().push(mqtt_lib::connect(
    CONFIG.mqtt.broker_address.clone(),
    CONFIG.mqtt.username.clone(),
    CONFIG.mqtt.password.clone(),
    CONFIG.mqtt.client_id.clone(),
    &CONFIG.mqtt.topic.clone(),
  ));

  MQTT_STREAMS.lock().unwrap().push(mqtt_lib::connect(
    CONFIG.mqtt.broker_address.clone(),
    CONFIG.mqtt.username.clone(),
    CONFIG.mqtt.password.clone(),
    CONFIG.mqtt.client_id.clone(),
    &CONFIG.mqtt.topic.clone(),
  ));

  mqtt_lib::publish(
    &mut MQTT_STREAMS.lock().unwrap()[1],
    "Hai hoo".to_string(),
    settings.mqtt.topic,
  );

  let _listen = thread::spawn(move || {
    mqtt_lib::mqtt_subscribe_listener(MQTT_STREAMS.lock().unwrap()[0].try_clone().unwrap())
  });
  rocket::ignite()
    .mount("/", routes![hello])
    .mount("/status", routes![am_i_up])
    .mount("/heroes", routes![heroes])
    .mount("/publish", routes![publish])
    .launch();
}
