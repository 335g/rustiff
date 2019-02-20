
extern crate rustiff;
extern crate failure;

use std::fs::File;
use rustiff::{
    Decoder,
    DecodeErrorKind,
    FileHeaderErrorKind,
    PhotometricInterpretation,
    Compression,
    ImageData,
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

#[test]
fn test_decode_image_no_compression() -> Result<(), failure::Error> {
    let file = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif")?;
    let mut decoder = Decoder::new(file)?;
    
    let header = decoder.header();
    match header {
        Ok(header) => {
            assert_eq!(header.width(), 6);
            assert_eq!(header.height(), 4);
            assert_eq!(header.bits_per_sample().bits(), &vec![8, 8, 8, 8]);
            assert_eq!(header.compression(), None);
            assert_eq!(header.photometric_interpretation(), PhotometricInterpretation::CMYK);
        }
        Err(_) => panic!("ImageHeader should be made"),
    }

    //let data: Vec<Vec<u8>> = vec![
    //    vec![0,0,0,0], vec![], vec![], vec![], vec![], vec![],
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //];

    //let image = decoder.image()?;
    //match image.data() {
    //    &ImageData::U8(ref x) => assert_eq!(x, &data),
    //    &ImageData::U16(_) => panic!("ImageData should be u8 data"),
    //}

    Ok(())
}

#[test]
fn test_decode_image_lzw() -> Result<(), failure::Error> {
    let file = File::open("tests/images/007_cmyk_tone_interleave_ibm_lzw.tif")?;
    let mut decoder = Decoder::new(file)?;

    let header = decoder.header();
    match header {
        Ok(header) => {
            assert_eq!(header.width(), 6);
            assert_eq!(header.height(), 4);
            assert_eq!(header.bits_per_sample().bits(), &vec![8, 8, 8, 8]);
            assert_eq!(header.compression(), Some(Compression::LZW));
            assert_eq!(header.photometric_interpretation(), PhotometricInterpretation::CMYK);
        }
        Err(_) => panic!("ImageHeader should be made"),
    }

    //let data: Vec<Vec<u8>> = vec![
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //    vec![], vec![], vec![], vec![], vec![], vec![],
    //];

    //let image = decoder.image()?;
    //match image.data() {
    //    &ImageData::U8(ref x) => assert_eq!(x, &data),
    //    &ImageData::U16(_) => panic!("ImageData should u8 data"),
    //}

    Ok(())
}


