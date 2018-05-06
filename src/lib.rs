
extern crate byteorder;
extern crate lzw;
extern crate num;

#[macro_use] extern crate failure;

mod error;
mod byte;
mod decode;
mod ifd;

pub use decode::{
    Decoder,
    IFDs,
};

pub use ifd::{
    IFD,
    Tag,
    Entry,
};
