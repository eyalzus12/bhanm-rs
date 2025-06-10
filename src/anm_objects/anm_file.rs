use super::{AnmClass, AnmReadingError, AnmWritingError};
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::{
    collections::HashMap,
    io::{Read, Write},
};

type ClassesCollection = HashMap<String, AnmClass>;

pub struct AnmFile {
    pub header: i32,
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

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), AnmWritingError> {
        writer.write_i32::<LE>(self.header)?;
        let zlib = ZlibEncoder::new(writer, Compression::best());
        self.write_classes(zlib)?;

        Ok(())
    }

    fn read_classes<R: Read>(mut reader: R) -> Result<ClassesCollection, AnmReadingError> {
        let mut classes = ClassesCollection::new();
        while reader.read_u8()? != 0 {
            let key_length = reader.read_u16::<LE>()? as usize;
            let mut key_buf = vec![0u8; key_length];
            reader.read_exact(&mut key_buf)?;
            let key = String::from_utf8(key_buf)?;
            let class = AnmClass::read(&mut reader)?;
            classes.insert(key, class);
        }

        Ok(classes)
    }

    fn write_classes<W: Write>(&self, mut writer: W) -> Result<(), AnmWritingError> {
        for (key, class) in self.classes.iter() {
            let key_length = key.len();
            let key_length = match key_length.try_into() {
                Ok(v) => v,
                Err(_) => return Err(AnmWritingError::TooLongClassKey { key_length }),
            };

            writer.write_u8(1)?;
            writer.write_u16::<LE>(key_length)?;
            writer.write_all(key.as_bytes())?;
            class.write(&mut writer)?;
        }
        writer.write_u8(0)?;

        Ok(())
    }
}
