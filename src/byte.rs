
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use byteorder::{
    LittleEndian,
    BigEndian,
    ReadBytesExt,
};

use std::{
    io::{
        self,
        Read,
        Seek,
        Cursor,
    },
};

/// enum for `byteorder::BigEndian` and `byteorder::LittleEndian`.
///
/// They should be treated as the same type, because `decoder::Decoder` 
/// determine endian according to file contents.
#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Big,
    Little,
}

/// private trait for Read extension.
pub(crate) trait ReadExt: Read {
    /// Reads an u8 value.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline(always)]
    fn read_u8(&mut self) -> io::Result<u8> {
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

    /// Reads a sequence of unsigned 16 bit integers from the underlying reader.
    ///
    /// #errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    ///
    #[inline]
    fn read_u16_into(&mut self, byte_order: Endian, dst: &mut [u16]) -> io::Result<()> {
        match byte_order {
            Endian::Big => <Self as ReadBytesExt>::read_u16_into::<BigEndian>(self, dst)?,
            Endian::Little => <Self as ReadBytesExt>::read_u16_into::<LittleEndian>(self, dst)?,
        }
        Ok(())
    }
}

impl<R: Read> ReadExt for R {}

/// private trait for Seek extension.
pub(crate) trait SeekExt: Seek {
    /// jump memory address.
    fn goto(&mut self, x: u64) -> io::Result<()> {
        self.seek(io::SeekFrom::Start(x))
            .map(|_| ())
    }
}

impl<S: Seek> SeekExt for S {}

/// Reader for data compressed in LZW format.
#[derive(Debug)]
pub(crate) struct LZWReader(Cursor<Vec<u8>>);

impl LZWReader {
    #[allow(missing_docs)]
    pub fn new<R>(reader: &mut R, compressed_len: usize) -> io::Result<(LZWReader, usize)> where R: Read {
        let mut compressed = vec![0; compressed_len as usize];
        reader.read_exact(&mut compressed)?;
        let mut uncompressed = vec![];
        let mut decoder = ::lzw::DecoderEarlyChange::new(::lzw::MsbReader::new(), 8);
        let mut read = 0;
        while read < compressed_len {
            let (len, bytes) = decoder.decode_bytes(&compressed[read..])?;
            read += len;
            uncompressed.extend_from_slice(bytes);
        }

        let bytes = uncompressed.len();
        let reader = LZWReader(io::Cursor::new(uncompressed));

        Ok((reader, bytes))
    }
}

impl Read for LZWReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

