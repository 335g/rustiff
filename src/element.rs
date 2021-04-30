use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io;
use crate::{error::DecodingElementError, val::Value};
use crate::data::DataType;

/// enum for `byteorder::BigEndian` and `byteorder::LittleEndian`.
///
/// They should be treated as the same type, because `decoder::Decoder`
/// determine endian according to file contents.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

pub trait Element: Sized {
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError>;
}

impl Element for u8 {
    /// Reads an u8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(reader: &mut R, _endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Byte => {
                let val = <R as ReadBytesExt>::read_u8(reader)?;
                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for i8 {
    /// Reads an i8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(reader: &mut R, _endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::SByte => {
                let val = <R as ReadBytesExt>::read_i8(reader)?;
                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for u16 {
    /// Reads an u16 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Short => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_u16::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_u16::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for i16 {
    /// Reads an i16 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::SShort => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_i16::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_i16::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for u32 {
    /// Reads an u32 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Long => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_u32::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_u32::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for i32 {
    /// Reads an i32 value with Endian.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::SLong => {
                let val = match *endian {
                    Endian::Big => <R as ReadBytesExt>::read_i32::<BigEndian>(reader),
                    Endian::Little => <R as ReadBytesExt>::read_i32::<LittleEndian>(reader),
                }?;

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for char {
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
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Ascii => {
                let val = <R as ReadBytesExt>::read_u8(reader)
                .and_then(|x| {
                    x.is_ascii()
                        .then(|| char::from(x))
                        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Read data is not ascii char"))
                })?;

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for f32 {
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Float => {
                let mut buf = [0u8; 4];
                let _ = reader.read_exact(&mut buf)?;

                let val = match *endian {
                    Endian::Big => u32::from_be_bytes(buf),
                    Endian::Little => u32::from_le_bytes(buf)
                };
                let val = f32::from_bits(val);

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for f64 {
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Double => {
                let mut buf = [0u8; 8];
                let _ = reader.read_exact(&mut buf)?;

                let val = match *endian {
                    Endian::Big => u64::from_be_bytes(buf),
                    Endian::Little => u64::from_le_bytes(buf)
                };
                let val = f64::from_bits(val);

                Ok(val)
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

impl Element for Value {
    fn read<R: io::Read>(reader: &mut R, endian: &Endian, datatype: DataType) -> Result<Self, DecodingElementError> {
        match datatype {
            DataType::Short => {
                let val = u16::read(reader, endian, DataType::Short)?;
                Ok(Value::Short(val))
            }
            DataType::Long => {
                let val = u32::read(reader, endian, DataType::Long)?;
                Ok(Value::Long(val))
            }
            ty => {
                Err(DecodingElementError::NoMatchDataType {
                    element: std::any::type_name::<Self>(),
                    datatype: ty
                })
            }
        }
    }
}

pub trait SeekExt: io::Seek {
    fn goto(&mut self, x: u64) -> io::Result<()> {
        self.seek(io::SeekFrom::Start(x)).map(|_| ())
    }
}

impl<S: io::Seek> SeekExt for S {}

