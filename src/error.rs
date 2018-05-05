
// tmp
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use failure::Error;

pub type Result<T> = ::std::result::Result<T, Error>;


pub enum DecodeError {}
pub enum EncodeError {}
