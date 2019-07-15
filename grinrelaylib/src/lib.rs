extern crate colored;
extern crate failure;
extern crate log;
extern crate parking_lot;
extern crate regex;
extern crate secp256k1zkp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate ws;
#[macro_use]
extern crate lazy_static;

pub mod client;
pub mod error;
pub mod utils;
pub mod types;

pub use parking_lot::{Mutex, RwLock, RwLockReadGuard};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
