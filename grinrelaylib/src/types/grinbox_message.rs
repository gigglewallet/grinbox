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

use ring::{aead, digest, pbkdf2};

use crate::error::{ErrorKind, Result};
use crate::types::GrinboxAddress;
use crate::utils::from_hex;
use crate::utils::secp::{PublicKey, Secp256k1, SecretKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct GrinboxMessage {
	#[serde(default)]
	pub destination: Option<GrinboxAddress>,
	encrypted_message: String,
	salt: String,
	nonce: String,
}

impl GrinboxMessage {
	pub fn key(&self, sender_public_key: &PublicKey, secret_key: &SecretKey) -> Result<[u8; 32]> {
		let salt = from_hex(self.salt.clone()).map_err(|_| ErrorKind::Decryption)?;

		let secp = Secp256k1::new();
		let mut common_secret = sender_public_key.clone();
		common_secret
			.mul_assign(&secp, secret_key)
			.map_err(|_| ErrorKind::Decryption)?;
		let common_secret_ser = common_secret.serialize_vec(&secp, true);
		let common_secret_slice = &common_secret_ser[1..33];

		let mut key = [0; 32];
		pbkdf2::derive(&digest::SHA512, 10000, &salt, common_secret_slice, &mut key);

		Ok(key)
	}

	pub fn decrypt_with_key(&self, key: &[u8; 32]) -> Result<String> {
		let mut encrypted_message =
			from_hex(self.encrypted_message.clone()).map_err(|_| ErrorKind::Decryption)?;
		let nonce = from_hex(self.nonce.clone()).map_err(|_| ErrorKind::Decryption)?;

		let opening_key = aead::OpeningKey::new(&aead::CHACHA20_POLY1305, key)
			.map_err(|_| ErrorKind::Decryption)?;
		let decrypted_data =
			aead::open_in_place(&opening_key, &nonce, &[], 0, &mut encrypted_message)
				.map_err(|_| ErrorKind::Decryption)?;

		String::from_utf8(decrypted_data.to_vec()).map_err(|_| ErrorKind::Decryption.into())
	}
}
