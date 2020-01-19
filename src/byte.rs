use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io;

/// enum for `byteorder::BigEndian` and `byteorder::LittleEndian`.
///
/// They should be treated as the same type, because `decoder::Decoder`
/// determine endian according to file contents.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    fn read_u8(&mut self, _byte_order: Endian) -> io::Result<u8> {
        <Self as ReadBytesExt>::read_u8(self)
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
    fn read_u16(&mut self, byte_order: Endian) -> io::Result<u16> {
        match byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
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
    fn read_u32(&mut self, byte_order: Endian) -> io::Result<u32> {
        match byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u32::<BigEndian>(self),
            Endian::Little => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
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
    #[inline]
    fn read_4byte(&mut self) -> io::Result<[u8; 4]> {
        let mut val = [0u8; 4];
        let _ = self.read_exact(&mut val)?;

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
