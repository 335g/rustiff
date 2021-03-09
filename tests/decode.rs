use rustiff::{tag, DecodeResult, Decoder, FileHeaderError};
use std::{error::Error, fs::File, path::Path};

fn decoder<P: AsRef<Path>>(path: P) -> DecodeResult<Decoder<File>> {
    let f = File::open(path).expect("Incorrect filepath");
    Decoder::new(f)
}

#[test]
fn decode_header_byteorder_none() {
    let f = File::open("tests/images/001_not_enough_byteorder.tif").expect("exist file");
    let err = Decoder::new(f).expect_err("It should be error.");
    let kind = err.source().expect("DecodeError must have source");

    assert!(kind.is::<FileHeaderError>());

    let downcasted = kind.downcast_ref::<FileHeaderError>().unwrap();
    assert_eq!(*downcasted, FileHeaderError::NoByteOrder);
}

#[test]
fn decode_header_byteorder_incorrect() {
    let f = File::open("tests/images/002_incorrect_byteoder.tif").expect("exist file");
    let err = Decoder::new(f).expect_err("It should be error.");
    let kind = err.source().expect("DecodeError must have source");

    assert!(kind.is::<FileHeaderError>());

    let downcasted = kind.downcast_ref::<FileHeaderError>().unwrap();
    match downcasted {
        FileHeaderError::InvalidByteOrder { byte_order: _ } => {}
        _ => assert!(false),
    }
}

#[test]
fn decode_header_version_none() {
    let f = File::open("tests/images/003_not_enough_version_number.tif").expect("exist file");
    let err = Decoder::new(f).expect_err("It should be error.");
    let kind = err.source().expect("DecodeError must have source");

    assert!(kind.is::<FileHeaderError>());

    let downcasted = kind.downcast_ref::<FileHeaderError>().unwrap();
    assert_eq!(*downcasted, FileHeaderError::NoVersion);
}

#[test]
fn decode_header_version_incorrect() {
    let f = File::open("tests/images/004_incorrect_version_number.tif").expect("exist file");
    let err = Decoder::new(f).expect_err("It should be error.");
    let kind = err.source().expect("DecodeError must have source");

    assert!(kind.is::<FileHeaderError>());

    let downcasted = kind.downcast_ref::<FileHeaderError>().unwrap();
    match downcasted {
        FileHeaderError::InvalidVersion { version: _ } => {}
        _ => assert!(false),
    }
}

#[test]
fn decode_image_no_compression() {
    let f = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif")
        .expect("exist file");
    let mut decoder = Decoder::new(f).expect("No problem as tiff format");

    let width = decoder
        .get_exist_value::<tag::ImageWidth>()
        .map(|x| x.as_long())
        .unwrap_or_default();

    assert_eq!(width, 6);
}

// #[test]
// fn test_decode_image_no_compression() -> Result<(), failure::Error> {
//     let file = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif")?;
//     let mut decoder = Decoder::new(file)?;

//     let header = decoder.header();
//     match header {
//         Ok(header) => {
//             assert_eq!(header.width(), 6);
//             assert_eq!(header.height(), 4);
//             assert_eq!(header.bits_per_sample().bits(), &vec![8, 8, 8, 8]);
//             assert_eq!(header.compression(), None);
//             assert_eq!(
//                 header.photometric_interpretation(),
//                 PhotometricInterpretation::CMYK
//             );
//         }
//         Err(_) => panic!("ImageHeader should be made"),
//     }

//     //let data: Vec<Vec<u8>> = vec![
//     //    vec![0,0,0,0], vec![], vec![], vec![], vec![], vec![],
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //];

//     //let image = decoder.image()?;
//     //match image.data() {
//     //    &ImageData::U8(ref x) => assert_eq!(x, &data),
//     //    &ImageData::U16(_) => panic!("ImageData should be u8 data"),
//     //}

//     Ok(())
// }

// #[test]
// fn test_decode_image_lzw() -> Result<(), failure::Error> {
//     let file = File::open("tests/images/007_cmyk_tone_interleave_ibm_lzw.tif")?;
//     let mut decoder = Decoder::new(file)?;

//     let header = decoder.header();
//     match header {
//         Ok(header) => {
//             assert_eq!(header.width(), 6);
//             assert_eq!(header.height(), 4);
//             assert_eq!(header.bits_per_sample().bits(), &vec![8, 8, 8, 8]);
//             assert_eq!(header.compression(), Some(Compression::LZW));
//             assert_eq!(
//                 header.photometric_interpretation(),
//                 PhotometricInterpretation::CMYK
//             );
//         }
//         Err(_) => panic!("ImageHeader should be made"),
//     }

//     //let data: Vec<Vec<u8>> = vec![
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //    vec![], vec![], vec![], vec![], vec![], vec![],
//     //];

//     //let image = decoder.image()?;
//     //match image.data() {
//     //    &ImageData::U8(ref x) => assert_eq!(x, &data),
//     //    &ImageData::U16(_) => panic!("ImageData should u8 data"),
//     //}

//     Ok(())
// }
