use colored::*;
use failure::Fail;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, Fail, PartialEq, Serialize, Deserialize, Debug)]
pub enum GrinboxError {
	#[fail(display = "GrinboxError: unknown error")]
	UnknownError,
	#[fail(display = "GrinboxError: invalid request")]
	InvalidRequest,
	#[fail(display = "GrinboxError: invalid signature")]
	InvalidSignature,
	#[fail(display = "GrinboxError: invalid challenge")]
	InvalidChallenge,
	#[fail(display = "GrinboxError: too many subscriptions")]
	TooManySubscriptions,
	#[fail(display = "GrinboxError: invalid abbreviation relay address")]
	InvalidRelayAbbr,
	#[fail(display = "GrinboxError: not online")]
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
