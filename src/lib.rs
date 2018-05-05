
extern crate byteorder;
extern crate lzw;

#[macro_use] extern crate failure;

mod error;

pub mod prelude {
    pub use error::{
        EncodeError,
        DecodeError,
    };
}
