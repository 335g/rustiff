use std::fs::File;

use rustiff::Decoder;

fn main() {
    // let f = File::open("tests/images/007_cmyk_tone_interleave_ibm_lzw.tif")
    // .expect("exist file");
    let f = File::open("tests/images/006_cmyk_tone_interleave_ibm_uncompressed.tif").unwrap();
    let mut decoder = Decoder::new(f).expect("No probrem as tiff format");
    let ifd = decoder.ifd().clone();

    for tag in ifd.keys() {
        println!("{}", tag);
        println!("{:?}", ifd.get_tag(tag));
        println!(" ")
    }

    // let img = decoder.image();
    // println!("img: {:?}", img);
}
