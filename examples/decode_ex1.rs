use std::fs::File;

use rustiff::Decoder;

fn main() {
    // let f = File::open("tests/images/007_cmyk_tone_interleave_ibm_lzw.tif")
    // .expect("exist file");
    let f = File::open("a.tif").unwrap();
    let mut decoder = Decoder::new(f).expect("No probrem as tiff format");
    println!("{:?}", decoder.predictor());

    // let img = decoder.image();
    // println!("img: {:?}", img);
}
