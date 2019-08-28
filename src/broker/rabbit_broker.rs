// Copyright 2018 The Vault713 Developers
// Modifications Copyright 2019 The Gotts Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::*;

use futures::{
	sync::mpsc::{unbounded, UnboundedSender},
	Future, Stream,
};

use grinrelaylib::error::Result;

use crate::broker::stomp::connection::{Credentials, HeartBeat};
use crate::broker::stomp::frame::Frame;
use crate::broker::stomp::header::{Header, HeaderName, SUBSCRIPTION};
use crate::broker::stomp::session::SessionEvent;
use crate::broker::stomp::session_builder::SessionBuilder;
use crate::broker::stomp::subscription::AckMode;
use crate::broker::{BrokerRequest, BrokerResponse};

type Session = crate::broker::stomp::session::Session<TcpStream>;

const DEFAULT_QUEUE_EXPIRATION: &str = "86400000";
const DEFAULT_MESSAGE_EXPIRATION: u32 = 86400;
const REPLY_TO_HEADER_NAME: &str = "grinrelay-reply-to";

pub struct Broker {
	address: SocketAddr,
	username: String,
	password: String,
	consumers: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Broker {
	pub fn new(
		address: SocketAddr,
		username: String,
		password: String,
		consumers: Arc<Mutex<HashMap<String, Vec<String>>>>,
	) -> Broker {
		Broker {
			address,
			username,
			password,
			consumers,
		}
	}

	pub fn start(&mut self) -> Result<UnboundedSender<BrokerRequest>> {
		let (tx, rx) = unbounded();
		let address = self.address.clone();
		let username = self.username.clone();
		let password = self.password.clone();
		let consumers = self.consumers.clone();
		std::thread::spawn(move || {
			let tcp_stream = Box::new(TcpStream::connect(&address));

			let session = SessionBuilder::new()
				.with(Credentials(&username, &password))
				.with(HeartBeat(10000, 10000))
				.build(tcp_stream);

			let session = BrokerSession {
				session: Arc::new(Mutex::new(session)),
				session_number: 0,
				consumers: Arc::new(Mutex::new(HashMap::new())),
				subject_to_consumer_id_lookup: Arc::new(Mutex::new(HashMap::new())),
				subscription_id_to_consumer_id_lookup: Arc::new(Mutex::new(HashMap::new())),
				consumer_shortname_to_subject_loopup: consumers,
			};

			let mut session_clone = session.clone();

			let request_loop = rx
				.for_each(move |request| {
					match request {
						BrokerRequest::Subscribe {
							id,
							subject,
							response_sender,
						} => {
							session_clone.subscribe(id, subject.clone(), response_sender.clone());
						}
						BrokerRequest::Unsubscribe { id } => {
							session_clone.unsubscribe(&id);
						}
						BrokerRequest::PostMessage {
							subject,
							payload,
							reply_to,
							message_expiration_in_seconds,
						} => {
							session_clone.publish(
								&subject,
								&payload,
								&reply_to,
								message_expiration_in_seconds,
							);
						}
					}
					Ok(())
				})
				.map_err(|()| std::io::Error::new(std::io::ErrorKind::Other, ""));

			let f = session.select(request_loop).map_err(|_| {}).map(|_| {});

			tokio::run(f);

			error!("broker thread ending!");

			// TODO: attempt reconnection and re-establishment of subscriptions?
			std::process::exit(1);
		});

		Ok(tx)
	}
}

struct Consumer {
	subject: String,
	subscription_id: String,
	sender: UnboundedSender<BrokerResponse>,
}

impl Consumer {
	pub fn new(
		subject: String,
		subscription_id: String,
		sender: UnboundedSender<BrokerResponse>,
	) -> Consumer {
		Consumer {
			subject,
			subscription_id,
			sender,
		}
	}
}

#[derive(Clone)]
struct BrokerSession {
	session: Arc<Mutex<Session>>,
	session_number: u32,
	consumers: Arc<Mutex<HashMap<String, Consumer>>>,
	subject_to_consumer_id_lookup: Arc<Mutex<HashMap<String, String>>>,
	subscription_id_to_consumer_id_lookup: Arc<Mutex<HashMap<String, String>>>,
	consumer_shortname_to_subject_loopup: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl BrokerSession {
	fn on_connected(&mut self) {
		info!("established broker session");
	}

	fn subscribe(&mut self, id: String, subject: String, sender: UnboundedSender<BrokerResponse>) {
		self.unsubscribe_by_subject(&subject);

		let subscription_id = self
			.session
			.lock()
			.subscription(&subject)
			.with(AckMode::Auto)
			.with(Header::new(
				HeaderName::from_str("x-expires"),
				DEFAULT_QUEUE_EXPIRATION,
			))
			.start();

		let consumer = Consumer::new(subject.clone(), subscription_id.clone(), sender);
		self.subject_to_consumer_id_lookup
			.lock()
			.insert(subject, id.clone());
		self.subscription_id_to_consumer_id_lookup
			.lock()
			.insert(subscription_id, id.clone());
		self.consumers.lock().insert(id, consumer);
	}

	fn unsubscribe_by_subject(&mut self, subject: &str) {
		if let Some(consumer_id) = self.subject_to_consumer_id_lookup.lock().remove(subject) {
			if let Some(consumer) = self.consumers.lock().remove(&consumer_id) {
				self.subscription_id_to_consumer_id_lookup
					.lock()
					.remove(&consumer.subscription_id);
				self.session.lock().unsubscribe(&consumer.subscription_id);
			} else {
				error!("could not find consumer for subject [{}]", subject);
			}
		}
	}

	fn unsubscribe(&mut self, id: &str) {
		if let Some(consumer) = self.consumers.lock().remove(id) {
			if let Some(_) = self
				.subject_to_consumer_id_lookup
				.lock()
				.remove(&consumer.subject)
			{
				self.subscription_id_to_consumer_id_lookup
					.lock()
					.remove(&consumer.subscription_id);
				self.session.lock().unsubscribe(&consumer.subscription_id);
			} else {
				error!("could not find consumer for id [{}]", id);
			}
		}
	}

	fn publish(
		&self,
		subject: &str,
		payload: &str,
		reply_to: &str,
		message_expiration_in_seconds: Option<u32>,
	) {
		let destination = format!("/queue/{}", subject);;

		let message_expiration = match message_expiration_in_seconds {
			Some(message_expiration_in_seconds @ 1...86400) => {
				format!("{}", message_expiration_in_seconds * 1000)
			}
			_ => format!("{}", DEFAULT_MESSAGE_EXPIRATION * 1000),
		};

		self.session
			.lock()
			.message(&destination, payload)
			.with(Header::new(
				HeaderName::from_str("x-expires"),
				DEFAULT_QUEUE_EXPIRATION,
			))
			.with(Header::new(
				HeaderName::from_str("expiration"),
				&message_expiration,
			))
			.with(Header::new(
				HeaderName::from_str(REPLY_TO_HEADER_NAME),
				reply_to,
			))
			.send();
	}

	fn on_message(&mut self, frame: Frame) {
		if let Some(subscription_id) = frame.headers.get(SUBSCRIPTION) {
			match self
				.subscription_id_to_consumer_id_lookup
				.lock()
				.get(subscription_id)
			{
				Some(consumer_id) => match self.consumers.lock().get(consumer_id) {
					Some(consumer) => {
						if let Some(reply_to) = frame
							.headers
							.get(HeaderName::from_str(REPLY_TO_HEADER_NAME))
						{
							let payload = std::str::from_utf8(&frame.body).unwrap();
							let response = BrokerResponse::Message {
								subject: consumer.subject.clone(),
								payload: payload.to_string(),
								reply_to: reply_to.to_string(),
							};
							if consumer.sender.unbounded_send(response).is_err() {
								error!("failed sending broker message to channel!");
							};
						} else {
							error!("reply_to header missing on message!");
						}
					}
					None => {
						error!("missing consumer for message frame [{}]", subscription_id);
					}
				},
				None => {
					error!("missing consumer for message frame [{}]", subscription_id);
				}
			}
		}
	}
}

impl Future for BrokerSession {
	type Item = ();
	type Error = std::io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let msg = match try_ready!(self.session.lock().poll()) {
			None => {
				return Ok(Async::Ready(()));
			}
			Some(msg) => msg,
		};

		trace!("msg: {:?}", msg);
		match msg {
			SessionEvent::Connected => {
				self.on_connected();
			}

			SessionEvent::Message {
				destination: _destination,
				ack_mode: _ack_mode,
				frame,
			} => self.on_message(frame),

			SessionEvent::Error(frame) => {
				error!("session error event: {}", frame);
			}

			SessionEvent::Disconnected(reason) => {
				warn!(
					"session [{}] disconnected due to [{:?}]",
					self.session_number, reason
				);
				return Ok(Async::Ready(()));
			}

			m => {
				warn!("unexepcted msg: {:?}", m);
			}
		}

		Ok(Async::NotReady)
	}
}
