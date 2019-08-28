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
use std::fmt::{Display, Formatter, Result};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GrinboxRequest {
	Challenge,
	Subscribe {
		address: String,
		signature: String,
	},
	RetrieveRelayAddr {
		abbr: String,
	},
	PostSlate {
		from: String,
		to: String,
		str: String,
		signature: String,
		message_expiration_in_seconds: Option<u32>,
	},
	Unsubscribe {
		address: String,
	},
}

impl Display for GrinboxRequest {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match *self {
			GrinboxRequest::Challenge => write!(f, "{}", "Challenge".bright_purple()),
			GrinboxRequest::Subscribe {
				ref address,
				signature: _,
			} => write!(
				f,
				"{} to {}",
				"Subscribe".bright_purple(),
				address.bright_green()
			),
			GrinboxRequest::Unsubscribe { ref address } => write!(
				f,
				"{} from {}",
				"Unsubscribe".bright_purple(),
				address.bright_green()
			),
			GrinboxRequest::PostSlate {
				ref from,
				ref to,
				str: _,
				signature: _,
				message_expiration_in_seconds: _,
			} => write!(
				f,
				"{} from {} to {}",
				"PostSlate".bright_purple(),
				from.bright_green(),
				to.bright_green()
			),
			GrinboxRequest::RetrieveRelayAddr { ref abbr } => write!(
				f,
				"{} : {}",
				"RetrieveRelayAddr".bright_purple(),
				abbr.bright_green()
			),
		}
	}
}
