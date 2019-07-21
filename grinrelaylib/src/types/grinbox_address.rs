use regex::Regex;
use std::fmt::{self, Display};

use crate::error::{ErrorKind, Result};
use crate::utils::crypto::AddrBech32;
use crate::utils::secp::PublicKey;
use parking_lot::RwLock;

pub const GRINRELAY_PREFIX: &str = "grinrelay://";
pub const GRINRELAY_ADDRESS_REGEX: &str = r"^(grinrelay://)?(?P<public_key>[0-9a-z\-]{62,67})(@(?P<domain>[a-zA-Z0-9\.]+)(:(?P<port>[0-9]*))?)?$";
pub const GRINRELAY_ADDRESS_HRP_MAINNET: &str = "gn";
pub const GRINRELAY_ADDRESS_HRP_TESTNET: &str = "tn";
pub const DEFAULT_GRINRELAY_DOMAIN: &str = "relay.grin.icu";
pub const DEFAULT_GRINRELAY_PORT: u16 = 3418;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChainTypes {
	/// Protocol testing network
	Floonet,
	/// Main production network
	Mainnet,
}

lazy_static! {
	/// The mining parameter mode
	pub static ref CHAIN_TYPE: RwLock<ChainTypes> =
			RwLock::new(ChainTypes::Mainnet);
}

pub fn is_mainnet() -> bool {
	let param_ref = CHAIN_TYPE.read();
	ChainTypes::Mainnet == *param_ref
}

pub fn set_running_mode(mode: ChainTypes) {
	let mut param_ref = CHAIN_TYPE.write();
	*param_ref = mode;
}

pub fn hrp_bytes() -> Vec<u8> {
	if is_mainnet() {
		GRINRELAY_ADDRESS_HRP_MAINNET.into()
	} else {
		GRINRELAY_ADDRESS_HRP_TESTNET.into()
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GrinboxAddress {
	pub public_key: String,
	pub domain: String,
	pub port: u16,
	pub hrp_bytes: Option<Vec<u8>>,
}

impl GrinboxAddress {
	pub fn new(public_key: PublicKey, domain: Option<String>, port: Option<u16>) -> Self {
		Self {
			public_key: public_key.to_bech32(hrp_bytes()),
			domain: domain.unwrap_or(DEFAULT_GRINRELAY_DOMAIN.to_string()),
			port: port.unwrap_or(DEFAULT_GRINRELAY_PORT),
			hrp_bytes: None,
		}
	}

	pub fn new_raw(
		public_key: PublicKey,
		domain: Option<String>,
		port: Option<u16>,
		hrp_bytes: Vec<u8>,
	) -> Self {
		Self {
			public_key: public_key.to_bech32(hrp_bytes.clone()),
			domain: domain.unwrap_or(DEFAULT_GRINRELAY_DOMAIN.to_string()),
			port: port.unwrap_or(DEFAULT_GRINRELAY_PORT),
			hrp_bytes: Some(hrp_bytes),
		}
	}

	pub fn from_str(s: &str) -> Result<Self> {
		let re = Regex::new(GRINRELAY_ADDRESS_REGEX).unwrap();
		let captures = re.captures(s);
		if captures.is_none() {
			Err(ErrorKind::GrinboxAddressParsingError(s.to_string()))?;
		}

		let captures = captures.unwrap();
		let public_key = captures.name("public_key").unwrap().as_str().to_string();
		let domain = captures.name("domain").map(|m| m.as_str().to_string());
		let port = captures
			.name("port")
			.map(|m| u16::from_str_radix(m.as_str(), 10).unwrap());

		let public_key = PublicKey::from_bech32_check(&public_key, hrp_bytes())?;

		Ok(GrinboxAddress::new(public_key, domain, port))
	}

	pub fn from_str_raw(s: &str) -> Result<Self> {
		let re = Regex::new(GRINRELAY_ADDRESS_REGEX).unwrap();
		let captures = re.captures(s);
		if captures.is_none() {
			Err(ErrorKind::GrinboxAddressParsingError(s.to_string()))?;
		}

		let captures = captures.unwrap();
		let public_key = captures.name("public_key").unwrap().as_str().to_string();
		let domain = captures.name("domain").map(|m| m.as_str().to_string());
		let port = captures
			.name("port")
			.map(|m| u16::from_str_radix(m.as_str(), 10).unwrap());

		let (public_key, hrp_bytes) = PublicKey::from_bech32_check_raw(&public_key)?;

		Ok(GrinboxAddress::new_raw(public_key, domain, port, hrp_bytes))
	}

	pub fn public_key(&self) -> Result<PublicKey> {
		PublicKey::from_bech32_check(&self.public_key, hrp_bytes())
	}

	pub fn stripped(&self) -> String {
		format!("{}", self)[GRINRELAY_PREFIX.len()..].to_string()
	}
}

impl Display for GrinboxAddress {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}{}", GRINRELAY_PREFIX, self.public_key)?;
		if !self.domain.ends_with(DEFAULT_GRINRELAY_DOMAIN) || self.port != DEFAULT_GRINRELAY_PORT {
			write!(f, "@{}", self.domain)?;
			if self.port != DEFAULT_GRINRELAY_PORT {
				write!(f, ":{}", self.port)?;
			}
		}
		Ok(())
	}
}
