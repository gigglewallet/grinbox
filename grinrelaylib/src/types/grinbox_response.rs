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
