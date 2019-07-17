#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate colored;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate futures;
extern crate nitox;
extern crate serde_json;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_timer;
extern crate unicode_segmentation;
extern crate bytes;
extern crate nom;
extern crate uuid;
extern crate ws;

extern crate grinrelaylib;

mod broker;
mod server;

use crate::broker::Broker;
use crate::server::AsyncServer;
use std::net::ToSocketAddrs;
use grinrelaylib::types::{ChainTypes, set_running_mode};

fn main() {
    env_logger::init();

    info!("hello, world!");

    let broker_uri = std::env::var("BROKER_URI")
        .unwrap_or_else(|_| "127.0.0.1:61613".to_string())
        .to_socket_addrs()
        .unwrap()
        .next();

    let username = std::env::var("BROKER_USERNAME").unwrap_or("guest".to_string());
    let password = std::env::var("BROKER_PASSWORD").unwrap_or("guest".to_string());

    let grinrelay_domain = std::env::var("GRINRELAY_DOMAIN").unwrap_or("127.0.0.1".to_string());
    let grinrelay_port = std::env::var("GRINRELAY_PORT").unwrap_or("13420".to_string());
    let grinrelay_port = u16::from_str_radix(&grinrelay_port, 10).expect("invalid GRINRELAY_PORT given!");
    let grinrelay_protocol_unsecure = std::env::var("GRINRELAY_PROTOCOL_UNSECURE").map(|_| true).unwrap_or(false);

    let is_mainnet = std::env::var("GRINRELAY_IS_MAINNET").map(|_| true).unwrap_or(false);
    if is_mainnet {
        set_running_mode(ChainTypes::Mainnet);
    } else {
        set_running_mode(ChainTypes::Floonet);
    }

    if broker_uri.is_none() {
        error!("could not resolve broker uri!");
        panic!();
    }

    let broker_uri = broker_uri.unwrap();

    let bind_address =
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:13420".to_string());

    info!("Broker URI: {}", broker_uri);
    info!("Bind address: {}", bind_address);

    let mut broker = Broker::new(broker_uri, username, password);
    let sender = broker.start().expect("failed initiating broker session");
    let response_handlers_sender = AsyncServer::init();

    ws::Builder::new()
        .build(|out| AsyncServer::new(out, sender.clone(), response_handlers_sender.clone(), &grinrelay_domain, grinrelay_port, grinrelay_protocol_unsecure))
        .unwrap()
        .listen(&bind_address[..])
        .unwrap();
}
