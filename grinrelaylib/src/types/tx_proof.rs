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

use crate::types::{GrinboxAddress, GrinboxMessage, Slate};
use crate::utils::crypto::{verify_signature, Hex};
use crate::utils::secp::{Commitment, SecretKey, Signature};

#[derive(Debug)]
pub enum ErrorKind {
	ParseAddress,
	ParsePublicKey,
	ParseSignature,
	VerifySignature,
	ParseGrinboxMessage,
	VerifyDestination,
	DecryptionKey,
	DecryptMessage,
	ParseSlate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxProof {
	pub address: GrinboxAddress,
	pub message: String,
	pub challenge: String,
	pub signature: Signature,
	pub key: [u8; 32],
	pub amount: u64,
	pub fee: u64,
	pub inputs: Vec<Commitment>,
	pub outputs: Vec<Commitment>,
}

impl TxProof {
	pub fn verify_extract(
		&self,
		expected_destination: Option<&GrinboxAddress>,
	) -> Result<(Option<GrinboxAddress>, Slate), ErrorKind> {
		let mut challenge = String::new();
		challenge.push_str(self.message.as_str());
		challenge.push_str(self.challenge.as_str());

		let public_key = self
			.address
			.public_key()
			.map_err(|_| ErrorKind::ParsePublicKey)?;

		verify_signature(&challenge, &self.signature, &public_key)
			.map_err(|_| ErrorKind::VerifySignature)?;

		let grinbox_message: GrinboxMessage =
			serde_json::from_str(&self.message).map_err(|_| ErrorKind::ParseGrinboxMessage)?;

		// TODO: at some point, make this check required
		let destination = grinbox_message.destination.clone();
		if destination.is_some()
			&& expected_destination.is_some()
			&& destination.as_ref() != expected_destination
		{
			return Err(ErrorKind::VerifyDestination);
		}

		let decrypted_message = grinbox_message
			.decrypt_with_key(&self.key)
			.map_err(|_| ErrorKind::DecryptMessage)?;

		serde_json::from_str(&decrypted_message)
			.map(|message| (destination, message))
			.map_err(|_| ErrorKind::ParseSlate)
	}

	pub fn from_response(
		from: String,
		message: String,
		challenge: String,
		signature: String,
		secret_key: &SecretKey,
		expected_destination: Option<&GrinboxAddress>,
	) -> Result<(Slate, TxProof), ErrorKind> {
		let address =
			GrinboxAddress::from_str(from.as_str()).map_err(|_| ErrorKind::ParseAddress)?;
		let signature =
			Signature::from_hex(signature.as_str()).map_err(|_| ErrorKind::ParseSignature)?;
		let public_key = address
			.public_key()
			.map_err(|_| ErrorKind::ParsePublicKey)?;
		let grinbox_message: GrinboxMessage =
			serde_json::from_str(&message).map_err(|_| ErrorKind::ParseGrinboxMessage)?;
		let key = grinbox_message
			.key(&public_key, secret_key)
			.map_err(|_| ErrorKind::DecryptionKey)?;

		let proof = TxProof {
			address,
			message,
			challenge,
			signature,
			key,
			amount: 0,
			fee: 0,
			inputs: vec![],
			outputs: vec![],
		};

		let (_, slate) = proof.verify_extract(expected_destination)?;

		Ok((slate, proof))
	}
}
