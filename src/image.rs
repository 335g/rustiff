use crate::val::{BitsPerSample, Compression, PhotometricInterpretation};

#[derive(Debug)]
pub struct Image {
    width: usize,
    height: usize,
    bits_per_sample: BitsPerSample,
    compression: Option<Compression>,
    photometric_interpretation: PhotometricInterpretation,
}
