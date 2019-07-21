use crate::error::{ErrorKind, Result};
use rustc_serialize::hex::{FromHex, ToHex};

pub mod bech32;
pub mod crypto;
pub mod secp;

/// Encode the provided bytes into a hex string
pub fn to_hex(bytes: Vec<u8>) -> String {
	bytes.to_hex()
}

/// Decode a hex string into bytes (no '0x' prefix).
pub fn from_hex(hex_str: String) -> Result<Vec<u8>> {
	hex_str
		.from_hex()
		.map_err(|_| ErrorKind::NumberParsingError.into())
}
