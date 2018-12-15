extern crate mqtt;

use std::io::Write;
use std::net;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::topic_filter::TopicFilter;
use mqtt::TopicName;
use mqtt::{Decodable, Encodable, QualityOfService};

const KEEP_ALIVE: u16 = 10;

pub fn connect(
  broker_addr: String,
  username: String,
  password: String,
  client_id: String,
  topic: &String,
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

  let channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![(
    TopicFilter::new("hello/me").unwrap(),
    QualityOfService::Level0,
  )];
  info!("Applying channel filters {:?} ...", channel_filters);
  let sub = SubscribePacket::new(10, channel_filters);
  let mut buf = Vec::new();
  sub.encode(&mut buf).unwrap();
  stream.write_all(&buf[..]).unwrap();

  loop {
    let packet = match VariablePacket::decode(&mut stream) {
      Ok(pk) => pk,
      Err(err) => {
        warn!("Error in receiving packet {:?}", err);
        continue;
      }
    };
    info!("PACKET {:?}", packet);

    if let VariablePacket::SubackPacket(ref ack) = packet {
      if ack.packet_identifier() != 10 {
        panic!("SUBACK packet identifier not match");
      }

      info!("Subscribed!");

      let msg = format!("hello");
      println!("Sending message '{}' to topic: '{}'", msg, "hello");
      publish(&mut stream, msg, topic.to_string());
      info!("Test publish!");
      break;
    }
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
