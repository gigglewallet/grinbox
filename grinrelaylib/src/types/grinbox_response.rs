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

use colored::*;
use failure::Fail;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, Fail, PartialEq, Serialize, Deserialize, Debug)]
pub enum GrinboxError {
	#[fail(display = "GrinRelay Protocol: unknown error")]
	UnknownError,
	#[fail(display = "GrinRelay Protocol: invalid request")]
	InvalidRequest,
	#[fail(display = "GrinRelay Protocol: invalid signature")]
	InvalidSignature,
	#[fail(display = "GrinRelay Protocol: invalid challenge")]
	InvalidChallenge,
	#[fail(display = "GrinRelay Protocol: too many subscriptions")]
	TooManySubscriptions,
	#[fail(display = "GrinRelay Protocol: invalid abbreviation relay address")]
	InvalidRelayAbbr,
	#[fail(display = "GrinRelay Protocol: not online")]
	Offline,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GrinboxResponse {
	Ok,
	Error {
		kind: GrinboxError,
		description: String,
	},
	Challenge {
		str: String,
	},
	Slate {
		from: String,
		str: String,
		signature: String,
		challenge: String,
	},
	RelayAddr {
		abbr: String,
		relay_addr: Vec<String>,
	},
}

impl Display for GrinboxResponse {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match *self {
			GrinboxResponse::Ok => write!(f, "{}", "Ok".cyan()),
			GrinboxResponse::Error {
				ref kind,
				description: _,
			} => write!(f, "{}: {}", "error".bright_red(), kind),
			GrinboxResponse::Challenge { ref str } => {
				write!(f, "{} {}", "Challenge".cyan(), str.bright_green())
			}
			GrinboxResponse::Slate {
				ref from,
				str: _,
				signature: _,
				challenge: _,
			} => write!(f, "{} from {}", "Slate".cyan(), from.bright_green()),
			GrinboxResponse::RelayAddr {
				ref abbr,
				ref relay_addr,
			} => write!(
				f,
				"{}: abbr: {}, full: {}",
				"RelayAddr".cyan(),
				abbr.bright_green(),
				relay_addr[0].bright_green()
			),
		}
	}
}
