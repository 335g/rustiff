
extern crate byteorder;
extern crate lzw;
extern crate num;

#[macro_use] extern crate failure;

mod error;
mod byte;
mod decode;

pub mod prelude {
    pub use error::{
        EncodeError,
        DecodeError,
    };

    pub use decode::Decoder;
}
