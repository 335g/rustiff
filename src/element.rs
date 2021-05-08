use crate::{data::DataType, error::DecodingError, val::Value};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io;

/// enum for `byteorder::BigEndian` and `byteorder::LittleEndian`.
///
/// They should be treated as the same type, because `decoder::Decoder`
/// determine endian according to file contents.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Element {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    Char(char),
    F32(f32),
    F64(f64),
    Value(Value),
}

pub trait Elemental: Sized {
    fn size(datatype: &DataType) -> usize;
    fn element(&self) -> Element;

    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType)
        -> Result<Self, DecodingError>;
}

impl Elemental for u8 {
    fn size(_datatype: &DataType) -> usize {
        1
    }

    fn element(&self) -> Element {
        Element::U8(*self)
    }

    /// Reads an u8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        _endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Byte => {
                let val = <R as ReadBytesExt>::read_u8(reader)?;
                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for i8 {
    fn size(_datatype: &DataType) -> usize {
        1
    }

    fn element(&self) -> Element {
        Element::I8(*self)
    }

    /// Reads an i8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        _endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::SByte => {
                let val = <R as ReadBytesExt>::read_i8(reader)?;
                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for u16 {
    fn size(_datatype: &DataType) -> usize {
        2
    }

    fn element(&self) -> Element {
        Element::U16(*self)
    }

    /// Reads an u16 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Short => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_u16::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_u16::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for i16 {
    fn size(_datatype: &DataType) -> usize {
        2
    }

    fn element(&self) -> Element {
        Element::I16(*self)
    }

    /// Reads an i16 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::SShort => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_i16::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_i16::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for u32 {
    fn size(_datatype: &DataType) -> usize {
        4
    }

    fn element(&self) -> Element {
        Element::U32(*self)
    }

    /// Reads an u32 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Long => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_u32::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_u32::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for i32 {
    fn size(_datatype: &DataType) -> usize {
        4
    }

    fn element(&self) -> Element {
        Element::I32(*self)
    }

    /// Reads an i32 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::SLong => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_i32::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_i32::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty))
        }
    }
}

impl Elemental for char {
    fn size(_datatype: &DataType) -> usize {
        1
    }

    fn element(&self) -> Element {
        Element::Char(*self)
    }

    /// Reads an ascii char.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    /// This method also returns error, If read value is not ascii char.
    ///
    fn read<R: io::Read>(
        reader: &mut R,
        _endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Ascii => {
                let val = <R as ReadBytesExt>::read_u8(reader).and_then(|x| {
                    x.is_ascii().then(|| char::from(x)).ok_or(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Read data is not ascii char",
                    ))
                })?;

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for f32 {
    fn size(_datatype: &DataType) -> usize {
        4
    }

    fn element(&self) -> Element {
        Element::F32(*self)
    }

    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Float => {
                let mut buf = [0u8; 4];
                let _ = reader.read_exact(&mut buf)?;

                let val = match *endian {
                    Endian::Big => u32::from_be_bytes(buf),
                    Endian::Little => u32::from_le_bytes(buf),
                };
                let val = f32::from_bits(val);

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty))
        }
    }
}

impl Elemental for f64 {
    fn size(_datatype: &DataType) -> usize {
        8
    }

    fn element(&self) -> Element {
        Element::F64(*self)
    }

    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Double => {
                let mut buf = [0u8; 8];
                let _ = reader.read_exact(&mut buf)?;

                let val = match *endian {
                    Endian::Big => u64::from_be_bytes(buf),
                    Endian::Little => u64::from_le_bytes(buf),
                };
                let val = f64::from_bits(val);

                Ok(val)
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

impl Elemental for Value {
    fn size(datatype: &DataType) -> usize {
        match datatype {
            DataType::Short => 2,
            DataType::Long => 4,
            _ => 4, // Not use
        }
    }

    fn element(&self) -> Element {
        Element::Value(self.clone())
    }

    fn read<R: io::Read>(
        reader: &mut R,
        endian: &Endian,
        datatype: DataType,
    ) -> Result<Self, DecodingError> {
        match datatype {
            DataType::Short => {
                let val = EndianRead::read_u16(reader, endian)?;
                Ok(Value::Short(val))
            }
            DataType::Long => {
                let val = EndianRead::read_u32(reader, endian)?;
                Ok(Value::Long(val))
            }
            ty => Err(DecodingError::InvalidDataType(ty)),
        }
    }
}

pub trait EndianRead: io::Read {
    /// Reads an u8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_u8(&mut self, _byte_order: &Endian) -> io::Result<u8> {
        <Self as ReadBytesExt>::read_u8(self)
    }

    /// Reads an i8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_i8(&mut self, _byte_order: &Endian) -> io::Result<i8> {
        <Self as ReadBytesExt>::read_i8(self)
    }

    /// Reads an u16 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_u16(&mut self, byte_order: &Endian) -> io::Result<u16> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
        }
    }

    /// Reads an i16 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_i16(&mut self, byte_order: &Endian) -> io::Result<i16> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_i16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_i16::<LittleEndian>(self),
        }
    }

    /// Reads an u32 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_u32(&mut self, byte_order: &Endian) -> io::Result<u32> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u32::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
        }
    }

    /// Reads an i32 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_i32(&mut self, byte_order: &Endian) -> io::Result<i32> {
        match *byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_i32::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_i32::<LittleEndian>(self),
        }
    }

    /// Reads an four u8 values.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_4byte(&mut self) -> io::Result<[u8; 4]> {
        let mut val = [0u8; 4];
        let _ = self.read_exact(&mut val)?;

        Ok(val)
    }

    /// Reads an ascii char.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    /// This method also returns error, If read value is not ascii char.
    ///
    #[inline(always)]
    fn read_ascii(&mut self) -> io::Result<char> {
        <Self as ReadBytesExt>::read_u8(self).and_then(|x| {
            x.is_ascii().then(|| char::from(x)).ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Read data is not ascii char",
            ))
        })
    }

    #[inline(always)]
    fn read_f32(&mut self, byte_order: &Endian) -> io::Result<f32> {
        let mut buf = [0u8; 4];
        let _ = self.read_exact(&mut buf)?;

        let val = match *byte_order {
            Endian::Big => u32::from_be_bytes(buf),
            Endian::Little => u32::from_le_bytes(buf),
        };
        let val = f32::from_bits(val);

        Ok(val)
    }

    #[inline(always)]
    fn read_f64(&mut self, byte_order: &Endian) -> io::Result<f64> {
        let mut buf = [0u8; 8];
        let _ = self.read_exact(&mut buf)?;

        let val = match *byte_order {
            Endian::Big => u64::from_be_bytes(buf),
            Endian::Little => u64::from_le_bytes(buf),
        };
        let val = f64::from_bits(val);

        Ok(val)
    }
}

impl<R: io::Read> EndianRead for R {}

pub trait SeekExt: io::Seek {
    fn goto(&mut self, x: u64) -> io::Result<()> {
        self.seek(io::SeekFrom::Start(x)).map(|_| ())
    }
}

impl<S: io::Seek> SeekExt for S {}
