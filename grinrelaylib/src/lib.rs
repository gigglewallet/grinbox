#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde;

pub mod client;
pub mod error;
pub mod types;
pub mod utils;

pub use parking_lot::{Mutex, RwLock, RwLockReadGuard};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
