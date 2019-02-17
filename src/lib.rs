//! The `rustiff` crate provides TIFF format decoder and encoder. (TODO: impl encoder)
//! 
//! # Brief overview
//! 
//! The primary types in this crate is [`Decoder`](struct.Decoder.html) for decoding TIFF data.
//! 
//! [`IFD(Image File Directory)`](struct.IFD.html) is used when multiple images are included in one file.
//! 
//! Many tags are defined in [`tag module`](tag/index.html).
//! 
//! # Setup
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rustiff = "0.2"
//! ```
//!
//! and this to your crate root (if rust 2015):
//!
//! ```ignore
//! extern crate rustiff;
//! ```
//!
//! # Example
//!
//! This example shows how to read data conforming to TIFF format and print each pixel data
//! to stdout.
//! 
//! ```ignore
//! extern crate rustiff;
//!
//! use std::fs::File;
//! use std::process;
//! 
//! fn main() {
//! 
//! }
//!
//! 

#![warn(missing_docs)]

extern crate byteorder;
extern crate lzw;

#[macro_use]
extern crate failure;

mod error;
mod byte;
mod decode;
mod ifd;
mod image;
mod tool;

#[allow(missing_docs)]
pub mod tag;

pub use decode::Decoder;
pub use ifd::IFD;
pub use error::{
    DecodeError,
    DecodeErrorKind,
    FileHeaderErrorKind,
    DecodeResult,
};
pub use image::{
    Image,
    ImageHeader,
    ImageHeaderBuildError,
    Compression,
    BitsPerSample,
    PhotometricInterpretation,
};
