
extern crate rustiff;
use std::fs::File;
use rustiff::{
    Decoder,
    DecodeErrorKind,
    FileHeaderErrorKind,
};

#[test]
fn test_decode_header_byteorder_none() {
    let file = File::open("tests/images/001_not_enough_byteorder.tif").expect("incorrect file path");
    let decoder = Decoder::new(file);

    match decoder {
        Ok(_) => panic!("It should be error"),
        Err(e) => {
            match e.kind() {
                &DecodeErrorKind::IncorrectFileHeader(ref e) => assert_eq!(e, &FileHeaderErrorKind::NoByteOrder),
                _ => panic!("It should not be other error."),
            }
        }
    }
}

#[test]
fn test_decode_header_byteorder_incorrect() {
    let file = File::open("tests/images/002_incorrect_byteoder.tif").expect("incorrect file path");
    let decoder = Decoder::new(file);

    match decoder {
        Ok(_) => panic!("It should be error."),
        Err(e) => {
            match e.kind() {
                &DecodeErrorKind::IncorrectFileHeader(FileHeaderErrorKind::IncorrectByteOrder{ byte_order }) => assert_ne!(byte_order, [0x49, 0x49]),
                _ => panic!("It should not be other error."),
            }
        }
    }
}

#[test]
fn test_decode_header_version_none() {
    let file = File::open("tests/images/003_not_enough_version_number.tif").expect("incorrect file path");
    let decoder = Decoder::new(file);

    match decoder {
        Ok(_) => panic!("It should be error."),
        Err(e) => {
            match e.kind() {
                &DecodeErrorKind::IncorrectFileHeader(ref e) => assert_eq!(e, &FileHeaderErrorKind::NoVersion),
                _ => panic!("It should not be other error."),
            }
        }
    }
}

#[test]
fn test_decode_header_version_incorrect() {
    let file = File::open("tests/images/004_incorrect_version_number.tif").expect("incorrect file path");
    let decoder = Decoder::new(file);

    match decoder {
        Ok(_) => panic!("It should be error."),
        Err(e) => {
            match e.kind() {
                &DecodeErrorKind::IncorrectFileHeader(FileHeaderErrorKind::IncorrectVersion{ version }) => assert_ne!(version, 42),
                _ => panic!("It should not be other error."),
            }
        }
    }
}

