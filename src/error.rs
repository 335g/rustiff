
// tmp
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use failure::Error;
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum DecodeError {
    #[fail(display = "Incorrect header : ({:?})", reason)]
    IncorrectHeader { reason: String, },

    #[fail(display = "Cannot decode byte code to IFD.")]
    NoIFD,

    #[fail(display = "Cannot decode byte code to IFD entry.")]
    NoIFDEntry
}

pub enum EncodeError {}
