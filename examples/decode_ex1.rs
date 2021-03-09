use std::{fs::File, path::Path};

use rustiff::{tag, Data, DecodeResult, Decoder};

fn decoder<P: AsRef<Path>>(path: P) -> DecodeResult<Decoder<File>> {
    let f = File::open(path).expect("Incorrect filepath");
    Decoder::new(f)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif")?;
    let mut decoder = Decoder::new(f)?;
    let width = decoder.get_exist_value::<tag::ImageWidth>()?;
    println!("width: {:?}", width);

    Ok(())
}
