use ring::{aead, digest, pbkdf2};

use crate::error::{ErrorKind, Result};
use crate::utils::from_hex;
use crate::utils::secp::{Secp256k1, PublicKey, SecretKey};
use crate::types::GrinboxAddress;

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
