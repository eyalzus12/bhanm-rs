use super::{AnmClass, AnmReadingError};
use byteorder::{LittleEndian as LE, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::{collections::HashMap, io::Read};

type ClassesCollection = HashMap<String, AnmClass>;

pub struct AnmFile {
    header: i32,
    pub classes: ClassesCollection,
}

impl AnmFile {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, AnmReadingError> {
        let header = reader.read_i32::<LE>()?;
        let zlib = ZlibDecoder::new(reader);
        Ok(Self {
            header,
            classes: Self::read_classes(zlib)?,
        })
    }

    fn read_classes<R: Read>(mut reader: R) -> Result<ClassesCollection, AnmReadingError> {
        let mut classes = ClassesCollection::new();
        while reader.read_u8()? != 0 {
            let key_length = reader.read_u16::<LE>()? as usize;
            let mut key_buf = Vec::with_capacity(key_length);
            reader.read_exact(&mut key_buf)?;
            let key = String::from_utf8(key_buf)?;
            let class = AnmClass::read(&mut reader)?;
            classes.insert(key, class);
        }

        Ok(classes)
    }
}
