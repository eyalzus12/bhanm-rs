use super::{AnmBone, AnmReadingError};
use std::io::Read;

pub struct AnmFrame {
    pub id: i16,
    pub bones: Vec<AnmBone>,
    /// affects gameplay! used by grab moves like caspian gauntlets ssig
    pub fire_socket: Option<(f64, f64)>,
    /// unused by the game
    pub eb_platform_pos: Option<(f64, f64)>,
}

impl AnmFrame {
    pub fn new<R: Read>(mut reader: R, prev_frame: Option<&Self>) -> Result<Self, AnmReadingError> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf[..2])?;
        let id = i16::from_le_bytes(*buf.first_chunk().unwrap());
        reader.read_exact(&mut buf[..1])?;
        let has_fire_socket = buf[0] != 0;

        let fire_socket = if has_fire_socket {
            reader.read_exact(&mut buf)?;
            let x = f64::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            let y = f64::from_le_bytes(buf);
            Some((x, y))
        } else {
            None
        };

        reader.read_exact(&mut buf[..1])?;
        let has_eb_platform = buf[0] != 0;
        let eb_platform_pos = if has_eb_platform {
            reader.read_exact(&mut buf)?;
            let x = f64::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            let y = f64::from_le_bytes(buf);
            Some((x, y))
        } else {
            None
        };

        reader.read_exact(&mut buf[..2])?;
        let _bone_count: i16 = i16::from_le_bytes(*buf.first_chunk().unwrap());
        if _bone_count < 0 {
            return Err(AnmReadingError::NegativeBoneCountError {
                bone_count: _bone_count,
            });
        }
        let bone_count = _bone_count as usize;
        let mut bones: Vec<AnmBone> = Vec::with_capacity(bone_count);
        for i in 0..bone_count {
            reader.read_exact(&mut buf[..1])?;
            let clone_prev = buf[0] != 0;
            if clone_prev {
                if let Some(prev) = prev_frame {
                    if i >= prev.bones.len() {
                        return Err(AnmReadingError::NoPrevFrameBoneError());
                    }
                    let prev_bone = &prev.bones[i];
                    reader.read_exact(&mut buf[..1])?;
                    let change_frame = buf[0] != 0;
                    let bone = if change_frame {
                        reader.read_exact(&mut buf[..1])?;
                        let new_frame = buf[0] as i8;
                        AnmBone {
                            frame: new_frame,
                            ..prev_bone.clone()
                        }
                    } else {
                        prev_bone.clone()
                    };
                    bones.push(bone);
                } else {
                    return Err(AnmReadingError::NoPrevFrameError());
                }
            } else {
                let prev_bone = if i > 0 { Some(&bones[i - 1]) } else { None };
                bones.push(AnmBone::new(&mut reader, prev_bone)?);
            }
        }

        return Ok(Self {
            id,
            bones,
            fire_socket,
            eb_platform_pos,
        });
    }

    pub(super) fn get_byte_size(&self, prev_frame: Option<&Self>) -> usize {
        let mut result = 0usize;
        result += size_of::<i16>(); // id

        result += size_of::<u8>(); // fire socket indicator
        if self.fire_socket.is_some() {
            result += size_of::<f64>() * 2;
        }
        result += size_of::<u8>(); // eb platform indicator
        if self.eb_platform_pos.is_some() {
            result += size_of::<f64>() * 2;
        }

        result += size_of::<i16>(); // bone count
        for (i, bone) in self.bones.iter().enumerate() {
            let prev_bone = match prev_frame {
                Some(prev) => {
                    if i < prev.bones.len() {
                        Some(&prev.bones[i])
                    } else {
                        None
                    }
                }
                None => None,
            };

            result += size_of::<u8>(); // prev frame clone indicator

            enum BoneCloneLevel {
                None,
                Partial,
                Full,
            }
            let bone_clone_level = match prev_bone {
                Some(prev_bone) => {
                    if bone.is_partial_clone_of(&prev_bone) {
                        if bone.frame == prev_bone.frame {
                            BoneCloneLevel::Full
                        } else {
                            BoneCloneLevel::Partial
                        }
                    } else {
                        BoneCloneLevel::None
                    }
                }
                None => BoneCloneLevel::None,
            };

            match bone_clone_level {
                BoneCloneLevel::Full => {
                    result += size_of::<u8>(); // full copy indicator
                }
                BoneCloneLevel::Partial => {
                    result += size_of::<u8>(); // full copy indicator
                    result += size_of::<i8>(); // frame override
                }
                BoneCloneLevel::None => {
                    result += bone.get_byte_size(prev_bone);
                }
            }
        }

        return result;
    }
}
