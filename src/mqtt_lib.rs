extern crate mqtt;

use std::io::Write;
use std::net;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicName;
use mqtt::{Decodable, Encodable};

const KEEP_ALIVE: u16 = 10;

pub fn connect(
  broker_addr: String,
  username: String,
  password: String,
  client_id: String,
) -> net::TcpStream {
  info!("Connecting to {:?} ... ", broker_addr);
  let mut stream = net::TcpStream::connect(broker_addr).unwrap();
  info!("Connected!");

  info!("Client identifier {:?}", client_id);
  let mut conn = ConnectPacket::new("MQTT", client_id);
  conn.set_clean_session(true);
  conn.set_keep_alive(KEEP_ALIVE);
  let mut buf = Vec::new();
  conn.encode(&mut buf).unwrap();
  stream.write_all(&buf[..]).unwrap();

  let connack = ConnackPacket::decode(&mut stream).unwrap();
  trace!("CONNACK {:?}", connack);

  if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
    panic!(
      "Failed to connect to server, return code {:?}",
      connack.connect_return_code()
    );
  }

  stream
}

pub fn publish(stream: &mut net::TcpStream, msg: String, topic_name: String) {
  info!("Sending message '{}' to topic: '{}'", msg, topic_name);

  let topic = TopicName::new(topic_name.clone()).unwrap();
  let packet = PublishPacket::new(topic, QoSWithPacketIdentifier::Level1(10), msg);
  let mut buf = Vec::new();
  packet.encode(&mut buf).unwrap();
  stream.write_all(&buf).unwrap();
}
