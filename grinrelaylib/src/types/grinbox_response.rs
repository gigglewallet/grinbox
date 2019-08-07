use colored::*;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum GrinboxError {
	UnknownError,
	InvalidRequest,
	InvalidSignature,
	InvalidChallenge,
	TooManySubscriptions,
	InvalidRelayAbbr,
}

impl Display for GrinboxError {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match *self {
			GrinboxError::UnknownError => write!(f, "{}", "unknown error!"),
			GrinboxError::InvalidRequest => write!(f, "{}", "invalid request!"),
			GrinboxError::InvalidSignature => write!(f, "{}", "invalid signature!"),
			GrinboxError::InvalidChallenge => write!(f, "{}", "invalid challenge!"),
			GrinboxError::TooManySubscriptions => write!(f, "{}", "too many subscriptions!"),
			GrinboxError::InvalidRelayAbbr => write!(f, "{}", "invalid abbr relay address!"),
		}
	}
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
		relay_addr: String,
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
				"{}:  abbr: {} ,  full: {}",
				"RelayAddr".cyan(),
				abbr.bright_green(),
				relay_addr.bright_green()
			),
		}
	}
}
