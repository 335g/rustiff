use std::fs::File;

use rustiff::{tag, Decoder, Data};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("tests/images/010_cmyk_2layer.tif").expect("");
    let mut decoder = Decoder::new(f).expect("");
    let ifd = decoder.ifd()?;

    let img = decoder.image(&ifd)?;
    match img {
        Data::Short(x) => println!("{:?}", x),
        Data::Byte(x) => println!("{:?}", x),
    }


    Ok(())
}
