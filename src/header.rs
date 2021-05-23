use crate::{ifd::ImageFileDirectory, val::{BitsPerSample, Compression, PhotometricInterpretation, Predictor, RowsPerStrip, StripByteCounts, StripOffsets}};

#[derive(Debug)]
pub struct Header {
    next_addr: Option<u64>,
    ifd: ImageFileDirectory,
    width: usize,
    height: usize,
    bits_per_sample: BitsPerSample,
    compression: Option<Compression>,
    photometric_interpretation: PhotometricInterpretation,
    rows_per_strip: usize,
    strip_offsets: StripOffsets,
    strip_byte_counts: StripByteCounts,
    predictor: Predictor,
}

impl Header {
    pub fn new(
        next_addr: Option<u64>,
        ifd: ImageFileDirectory,
        width: usize,
        height: usize,
        bits_per_sample: BitsPerSample,
        compression: Option<Compression>,
        photometric_interpretation: PhotometricInterpretation,
        rows_per_strip: usize,
        strip_offsets: StripOffsets,
        strip_byte_counts: StripByteCounts,
        predictor: Predictor,
    ) -> Self {
        Self {
            next_addr,
            ifd,
            width,
            height,
            bits_per_sample,
            compression,
            photometric_interpretation,
            rows_per_strip,
            strip_offsets,
            strip_byte_counts,
            predictor,
        }
    }

    pub fn ifd(&self) -> &ImageFileDirectory {
        &self.ifd
    }

    pub(crate) fn next_addr(&self) -> Option<u64> {
        self.next_addr.clone()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn bits_per_sample(&self) -> &BitsPerSample {
        &self.bits_per_sample
    }

    pub fn compression(&self) -> Option<&Compression> {
        self.compression.as_ref()
    }

    pub fn photometric_interpretation(&self) -> &PhotometricInterpretation {
        &self.photometric_interpretation
    }

    pub fn rows_per_strip(&self) -> usize {
        self.rows_per_strip
    }

    pub fn strip_offsets(&self) -> &StripOffsets {
        &self.strip_offsets
    }

    pub fn strip_byte_counts(&self) -> &StripByteCounts {
        &self.strip_byte_counts
    }

    pub fn predictor(&self) -> &Predictor {
        &self.predictor
    }
}
