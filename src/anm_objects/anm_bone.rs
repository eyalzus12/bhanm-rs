use super::{AnmReadingError, ByteSized};
use std::{f32, io::Read};

#[derive(Clone)]
pub struct AnmBone {
    pub id: i16,
    pub scale_x: f32,
    pub rotate_skew0: f32,
    pub rotate_skew1: f32,
    pub scale_y: f32,
    pub x: f32,
    pub y: f32,
    pub opacity: f64,
    pub frame: i8,
}

impl AnmBone {
    pub fn new<R: Read>(mut reader: R, prev_bone: Option<&Self>) -> Result<Self, AnmReadingError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[..2])?;
        let id = i16::from_le_bytes(*buf.first_chunk().unwrap());
        reader.read_exact(&mut buf[..1])?;
        let opaque = buf[0] != 0;
        reader.read_exact(&mut buf[..1])?;
        let copy_transform: bool = buf[0] != 0;

        let scale_x;
        let rotate_skew0;
        let rotate_skew1;
        let scale_y;
        if copy_transform {
            if let Some(prev) = prev_bone {
                scale_x = prev.scale_x;
                rotate_skew0 = prev.rotate_skew0;
                rotate_skew1 = prev.rotate_skew1;
                scale_y = prev.scale_y;
            } else {
                return Err(AnmReadingError::NoPrevBoneTransformError().into());
            }
        } else {
            let mut identity = false;
            let mut symmetric = false;

            reader.read_exact(&mut buf[..1])?;
            let special_transform = buf[0] != 0;
            if special_transform {
                reader.read_exact(&mut buf[..1])?;
                identity = buf[0] != 0;
                symmetric = !identity;
            }

            if identity {
                scale_x = 1.;
                rotate_skew0 = 0.;
                rotate_skew1 = 0.;
                scale_y = 1.;
            } else {
                reader.read_exact(&mut buf)?;
                scale_x = f32::from_le_bytes(buf);
                reader.read_exact(&mut buf)?;
                rotate_skew0 = f32::from_le_bytes(buf);
                if symmetric {
                    rotate_skew1 = rotate_skew0;
                    scale_y = -scale_x;
                } else {
                    reader.read_exact(&mut buf)?;
                    rotate_skew1 = f32::from_le_bytes(buf);
                    reader.read_exact(&mut buf)?;
                    scale_y = f32::from_le_bytes(buf);
                }
            }
        }

        let x;
        let y;
        reader.read_exact(&mut buf[..1])?;
        let copy_position = buf[0] != 0;
        if copy_position {
            if let Some(prev) = prev_bone {
                x = prev.x;
                y = prev.y;
            } else {
                return Err(AnmReadingError::NoPrevBonePositionError().into());
            }
        } else {
            reader.read_exact(&mut buf)?;
            x = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            y = f32::from_le_bytes(buf);
        }

        let mut frame = 1i8;
        reader.read_exact(&mut buf[..1])?;
        let has_frame = buf[0] != 0;
        if has_frame {
            reader.read_exact(&mut buf[..1])?;
            frame = buf[0] as i8;
        }

        let mut opacity = 1.0f64;
        if !opaque {
            reader.read_exact(&mut buf[..1])?;
            let opacity_byte = buf[0];
            opacity = opacity_byte as f64 / 255f64;
        }

        Ok(Self {
            id,
            scale_x,
            rotate_skew0,
            rotate_skew1,
            scale_y,
            x,
            y,
            opacity,
            frame,
        })
    }

    pub(super) fn is_partial_clone_of(&self, other: &Self) -> bool {
        self.has_same_transform_as(other)
            && self.has_same_position_as(other)
            && self.id == other.id
            && self.frame == other.frame
            && self.opacity == other.opacity
    }

    fn has_same_transform_as(&self, other: &Self) -> bool {
        self.scale_x == other.scale_x
            && self.rotate_skew0 == other.rotate_skew0
            && self.rotate_skew1 == other.rotate_skew1
            && self.scale_y == other.scale_y
    }

    fn has_same_position_as(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn is_identity(&self) -> bool {
        self.scale_x == 1.
            && self.rotate_skew0 == 0.
            && self.rotate_skew1 == 0.
            && self.scale_y == 1.
    }

    fn is_symmetric(&self) -> bool {
        self.scale_y == -self.scale_x && self.rotate_skew0 == self.rotate_skew1
    }
}

impl ByteSized for AnmBone {
    fn get_byte_size(&self, prev_bone: Option<&Self>) -> usize {
        let mut result = 0usize;
        result += size_of::<u16>(); // id
        result += size_of::<u8>(); // opaque

        result += size_of::<u8>(); // copy transform indicator
        let copy_transform = match prev_bone {
            Some(prev) => self.has_same_transform_as(prev),
            None => false,
        };
        // can't copy transform, so have to store more data
        if !copy_transform {
            result += size_of::<u8>(); // identity/symmetric indicator
            if self.is_identity() {
                result += size_of::<u8>(); // 2nd indicator
            } else {
                result += size_of::<f32>(); // scale_x
                result += size_of::<f32>(); // rotate_skew0
                if self.is_symmetric() {
                    result += size_of::<f32>(); // 2nd indicator
                } else {
                    result += size_of::<f32>(); // rotate_skew1
                    result += size_of::<f32>(); // scale_y
                }
            }
        }

        result += size_of::<u8>(); // copy position indicator
        let copy_position = match prev_bone {
            Some(prev) => self.has_same_position_as(prev),
            None => false,
        };
        // can't copy position, so have to store more data
        if !copy_position {
            result += size_of::<f32>(); // x
            result += size_of::<f32>(); // y
        }

        result += size_of::<u8>(); // default frame indicator
        if self.frame != 1 {
            result += size_of::<i8>(); // frame
        }

        if self.opacity != 1. {
            result += size_of::<u8>(); // opacity
        }

        return result;
    }
}
