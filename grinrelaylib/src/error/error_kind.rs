use crate::types::GrinboxError;
use crate::utils::bech32::CodingError;
use failure::Fail;

#[derive(Clone, Debug, Eq, Fail, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    #[fail(display = "\x1b[31;1merror:\x1b[0m {}", 0)]
    GenericError(String),
    #[fail(display = "\x1b[31;1merror:\x1b[0m secp error")]
    SecpError,
    #[fail(display = "\x1b[31;1merror:\x1b[0m invalid chain type!")]
    InvalidChainType,
    #[fail(display = "\x1b[31;1merror:\x1b[0m invalid key!")]
    InvalidBech32Key,
    #[fail(display = "\x1b[31;1merror:\x1b[0m could not parse number from string!")]
    NumberParsingError,
    #[fail(
        display = "\x1b[31;1merror:\x1b[0m could not parse `{}` to a grinrelay address!",
        0
    )]
    GrinboxAddressParsingError(String),
    #[fail(display = "\x1b[31;1merror:\x1b[0m unable to encrypt message")]
    Encryption,
    #[fail(display = "\x1b[31;1merror:\x1b[0m unable to decrypt message")]
    Decryption,
    #[fail(display = "\x1b[31;1merror:\x1b[0m unable to verify proof")]
    VerifyProof,
    #[fail(display = "\x1b[31;1merror:\x1b[0m grinrelay websocket terminated unexpectedly!")]
    GrinboxWebsocketAbnormalTermination,
    #[fail(display = "\x1b[31;1merror:\x1b[0m grinrelay protocol error `{}`", 0)]
    GrinboxProtocolError(GrinboxError),
    #[fail(display = "\x1b[31;1merror:\x1b[0m bech32 coding error `{}`", 0)]
    Bech32Error(CodingError),
}
