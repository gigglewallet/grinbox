// Copyright (c) 2017 Clark Moody
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Encode and decode the Bech32 format, with checksums
//!
//! # Examples
//! ```rust
//! use bech32::Bech32;
//!
//! let addr = Bech32 {
//!            hrp: "gn".to_string(),
//!            data: vec![
//!                0x11, 0x6a, 0x20, 0x76, 0x58, 0x22, 0x1a, 0xa9, 0xa0, 0x4b, 0xe2,
//!                0xfa, 0x6e, 0xbf, 0x28, 0x51, 0xdd, 0xf0, 0x36, 0x8d, 0x7a, 0x74,
//!                0x94, 0x34, 0xc9, 0xd1, 0x2c, 0x8f, 0xc8, 0x2a, 0xa8, 0x11, 0xf8,
//!            ]
//!        };
//!
//! let bech32_addr = addr.to_string(false).unwrap();
//! assert_eq!(bech32_addr, "gn1z94zqajcygd2ngztutaxa0eg28wlqd5d0f6fgdxf6ykgljp24qglsmstrr5".to_string());
//!
//! ```

use failure::Fail;
use hex;
use std::fmt;

/// Error types for Bech32 encoding / decoding
#[derive(Clone, Debug, Eq, Fail, PartialEq, Serialize, Deserialize)]
pub enum CodingError {
    /// String does not contain the separator character
    #[fail(display = "Missing Separator Error")]
    MissingSeparator,
    /// The checksum does not match the rest of the data
    #[fail(display = "Invalid Checksum Error")]
    InvalidChecksum,
    /// The data or human-readable part is too long or too short
    #[fail(display = "Invalid Length Error")]
    InvalidLength,
    /// Some part of the string contains an invalid character
    #[fail(display = "Invalid Char Error")]
    InvalidChar,
    /// Some part of the data has an invalid value
    #[fail(display = "Invalid Data Error")]
    InvalidData,
    /// The whole string must be of one case
    #[fail(display = "Mised Case Error")]
    MixedCase,
    /// Some AddressError
    #[fail(display = "\x1b[31;1merror:\x1b[0m address error `{}`", 0)]
    Address(AddressError),
}

/// Error types while encoding and decoding Bech32 addresses
#[derive(Clone, Debug, Eq, Fail, PartialEq, Serialize, Deserialize)]
pub enum AddressError {
    /// Some 5-bit <-> 8-bit conversion error
    #[fail(display = "Bit Conversion Error")]
    Conversion(BitConversionError),
    /// The provided human-readable portion does not match
    #[fail(display = "Human Readable Mismatch Error")]
    HumanReadableMismatch,
    /// The human-readable part is invalid (must be "gn" or "tn")
    #[fail(display = "Invalid Human Readable Part Error")]
    InvalidHumanReadablePart,
}

/// Grouping structure for the human-readable part and the data part
/// of decoded Bech32 string.
#[derive(Clone, Debug, Eq, Fail, PartialEq, Serialize, Deserialize)]
pub struct Bech32 {
    /// Human-readable part
    pub hrp: String,
    /// Data payload
    pub data: Vec<u8>,
}

impl fmt::Display for Bech32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bech32{{hrp: {}, data: {}}}",
            self.hrp,
            hex::encode(&self.data)
        )
    }
}

// Human-readable part and data part separator
const SEP: char = '1';

// Encoding character set. Maps data value -> char
const CHARSET: [char; 32] = [
    'q','p','z','r','y','9','x','8',
    'g','f','2','t','v','d','w','0',
    's','3','j','n','5','4','k','h',
    'c','e','6','m','u','a','7','l'
];

// Reverse character set. Maps ASCII byte -> CHARSET index on [0,31]
const CHARSET_REV: [i8; 128] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    15, -1, 10, 17, 21, 20, 26, 30,  7,  5, -1, -1, -1, -1, -1, -1,
    -1, 29, -1, 24, 13, 25,  9,  8, 23, -1, 18, 22, 31, 27, 19, -1,
     1,  0,  3, 16, 11, 28, 12, 14,  6,  4,  2, -1, -1, -1, -1, -1,
    -1, 29, -1, 24, 13, 25,  9,  8, 23, -1, 18, 22, 31, 27, 19, -1,
     1,  0,  3, 16, 11, 28, 12, 14,  6,  4,  2, -1, -1, -1, -1, -1
];

type EncodeResult = Result<String, CodingError>;
type DecodeResult = Result<Bech32, CodingError>;

impl Bech32 {
    /// Encode as a string
    pub fn to_string(&self, split: bool) -> EncodeResult {
        if self.hrp.len() < 1 {
            return Err(CodingError::InvalidLength);
        }
        let hrp_bytes: Vec<u8> = self.hrp.clone().into_bytes();

        // Convert 8-bit data into 5-bit
        let mut combined: Vec<u8> = match convert_bits(self.data.to_vec(), 8, 5, true) {
            Ok(p) => p,
            Err(e) => return Err(CodingError::Address(AddressError::Conversion(e))),
        };

        combined.extend_from_slice(&create_checksum(&hrp_bytes, &combined));
        let mut encoded: String = String::with_capacity(128);
        encoded.push_str(format!("{}{}", self.hrp, SEP).as_str());
        let start_pos = encoded.len();
        for p in combined {
            if p >= 32 {
                return Err(CodingError::InvalidData);
            }
            encoded.push(CHARSET[p as usize]);
        }

        if split {
            if encoded.len() > start_pos + 16 {
                encoded.insert_str(start_pos + 8, "-");
                encoded.insert_str(start_pos, "-");
            }

            if encoded.len() > start_pos + 16 {
                encoded.insert_str(encoded.len() - 6, "-");
            }
        }

        Ok(encoded)
    }

    /// Decode from a string
    pub fn from_string(bech32_addr: &str) -> DecodeResult {
        let mut s: String = bech32_addr.to_owned();
        s.retain(|c| c != '-');

        // Ensure overall length is within bounds
        let len: usize = s.len();
        if len < 8 || len > 90 {
            return Err(CodingError::InvalidLength);
        }

        // Check for missing separator
        if s.find(SEP).is_none() {
            return Err(CodingError::MissingSeparator);
        }

        // Split at separator and check for two pieces
        let parts: Vec<&str> = s.rsplitn(2, SEP).collect();
        let raw_hrp = parts[1];
        let raw_data = parts[0];
        if raw_hrp.len() < 1 || raw_data.len() < 6 {
            return Err(CodingError::InvalidLength);
        }

        let mut has_lower: bool = false;
        let mut has_upper: bool = false;
        let mut hrp_bytes: Vec<u8> = Vec::new();
        for b in raw_hrp.bytes() {
            // Valid subset of ASCII
            if b < 33 || b > 126 {
                return Err(CodingError::InvalidChar);
            }
            let mut c = b;
            // Lowercase
            if b >= b'a' && b <= b'z' {
                has_lower = true;
            }
            // Uppercase
            if b >= b'A' && b <= b'Z' {
                has_upper = true;
                // Convert to lowercase
                c = b + (b'a' - b'A');
            }
            hrp_bytes.push(c);
        }

        // Check data payload
        let mut data_bytes: Vec<u8> = Vec::new();
        for b in raw_data.bytes() {
            // Aphanumeric only
            if !((b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'Z') || (b >= b'a' && b <= b'z')) {
                return Err(CodingError::InvalidChar);
            }
            // Excludes these characters: [1,b,i,o]
            if b == b'1' || b == b'b' || b == b'i' || b == b'o' {
                return Err(CodingError::InvalidChar);
            }
            // Lowercase
            if b >= b'a' && b <= b'z' {
                has_lower = true;
            }
            let mut c = b;
            // Uppercase
            if b >= b'A' && b <= b'Z' {
                has_upper = true;
                // Convert to lowercase
                c = b + (b'a' - b'A');
            }
            data_bytes.push(CHARSET_REV[c as usize] as u8);
        }

        // Ensure no mixed case
        if has_lower && has_upper {
            return Err(CodingError::MixedCase);
        }

        // Ensure checksum
        if !verify_checksum(&hrp_bytes, &data_bytes) {
            return Err(CodingError::InvalidChecksum);
        }

        // Remove checksum from data payload
        let dbl: usize = data_bytes.len();
        data_bytes.truncate(dbl - 6);

        Ok(Bech32 {
            hrp: String::from_utf8(hrp_bytes).unwrap(),
            // Convert to 8-bit program and assign
            data: match convert_bits(data_bytes, 5, 8, false) {
                Ok(p) => p,
                Err(e) => return Err(CodingError::Address(AddressError::Conversion(e))),
            },
        })
    }
}

fn create_checksum(hrp: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    let mut values: Vec<u8> = hrp_expand(hrp);
    values.extend_from_slice(data);
    // Pad with 6 zeros
    values.extend_from_slice(&[0u8; 6]);
    let plm: u32 = polymod(values) ^ 1;
    let mut checksum: Vec<u8> = Vec::new();
    for p in 0..6 {
        checksum.push(((plm >> 5 * (5 - p)) & 0x1f) as u8);
    }
    checksum
}

fn verify_checksum(hrp: &Vec<u8>, data: &Vec<u8>) -> bool {
    let mut exp = hrp_expand(hrp);
    exp.extend_from_slice(data);
    polymod(exp) == 1u32
}

fn hrp_expand(hrp: &Vec<u8>) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    for b in hrp {
        v.push(*b >> 5);
    }
    v.push(0);
    for b in hrp {
        v.push(*b & 0x1f);
    }
    v
}

// Generator coefficients
const GEN: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

fn polymod(values: Vec<u8>) -> u32 {
    let mut chk: u32 = 1;
    let mut b: u8;
    for v in values {
        b = (chk >> 25) as u8;
        chk = (chk & 0x1ffffff) << 5 ^ (v as u32);
        for i in 0..5 {
            if (b >> i) & 1 == 1 {
                chk ^= GEN[i]
            }
        }
    }
    chk
}

/// Error types during bit conversion
#[derive(Clone, Debug, Eq, Fail, PartialEq, Serialize, Deserialize)]
pub enum BitConversionError {
    /// Input value exceeds "from bits" size
    #[fail(display = "Invalid Input Error")]
    InvalidInputValue(u8),
    /// Invalid padding values in data
    #[fail(display = "Invalid Padding Error")]
    InvalidPadding,
}

type ConvertResult = Result<Vec<u8>, BitConversionError>;

/// Convert between bit sizes
///
/// # Panics
/// Function will panic if attempting to convert `from` or `to` a bit size that
/// is larger than 8 bits.
fn convert_bits(data: Vec<u8>, from: u32, to: u32, pad: bool) -> ConvertResult {
    if from > 8 || to > 8 {
        panic!("convert_bits `from` and `to` parameters greater than 8");
    }
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret: Vec<u8> = Vec::new();
    let maxv: u32 = (1 << to) - 1;
    for value in data {
        let v: u32 = value as u32;
        if (v >> from) != 0 {
            // Input value exceeds `from` bit size
            return Err(BitConversionError::InvalidInputValue(v as u8));
        }
        acc = (acc << from) | v;
        bits += from;
        while bits >= to {
            bits -= to;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }
    if pad {
        if bits > 0 {
            ret.push(((acc << (to - bits)) & maxv) as u8);
        }
    } else if bits >= from || ((acc << (to - bits)) & maxv) != 0 {
        return Err(BitConversionError::InvalidPadding);
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bech32_demo() {
        // with 5 bytes data

        let b = Bech32 {
            hrp: "bech32".to_string(),
            data: vec![0x00, 0x44, 0x32, 0x14, 0xc7],
        };
        let encode = b.to_string(false).unwrap();
        assert_eq!(encode, "bech321qpzry9x8vrqdnl".to_string());

        let decode = Bech32::from_string(encode.as_str()).unwrap();
        assert_eq!(decode, b);

        // with 33 bytes data (a typical Public Key address)

        let a1 = Bech32 {
            hrp: "gn".to_string(),
            data: vec![
                0x11, 0x6a, 0x20, 0x76, 0x58, 0x22, 0x1a, 0xa9, 0xa0, 0x4b, 0xe2,
                0xfa, 0x6e, 0xbf, 0x28, 0x51, 0xdd, 0xf0, 0x36, 0x8d, 0x7a, 0x74,
                0x94, 0x34, 0xc9, 0xd1, 0x2c, 0x8f, 0xc8, 0x2a, 0xa8, 0x11, 0xf8,
            ],
        };
        let encode = a1.to_string(false).unwrap();
        assert_eq!(
            encode,
            "gn1z94zqajcygd2ngztutaxa0eg28wlqd5d0f6fgdxf6ykgljp24qglsmstrr5".to_string()
        );

        println!(
            "data size: {} bytes, bech32 address size: {} bytes",
            a1.data.len(),
            encode.len()
        );

        let decode = Bech32::from_string(encode.as_str()).unwrap();
        assert_eq!(decode, a1);

        // decode with some errors (no more than 4 errors can be detected definitely)

        let bech32_str = "gn1z94zqajcygd2ngztutaxa0eg28wlqd5d0f6fgdxf6ykgljp24qglsms4err";
        let decode = Bech32::from_string(bech32_str);
        assert_eq!(decode, Err(CodingError::InvalidChecksum));

        // with '-' splitter

        let encode = a1.to_string(true).unwrap();
        assert_eq!(
            encode,
            "gn1-z94zqajc-ygd2ngztutaxa0eg28wlqd5d0f6fgdxf6ykgljp24qgls-mstrr5".to_string()
        );

        let decode = Bech32::from_string(encode.as_str()).unwrap();
        assert_eq!(decode, a1);
    }
}
