use crate::AnmWritingError;

use super::{AnmAnimation, AnmReadingError};
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use std::{collections::HashMap, io::Read, io::Write};

pub struct AnmClass {
    pub index: String,
    pub file_name: String,
    pub animations: AnimationCollection,
}

impl AnmClass {
    pub(super) fn read<R: Read>(mut reader: R) -> Result<Self, AnmReadingError> {
        let index_length = reader.read_u16::<LE>()? as usize;
        let mut index_buf = vec![0u8; index_length];
        reader.read_exact(&mut index_buf)?;
        let index = String::from_utf8(index_buf)?;

        let file_name_length = reader.read_u16::<LE>()? as usize;
        let mut file_name_buf = vec![0u8; file_name_length];
        reader.read_exact(&mut file_name_buf)?;
        let file_name = String::from_utf8(file_name_buf)?;

        let animation_count = reader.read_u32::<LE>()? as usize;
        let mut animations = AnimationCollection::with_capacity(animation_count);
        for _ in 0..animation_count {
            let animation = AnmAnimation::read(&mut reader)?;
            animations.insert(animation);
        }

        Ok(Self {
            index,
            file_name,
            animations,
        })
    }

    pub(super) fn write<W: Write>(&self, mut writer: W) -> Result<(), AnmWritingError> {
        let index_length = self.index.len();
        let index_length = match index_length.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::TooLongClassIndex { index_length }),
        };

        let filename_length = self.file_name.len();
        let filename_length = match filename_length.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::TooLongClassFilename { filename_length }),
        };

        let animation_count = self.animations.len();
        let animation_count = match animation_count.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::TooManyAnimationsError { animation_count }),
        };

        writer.write_u16::<LE>(index_length)?;
        writer.write_all(self.index.as_bytes())?;
        writer.write_u16::<LE>(filename_length)?;
        writer.write_all(self.file_name.as_bytes())?;

        writer.write_u32::<LE>(animation_count)?;
        for animation in self.animations.iter() {
            animation.write(&mut writer)?;
        }

        Ok(())
    }
}

pub struct AnimationCollection {
    animations: HashMap<String, AnmAnimation>,
}

impl AnimationCollection {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            animations: HashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.animations.len()
    }

    pub fn insert(&mut self, animation: AnmAnimation) -> Option<AnmAnimation> {
        self.animations.insert(animation.name.clone(), animation)
    }

    pub fn get(&self, name: &str) -> Option<&AnmAnimation> {
        self.animations.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnmAnimation> {
        self.animations.values()
    }
}
