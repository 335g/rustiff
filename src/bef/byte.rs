use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
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
        <Self as ReadBytesExt>::read_u8(self)
            .and_then(|x| {
                x.is_ascii()
                    .then(|| char::from(x))
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Read data is not ascii char"))
            })
    }

    #[inline(always)]
    fn read_f32(&mut self, byte_order: &Endian) -> io::Result<f32> {
        let mut buf = [0u8; 4];
        let _ = self.read_exact(&mut buf)?;

        let val = match *byte_order {
            Endian::Big => u32::from_be_bytes(buf),
            Endian::Little => u32::from_le_bytes(buf)
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
            Endian::Little => u64::from_le_bytes(buf)
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

pub trait EndianWrite: io::Write {
    #[inline(always)]
    fn write_u8(&mut self, _byte_order: &Endian, n: u8) -> io::Result<()> {
        <Self as WriteBytesExt>::write_u8(self, n)
    }

    #[inline(always)]
    fn write_u16(&mut self, byte_order: &Endian, n: u16) -> io::Result<()> {
        match *byte_order {
            Endian::Big => <Self as WriteBytesExt>::write_u16::<BigEndian>(self, n),
            Endian::Little => <Self as WriteBytesExt>::write_u16::<LittleEndian>(self, n),
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, byte_order: &Endian, n: u32) -> io::Result<()> {
        match *byte_order {
            Endian::Big => <Self as WriteBytesExt>::write_u32::<BigEndian>(self, n),
            Endian::Little => <Self as WriteBytesExt>::write_u32::<LittleEndian>(self, n),
        }
    }

    #[inline(always)]
    fn write_4byte(&mut self, x: [u8; 4]) -> io::Result<()> {
        <Self as io::Write>::write_all(self, &x)
    }
}

impl<W: io::Write> EndianWrite for W {}
