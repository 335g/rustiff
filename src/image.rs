
use tag::{
    self,
    TagType,
};
use tool::{
    HasValue,
    Empty,
    Filled,
};

/// 
#[derive(Debug, Fail)]
#[fail(display = "{} cannot be constructed with unsupported value({:?}), reason: {}", tag, value, reason)]
pub struct ConstructError<T: TagType> {
    tag: T,
    value: T::Value,
    reason: String
}

impl<T> ConstructError<T> where T: TagType {
    pub(crate) fn new(tag: T, value: T::Value, reason: String) -> Self {
        ConstructError {
            tag: tag,
            value: value,
            reason: reason,
        }
    }
}

/// The color space of the image data.
///
/// IFD constructs this with `tag::PhotometricInterpretation`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhotometricInterpretation {
    /// For bilievel and grayscale images.
    ///
    /// 0 is imaged as white. 2**BitsPerSample - 1 is imaged as black.
    /// If GrayResponseCurve exists, it overrides the PhotometricInterpretation
    /// value. 
    /// 
    /// TODO: impl tag::GrayResponseCurve
    WhiteIsZero,

    /// For bilievel and grayscale images.
    ///
    /// 0 is imaged as black. 2**BitsPerSample - 1 is imaged as white.
    /// If GrayResponseCurve exists, it overrides the PhotometricInterpretation
    /// value. 
    /// 
    /// TODO: impl tag::GrayResponseCurve
    BlackIsZero,
    
    /// Rgb color space.
    ///
    /// In this mode, a color is described as a combination of three primary
    /// colors of light (red, green, blue) in particular concentrations. For each
    /// of the three samples, 0 represents minimum intensity, and  2**BitsPerSample - 1 
    /// represents maximum intensity. 
    RGB,

    /// Rgb color space with lookup table.
    ///
    /// In this mode, a color is described single component. The value of component 
    /// is used as an index into ColorMap (often called `Lookup table`). The value of
    /// component will be converted to a RGB triplet defining actual color by Lookup table.
    ///
    /// #need
    /// 
    /// - IFD must have `tag::ColorMap` tag,
    /// - IFD must have `tag::SamplesPerPixel` with value 1.
    Palette,
    
    /// Irregularly shaped region
    ///
    /// This means that the image is used to define an irregularly shaped region of 
    /// another image in the same TIFF file. Packbits compression is recommended.
    /// The 1-bits define the interior of the region. The 0-bits define the exterior
    /// of the region.
    ///
    /// #need
    ///
    /// - IFD must have `tag::SamplesPerPixel` with value 1.
    /// - IFD must have `tag::BitsPerSample` with value 1.
    ///
    /// #must
    ///
    /// - `ImageLength` must be the same as `ImageLength`.
    TransparencyMask,

    /// Cmyk color space.
    ///
    /// (0,0,0,0) represents white.
    CMYK,
    
    #[allow(missing_docs)]
    YCbCr,

    #[allow(missing_docs)]
    CIELab,
}

impl PhotometricInterpretation {
    pub fn from_u16(n: u16) -> Result<PhotometricInterpretation, ConstructError<tag::PhotometricInterpretation>> {
        use self::PhotometricInterpretation::*;

        match n {
            0 => Ok(WhiteIsZero),
            1 => Ok(BlackIsZero),
            2 => Ok(RGB),
            3 => Ok(Palette),
            4 => Ok(TransparencyMask),
            5 => Ok(CMYK),
            6 => Ok(YCbCr),
            7 => Ok(CIELab),
            n => {
                Err(ConstructError::new(
                    tag::PhotometricInterpretation,
                    n,
                    "tag::PhotometricInterpretation supports between 0 and 7.".to_string()
                ))
            },
        }
    }
}

/// Compression scheme used on the image data.
///
/// IFD constructs this with `tag::Compression`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Compression {
    /// LZW compression
    LZW,
}

impl Compression {
    /// Constructor
    pub fn from_u16(n: u16) -> Result<Option<Compression>, ConstructError<tag::Compression>> {
        match n {
            1 => Ok(None),
            5 => Ok(Some(Compression::LZW)),
            n => {
                Err(ConstructError::new(
                    tag::Compression,
                    n,
                    "tag::Compression supports only 1 (No compression) and 5 (LZW).".to_string()
                ))
            },
        }
    }
}

/// Bits/Sample
/// 
/// IFD constructs this with `tag::BitsPerSample`.
#[derive(Debug, Clone)]
pub struct BitsPerSample(Vec<u16>);

impl BitsPerSample {
    /// Constructor
    pub fn from_u16s(bits: Vec<u16>) -> Result<BitsPerSample, ConstructError<tag::BitsPerSample>> {
        let bits = bits.iter()
            .try_fold(vec![], |mut acc, &x| {
                if 0 < x && x <= 16 {
                    acc.push(x);
                    Ok(acc)
                } else {
                    Err(ConstructError::new(
                        tag::BitsPerSample,
                        bits.clone(),
                        "tag::BitsPerSample supports only values less than 16.".to_string()
                    ))
                }
            })
            .and_then(|x| {
                if x.is_empty() {
                    Err(ConstructError::new(
                        tag::BitsPerSample,
                        bits.clone(),
                        "tag::BitsPerSample supports only when there is more than one value.".to_string()
                    ))
                } else {
                    Ok(x)
                }
            })?;

        Ok(BitsPerSample(bits))
    }

    /// Contents of `BitsPerSample`
    pub fn bits(&self) -> &Vec<u16> {
        &self.0
    }
}

#[derive(Debug, Fail)]
pub enum ImageHeaderBuildError {
    #[fail(display = "Incompatible data ({:?}, {:?}, {:?})", photometric_interpretation, bits_per_sample, samples_per_pixel)]
    IncompatibleData { 
        photometric_interpretation: PhotometricInterpretation,
        bits_per_sample: BitsPerSample,
        samples_per_pixel: u16
    },
}

/// Builder for `ImageHeader`.
///
/// 
#[derive(Debug)]
pub struct ImageHeaderBuilder<PI, BPS, SPP, W, H> {
    photometric_interpretation: PI,
    bits_per_sample: BPS,
    samples_per_pixel: SPP,
    compression: Option<Compression>,
    width: W,
    height: H,
}

impl Default for ImageHeaderBuilder<Empty, Empty, Empty, Empty, Empty> {
    fn default() -> Self {
        ImageHeaderBuilder {
            photometric_interpretation: Empty,
            bits_per_sample: Empty,
            samples_per_pixel: Empty,
            compression: None,
            width: Empty,
            height: Empty,
        }
    }
}

impl<PI, BPS, SPP, W, H> ImageHeaderBuilder<PI, BPS, SPP, W, H> where PI: HasValue, BPS: HasValue, SPP: HasValue, W: HasValue, H: HasValue {
    pub fn photometric_interpretation(self, interpretation: PhotometricInterpretation) -> ImageHeaderBuilder<Filled<PhotometricInterpretation>, BPS, SPP, W, H> {
        ImageHeaderBuilder {
            photometric_interpretation: Filled(interpretation),
            bits_per_sample: self.bits_per_sample,
            samples_per_pixel: self.samples_per_pixel,
            compression: self.compression,
            width: self.width,
            height: self.height,
        }
    }

    pub fn bits_per_sample(self, bits_per_sample: BitsPerSample) -> ImageHeaderBuilder<PI, Filled<BitsPerSample>, SPP, W, H> {
        ImageHeaderBuilder {
            photometric_interpretation: self.photometric_interpretation,
            bits_per_sample: Filled(bits_per_sample),
            samples_per_pixel: self.samples_per_pixel,
            compression: self.compression,
            width: self.width,
            height: self.height,
        }
    }

    pub fn samples_per_pixel(self, samples_per_pixel: u16) -> ImageHeaderBuilder<PI, BPS, Filled<u16>, W, H> {
        ImageHeaderBuilder {
            photometric_interpretation: self.photometric_interpretation,
            bits_per_sample: self.bits_per_sample,
            samples_per_pixel: Filled(samples_per_pixel),
            compression: self.compression,
            width: self.width,
            height: self.height,
        }
    }

    pub fn compression(mut self, compression: Compression) -> Self {
        self.compression = Some(compression);
        self
    }

    pub fn width(self, width: u32) -> ImageHeaderBuilder<PI, BPS, SPP, Filled<u32>, H> {
        ImageHeaderBuilder {
            photometric_interpretation: self.photometric_interpretation,
            bits_per_sample: self.bits_per_sample,
            samples_per_pixel: self.samples_per_pixel,
            compression: self.compression,
            width: Filled(width),
            height: self.height,
        }
    }
    
    pub fn height(self, height: u32) -> ImageHeaderBuilder<PI, BPS, SPP, W, Filled<u32>> {
        ImageHeaderBuilder {
            photometric_interpretation: self.photometric_interpretation,
            bits_per_sample: self.bits_per_sample,
            samples_per_pixel: self.samples_per_pixel,
            compression: self.compression,
            width: self.width,
            height: Filled(height),
        }
    }
}

impl ImageHeaderBuilder<Filled<PhotometricInterpretation>, Filled<BitsPerSample>, Filled<u16>, Filled<u32>, Filled<u32>> {
    pub fn build(self) -> Result<ImageHeader, ImageHeaderBuildError> {
        let photometric_interpretation = self.photometric_interpretation.0;
        let bits_per_sample = self.bits_per_sample.0;
        let samples_per_pixel = self.samples_per_pixel.0;
        let compression = self.compression;
        let width = self.width.0;
        let height = self.height.0;

        // TODO: Error check

        let header = ImageHeader {
            photometric_interpretation: photometric_interpretation,
            bits_per_sample: bits_per_sample,
            samples_per_pixel: samples_per_pixel,
            compression: compression,
            width: width,
            height: height
        };

        Ok(header)
    }
}


#[derive(Debug, Clone)]
pub struct ImageHeader {
    photometric_interpretation: PhotometricInterpretation,
    bits_per_sample: BitsPerSample,
    samples_per_pixel: u16,
    compression: Option<Compression>,
    width: u32,
    height: u32,
}

#[allow(missing_docs)]
impl ImageHeader {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bits_per_sample(&self) -> &BitsPerSample {
        &self.bits_per_sample
    }

    pub fn samples_per_pixel(&self) -> u16 {
        self.samples_per_pixel
    }

    pub fn compression(&self) -> Option<Compression> {
        self.compression
    }

    pub fn photometric_interpretation(&self) -> PhotometricInterpretation {
        self.photometric_interpretation
    }
}

#[derive(Debug)]
pub enum ImageData {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

/// 
/// `tuple in Vec` or `Vec in tuple`
///
/// |R, R, R, ..., R|, |G, G, G, ..., G|, |B, B, B, ... B|
/// ex) (Vec<u8>, Vec<u16>, Vec<u8>)
/// (Vec<bits_per_sample of R>, Vec<bits_per_sample of G>, Vec<bits_per_sample of B>)
///
/// |R, G, B|, |R, G, B|, ..., |R, G, B|
/// ex) Vec<(u8, u16, u8)>
/// Vec<(bits_per_sample of R, bits_per_sample of G, bits_per_sample of B)>
/// 
#[derive(Debug)]
pub struct Image {
    header: ImageHeader,
    data: ImageData,
}

impl Image {
    /// This functions constructs `Image`.
    pub(crate) fn new(header: ImageHeader, data: ImageData) -> Image {
        Image {
            header: header,
            data: data,
        }
    }
    
    /// This function reutrns the reference of `ImageHeader`.
    pub fn header(&self) -> &ImageHeader {
        &self.header
    }

    /// u16 data of every pixel.
    pub fn u16_data(&self) -> Option<&Vec<u16>> {
        match self.data {
            ImageData::U16(ref data) => Some(data),
            _ => None,
        }
    }

    /// u8 data of every pixel.
    pub fn u8_data(&self) -> Option<&Vec<u8>> {
        match self.data {
            ImageData::U8(ref data) => Some(data),
            _ => None,
        }
    }
}


