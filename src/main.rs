#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;

mod broker;
mod server;

use crate::broker::Broker;
use crate::server::AsyncServer;
use colored::*;
use grinrelaylib::types::{set_running_mode, ChainTypes};
use std::collections::HashMap;
use std::default::Default;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use openssl::pkey::PKey;
use openssl::ssl::{SslAcceptor, SslMethod};
use openssl::x509::X509;

extern crate amqp;
extern crate env_logger;

use amqp::protocol::basic;
use amqp::AMQPScheme;
use amqp::TableEntry::LongString;
use amqp::{Basic, Channel, Options, Session, Table};

fn read_file(name: &str) -> std::io::Result<Vec<u8>> {
	let mut file = File::open(name)?;
	let mut buf = Vec::new();
	file.read_to_end(&mut buf)?;
	Ok(buf)
}

pub fn rabbit_consumer_monitor(consumers: Arc<Mutex<HashMap<String, String>>>) {
	thread::spawn(|| {
		info!("rabbit_consumer_monitor ******** start!");
		let options = Options {
			host: "127.0.0.1".to_string(),
			port: 5672,
			vhost: "/".to_string(),
			login: "admin".to_string(),
			password: "admin".to_string(),
			frame_max_limit: 131072,
			channel_max_limit: 65535,
			locale: "en_US".to_string(),
			scheme: AMQPScheme::AMQP,
			properties: Table::new(),
		};

		let mut session = Session::new(options).ok().expect("Can't create session");
		let mut channel = session
			.open_channel(1)
			.ok()
			.expect("Error openning channel 1");
		info!("Openned channel: {:?}", channel.id);

		let queue_name = "test_queue";
		//		let queue_builder = QueueBuilder::named(queue_name).durable();
		//		let queue_declare = queue_builder.declare(&mut channel);

		let bind_result = channel.queue_bind(
			"test_queue",
			"amq.rabbitmq.event",
			"queue.*",
			false,
			Table::new(),
		);
		if bind_result.is_ok() {
			info!("queue binding succed!");
		}

		let closure_consumer = move |_chan: &mut Channel,
		                             deliver: basic::Deliver,
		                             headers: basic::BasicProperties,
		                             _data: Vec<u8>| {
			if deliver.routing_key == "consumer.created" {
				let header = headers.to_owned().headers.unwrap();
				let queue = match header.get("queue").unwrap() {
					LongString(val) => val.to_string(),
					_ => "test_queue".to_string(),
				};

				if queue.starts_with("gn1") || queue.starts_with("tn1") {
					let key = queue.clone().get(61..67).unwrap().to_owned();
					if !consumers.lock().unwrap().contains_key(&key) {
						consumers.lock().unwrap().insert(key, queue);
					}

					for (last6code, full_name) in consumers.lock().unwrap().iter() {
						info!("consumer created: {}: {}", last6code, full_name);
					}
				}
			}

			if deliver.routing_key == "consumer.deleted" {
				let header = headers.to_owned().headers.unwrap();

				let queue = match header.get("queue").unwrap() {
					LongString(val) => val.to_string(),
					_ => "test_queue".to_string(),
				};

				if queue.starts_with("gn1") || queue.starts_with("tn1") {
					let key = queue.clone().get(61..67).unwrap().to_owned();
					if consumers.lock().unwrap().contains_key(&key) {
						consumers.lock().unwrap().remove(&key);
					}

					for (last6code, full_name) in consumers.lock().unwrap().iter() {
						info!("consumer deleted: {}: {}", last6code, full_name);
					}
				}
			}
		};
		let consumer_name = channel.basic_consume(
			closure_consumer,
			queue_name,
			"",
			false,
			false,
			false,
			false,
			Table::new(),
		);
		info!("Starting consumer {:?}", consumer_name);

		channel.start_consuming();

		channel.close(200, "Bye").unwrap();
		session.close(200, "Good Bye");
		info!("rabbit_consumer_monitor ******** exit!");
	});
}

fn main() {
	env_logger::init();

	info!("hello, world! let's serve for a much easier transaction :-)");

	let broker_uri = std::env::var("BROKER_URI")
		.unwrap_or_else(|_| "127.0.0.1:61613".to_string())
		.to_socket_addrs()
		.unwrap()
		.next();

	let grinrelay_protocol_unsecure = std::env::var("GRINRELAY_PROTOCOL_UNSECURE")
		.map(|_| true)
		.unwrap_or(false);

	let acceptor = if !grinrelay_protocol_unsecure {
		info!("{}", "wss enabled".bright_green());
		let cert_file = std::env::var("CERT")
			.unwrap_or("/etc/grinrelay/tls/server_certificate.pem".to_string());
		let key_file =
			std::env::var("KEY").unwrap_or("/etc/grinrelay/tls/server_key.pem".to_string());

		let cert = {
			let data = read_file(cert_file.as_str()).unwrap();
			X509::from_pem(data.as_ref()).unwrap()
		};

		let pkey = {
			let data = read_file(key_file.as_str()).unwrap();
			PKey::private_key_from_pem(data.as_ref()).unwrap()
		};

		Some(Rc::new({
			let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
			builder.set_private_key(&pkey).unwrap();
			builder.set_certificate(&cert).unwrap();

			builder.build()
		}))
	} else {
		None
	};

	let username = std::env::var("BROKER_USERNAME").unwrap_or("guest".to_string());
	let password = std::env::var("BROKER_PASSWORD").unwrap_or("guest".to_string());

	let grinrelay_domain = std::env::var("GRINRELAY_DOMAIN").unwrap_or("127.0.0.1".to_string());
	let grinrelay_port = std::env::var("GRINRELAY_PORT").unwrap_or("13420".to_string());
	let grinrelay_port =
		u16::from_str_radix(&grinrelay_port, 10).expect("invalid GRINRELAY_PORT given!");

	let is_mainnet = std::env::var("GRINRELAY_IS_MAINNET")
		.map(|_| true)
		.unwrap_or(false);
	if is_mainnet {
		set_running_mode(ChainTypes::Mainnet);
	} else {
		set_running_mode(ChainTypes::Floonet);
	}

	if broker_uri.is_none() {
		error!("could not resolve broker uri!");
		panic!();
	}

	let consumers = Arc::new(Mutex::new(HashMap::new()));
	rabbit_consumer_monitor(consumers.clone());

	let broker_uri = broker_uri.unwrap();

	let bind_address =
		std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:13420".to_string());

	info!("Broker URI: {}", broker_uri);
	info!("Bind address: {}", bind_address);

	let mut broker = Broker::new(broker_uri, username, password, consumers);
	let sender = broker.start().expect("failed initiating broker session");
	let response_handlers_sender = AsyncServer::init();

	thread::spawn(|| {
		// for server selection service only
		let listener = TcpListener::bind("0.0.0.0:3419").unwrap();

		// accept connections and process them serially
		for stream in listener.incoming() {
			if let Ok(stream) = stream {
				trace!("server selection from {}", stream.peer_addr().unwrap());
			}
		}
	});

	ws::Builder::new()
		.with_settings(ws::Settings {
			encrypt_server: !grinrelay_protocol_unsecure,
			..ws::Settings::default()
		})
		.build(|out: ws::Sender| {
			AsyncServer::new(
				out,
				sender.clone(),
				response_handlers_sender.clone(),
				&grinrelay_domain,
				grinrelay_port,
				grinrelay_protocol_unsecure,
				acceptor.clone(),
			)
		})
		.unwrap()
		.listen(&bind_address[..])
		.unwrap();
}
