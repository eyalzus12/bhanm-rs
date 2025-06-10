use super::{AnmBone, AnmReadingError, AnmWritingError};
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct AnmFrame {
    pub id: i16,
    pub bones: Vec<AnmBone>,
    /// affects gameplay! used by grab moves like caspian gauntlets ssig
    pub fire_socket: Option<(f64, f64)>,
    /// unused by the game
    pub eb_platform_pos: Option<(f64, f64)>,
}

impl AnmFrame {
    pub(super) fn read<R: Read>(
        mut reader: R,
        prev_frame: Option<&Self>,
    ) -> Result<Self, AnmReadingError> {
        let id = reader.read_i16::<LE>()?;

        let has_fire_socket = reader.read_u8()? != 0;
        let fire_socket = if has_fire_socket {
            let x = reader.read_f64::<LE>()?;
            let y = reader.read_f64::<LE>()?;
            Some((x, y))
        } else {
            None
        };

        let has_eb_platform = reader.read_u8()? != 0;
        let eb_platform_pos = if has_eb_platform {
            let x = reader.read_f64::<LE>()?;
            let y = reader.read_f64::<LE>()?;
            Some((x, y))
        } else {
            None
        };

        let bone_count = reader.read_i16::<LE>()?;
        if bone_count < 0 {
            return Err(AnmReadingError::NegativeBoneCountError { bone_count });
        }
        let bone_count = bone_count as usize;
        let mut bones: Vec<AnmBone> = Vec::with_capacity(bone_count);
        for i in 0..bone_count {
            let clone_prev = reader.read_u8()? != 0;
            if clone_prev {
                if let Some(prev_frame) = prev_frame {
                    if i >= prev_frame.bones.len() {
                        return Err(AnmReadingError::NoPrevFrameBoneError());
                    }
                    let prev_bone = &prev_frame.bones[i];
                    let clone_frame = reader.read_u8()? != 0;
                    let bone = if !clone_frame {
                        let new_frame = reader.read_i8()?;
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
                let prev_bone = bones.last();
                bones.push(AnmBone::read(&mut reader, prev_bone)?);
            }
        }

        return Ok(Self {
            id,
            bones,
            fire_socket,
            eb_platform_pos,
        });
    }

    pub(super) fn write<W: Write>(
        &self,
        mut writer: W,
        prev_frame: Option<&Self>,
    ) -> Result<(), AnmWritingError> {
        let bone_count = self.bones.len();
        let bone_count = match self.bones.len().try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::TooManyBonesError { bone_count }),
        };

        writer.write_i16::<LE>(self.id)?;

        if let Some(fire_socket) = self.fire_socket {
            writer.write_u8(1)?;
            writer.write_f64::<LE>(fire_socket.0)?;
            writer.write_f64::<LE>(fire_socket.1)?;
        } else {
            writer.write_u8(0)?;
        }

        if let Some(eb_platform_pos) = self.eb_platform_pos {
            writer.write_u8(1)?;
            writer.write_f64::<LE>(eb_platform_pos.0)?;
            writer.write_f64::<LE>(eb_platform_pos.1)?;
        } else {
            writer.write_u8(0)?;
        }

        writer.write_i16::<LE>(bone_count)?;

        for (i, bone) in self.bones.iter().enumerate() {
            let prev_bone = prev_frame.and_then(|f| f.bones.get(i));

            let cloned_prev_bone = prev_bone.and_then(|b| {
                if bone.is_partial_clone_of(b) {
                    Some(b)
                } else {
                    None
                }
            });

            if let Some(cloned_prev_bone) = cloned_prev_bone {
                writer.write_u8(1)?;
                if bone.frame == cloned_prev_bone.frame {
                    writer.write_u8(1)?;
                } else {
                    writer.write_u8(0)?;
                    writer.write_i8(bone.frame)?;
                }
            } else {
                writer.write_u8(0)?;
                bone.write(&mut writer, prev_bone)?;
            }
        }

        Ok(())
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
            let prev_bone = prev_frame.and_then(|f| f.bones.get(i));

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
