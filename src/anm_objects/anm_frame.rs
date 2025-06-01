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
            }
        }

        return Ok(Self {
            id,
            bones,
            fire_socket,
            eb_platform_pos,
        });
    }
}
