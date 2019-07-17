use sha2::{Digest, Sha256};

use super::bech32::Bech32;
use super::secp::{Commitment, Message, PublicKey, Secp256k1, SecretKey, Signature};
use super::{from_hex, to_hex};
use crate::error::{ErrorKind, Result};

pub trait Hex<T> {
    fn from_hex(str: &str) -> Result<T>;
    fn to_hex(&self) -> String;
}

pub trait AddrBech32<T> {
    fn from_bech32(bech32_str: &str) -> Result<T>;

    fn from_bech32_check(bech32_str: &str, hrp_bytes: Vec<u8>) -> Result<T>;
    fn from_bech32_check_raw(bech32_str: &str) -> Result<(T, Vec<u8>)>;

    fn to_bech32(&self, hrp_bytes: Vec<u8>) -> String;
}

fn serialize_public_key(public_key: &PublicKey) -> Vec<u8> {
    let secp = Secp256k1::new();
    let ser = public_key.serialize_vec(&secp, true);
    ser[..].to_vec()
}

impl Hex<PublicKey> for PublicKey {
    fn from_hex(str: &str) -> Result<PublicKey> {
        let secp = Secp256k1::new();
        let hex = from_hex(str.to_string())?;
        PublicKey::from_slice(&secp, &hex).map_err(|_| ErrorKind::InvalidBech32Key.into())
    }

    fn to_hex(&self) -> String {
        to_hex(serialize_public_key(self))
    }
}

impl AddrBech32<PublicKey> for PublicKey {
    fn from_bech32(bech32_str: &str) -> Result<PublicKey> {
        let secp = Secp256k1::new();
        let addr = Bech32::from_string(bech32_str);
        if let Err(e) = addr {
            return Err(ErrorKind::Bech32Error(e).into());
        }
        PublicKey::from_slice(&secp, &addr.unwrap().data)
            .map_err(|_| ErrorKind::InvalidBech32Key.into())
    }

    fn from_bech32_check(bech32_str: &str, version_expect: Vec<u8>) -> Result<PublicKey> {
        let secp = Secp256k1::new();
        let addr = Bech32::from_string(bech32_str)?;
        if addr.hrp.into_bytes() != version_expect {
            return Err(ErrorKind::InvalidChainType.into());
        }
        PublicKey::from_slice(&secp, &addr.data).map_err(|_| ErrorKind::InvalidBech32Key.into())
    }

    fn from_bech32_check_raw(bech32_str: &str) -> Result<(PublicKey, Vec<u8>)> {
        let secp = Secp256k1::new();
        let addr = Bech32::from_string(bech32_str)?;
        let pub_key = PublicKey::from_slice(&secp, &addr.data);
        if let Err(_) = pub_key {
            return Err(ErrorKind::InvalidBech32Key.into());
        }
        Ok((pub_key.unwrap(), addr.hrp.into_bytes()))
    }

    fn to_bech32(&self, hrp_bytes: Vec<u8>) -> String {
        let b = Bech32 {
            hrp: String::from_utf8_lossy(&hrp_bytes).into_owned(),
            data: serialize_public_key(self),
        };
        b.to_string(true).unwrap()
    }
}

impl Hex<Signature> for Signature {
    fn from_hex(str: &str) -> Result<Signature> {
        let secp = Secp256k1::new();
        let hex = from_hex(str.to_string())?;
        Signature::from_der(&secp, &hex).map_err(|_| ErrorKind::SecpError.into())
    }

    fn to_hex(&self) -> String {
        let secp = Secp256k1::new();
        let signature = self.serialize_der(&secp);
        to_hex(signature)
    }
}

impl Hex<SecretKey> for SecretKey {
    fn from_hex(str: &str) -> Result<SecretKey> {
        let secp = Secp256k1::new();
        let data = from_hex(str.to_string())?;
        SecretKey::from_slice(&secp, &data).map_err(|_| ErrorKind::SecpError.into())
    }

    fn to_hex(&self) -> String {
        to_hex(self.0.to_vec())
    }
}

impl Hex<Commitment> for Commitment {
    fn from_hex(str: &str) -> Result<Commitment> {
        let data = from_hex(str.to_string())?;
        Ok(Commitment::from_vec(data))
    }

    fn to_hex(&self) -> String {
        to_hex(self.0.to_vec())
    }
}

pub fn public_key_from_secret_key(secret_key: &SecretKey) -> Result<PublicKey> {
    let secp = Secp256k1::new();
    PublicKey::from_secret_key(&secp, secret_key).map_err(|_| ErrorKind::SecpError.into())
}

pub fn sign_challenge(challenge: &str, secret_key: &SecretKey) -> Result<Signature> {
    let mut hasher = Sha256::new();
    hasher.input(challenge.as_bytes());
    let message = Message::from_slice(hasher.result().as_slice())?;
    let secp = Secp256k1::new();
    secp.sign(&message, secret_key)
        .map_err(|_| ErrorKind::SecpError.into())
}

pub fn verify_signature(
    challenge: &str,
    signature: &Signature,
    public_key: &PublicKey,
) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.input(challenge.as_bytes());
    let message = Message::from_slice(hasher.result().as_slice())?;
    let secp = Secp256k1::new();
    secp.verify(&message, signature, public_key)
        .map_err(|_| ErrorKind::SecpError.into())
}
