extern crate serde;
extern crate toml;

use std::io;

#[derive(Debug)]
pub enum ConfigError {
  Io(io::Error),
  Parse(toml::de::Error),
}

#[derive(Deserialize)]
pub struct Config {
  pub mqtt: MQTT,
}

#[derive(Deserialize)]
pub struct MQTT {
  pub client_id: String,
  pub broker_address: String,
  pub username: String,
  pub password: String,
  pub topic: String,
}

pub fn read_config<T: io::Read + Sized>(mut f: T) -> Result<Config, ConfigError> {
  let mut buffer = String::new();
  f.read_to_string(&mut buffer).map_err(ConfigError::Io)?;
  toml::from_str(&buffer).map_err(ConfigError::Parse)
}
