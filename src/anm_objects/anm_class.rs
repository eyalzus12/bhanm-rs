use super::{AnmAnimation, AnmReadingError};
use byteorder::{LittleEndian as LE, ReadBytesExt};
use std::{collections::HashMap, io::Read};

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
