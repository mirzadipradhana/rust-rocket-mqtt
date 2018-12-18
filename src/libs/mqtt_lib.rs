extern crate futures;
extern crate mqtt;
extern crate time;
extern crate tokio;

use std::fmt::Debug;
use std::io::Write;
use std::net;
use std::str;
use std::thread;
use std::time::{Duration, Instant};

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::topic_filter::TopicFilter;
use mqtt::TopicName;
use mqtt::{Decodable, Encodable, QualityOfService};

use tokio::io::{self, AsyncRead};
use tokio::net::TcpStream;
use tokio::timer::Interval;

use futures::{future, Future, Stream};

const KEEP_ALIVE: u16 = 10;

fn alt_drop<E: Debug>(err: E) {
  warn!("{:?}", err);
}

pub fn mqtt_subscribe_listener(stream: net::TcpStream) {
  // connection made, start the async work
  let program = future::ok(()).and_then(move |()| {
    let stream = TcpStream::from_std(stream, &Default::default()).unwrap();
    let (mqtt_read, mqtt_write) = stream.split();

    let ping_time = Duration::new((KEEP_ALIVE / 2) as u64, 0);
    let ping_stream = Interval::new(Instant::now() + ping_time, ping_time);

    let ping_sender = ping_stream
      .map_err(alt_drop)
      .fold(mqtt_write, |mqtt_write, _| {
        info!("Sending PINGREQ to broker");

        let pingreq_packet = PingreqPacket::new();

        let mut buf = Vec::new();
        pingreq_packet.encode(&mut buf).unwrap();
        io::write_all(mqtt_write, buf)
          .map(|(mqtt_write, _buf)| mqtt_write)
          .map_err(alt_drop)
      });

    let receiver = future::loop_fn::<_, (), _, _>(mqtt_read, |mqtt_read| {
      VariablePacket::parse(mqtt_read).map(|(mqtt_read, packet)| {
        trace!("PACKET {:?}", packet);

        match packet {
          VariablePacket::PingrespPacket(..) => {
            info!("Receiving PINGRESP from broker ..");
          }
          VariablePacket::PublishPacket(ref publ) => {
            let msg = match str::from_utf8(&publ.payload_ref()[..]) {
              Ok(msg) => msg,
              Err(err) => {
                error!("Failed to decode publish message {:?}", err);
                return future::Loop::Continue(mqtt_read);
              }
            };
            info!("PUBLISH ({}): {}", publ.topic_name(), msg);
          }
          _ => {}
        }

        future::Loop::Continue(mqtt_read)
      })
    }).map_err(alt_drop);

    ping_sender.join(receiver).map(alt_drop)
  });

  tokio::run(program);
}

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
