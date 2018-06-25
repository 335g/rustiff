
extern crate byteorder;
extern crate lzw;
#[macro_use] extern crate failure;

mod error;
mod byte;
mod decode;
mod ifd;
mod image;

pub use decode::{
    Decoder,
};

pub use ifd::{
    IFD,
    Tag,
    Entry,
};
