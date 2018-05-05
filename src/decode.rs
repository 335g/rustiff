
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use error::Result;
use byte::{
    Endian,
    EndianReadExt,
    ReadExt,
    SeekExt,
};
use ifd::{
    self,
    IFD,
};

use std::{
    io::{
        self,
        Read,
        Seek,
    },
};

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    endian: Endian,
    ifds: Vec<IFD>,
}

impl<R> Decoder<R> where R: Read + Seek {
    pub fn new(mut reader: R) -> Result<Decoder<R>> {
        let mut byte_order = [0u8; 2];
        reader.read(&mut byte_order)?;
        let endian = match &byte_order {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => bail!("Not tiff"),
        };

        if reader.read_u16(&endian)? != 42 {
            bail!("Not tiff");
        }

        let ifds = Decoder::read_ifds(&mut reader, &endian)?;

        let decoder = Decoder {
            reader: reader,
            endian: endian,
            ifds: ifds,
        };

        Ok(decoder)
    }

    fn read_ifds(mut reader: R, endian: &Endian) -> Result<Vec<IFD>> {
        fn read_ifd<R>(mut reader: R, endian: &Endian, next: &mut u32) -> Result<IFD> 
        where 
            R: Read + Seek 
        {
            fn read_entry<R>(mut reader: R, endian: &Endian) -> Result<(ifd::Tag, ifd::Entry)>
            where
                R: Read + Seek,
            {
                unimplemented!()
            }

            reader.goto(*next as u64)?;

            let mut ifd = IFD::new();
            for _ in 0..reader.read_u16(&endian)? {
                let (tag, entry) = read_entry(&mut reader, &endian)?;
                ifd.insert(tag, entry);
            }
            
            // Update next addr
            *next = reader.read_u32(&endian)?;
            
            Ok(ifd)
        }
        
        // `4` is the address for the initial IFD
        let mut next: u32 = 4;
        let mut ifds = vec![];
        loop {
            let ifd = read_ifd(&mut reader, &endian, &mut next)?;
            ifds.push(ifd);

            next = reader.read_u32(&endian)?;
            if next == 0 {
                break
            }
        }

        Ok(ifds)
    }
}


