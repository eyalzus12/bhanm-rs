use super::{AnmReadingError, AnmWritingError};
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use std::{
    f32,
    io::{Read, Write},
};

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
    pub(super) fn read<R: Read>(
        mut reader: R,
        prev_bone: Option<&Self>,
    ) -> Result<Self, AnmReadingError> {
        let id = reader.read_i16::<LE>()?;
        let opaque = reader.read_u8()? != 0;
        let copy_transform = reader.read_u8()? != 0;

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

            let special_transform = reader.read_u8()? != 0;
            if special_transform {
                identity = reader.read_u8()? != 0;
                symmetric = !identity;
            }

            if identity {
                scale_x = 1.;
                rotate_skew0 = 0.;
                rotate_skew1 = 0.;
                scale_y = 1.;
            } else {
                scale_x = reader.read_f32::<LE>()?;
                rotate_skew0 = reader.read_f32::<LE>()?;
                if symmetric {
                    rotate_skew1 = rotate_skew0;
                    scale_y = -scale_x;
                } else {
                    rotate_skew1 = reader.read_f32::<LE>()?;
                    scale_y = reader.read_f32::<LE>()?;
                }
            }
        }

        let x;
        let y;
        let copy_position = reader.read_u8()? != 0;
        if copy_position {
            if let Some(prev) = prev_bone {
                x = prev.x;
                y = prev.y;
            } else {
                return Err(AnmReadingError::NoPrevBonePositionError().into());
            }
        } else {
            x = reader.read_f32::<LE>()?;
            y = reader.read_f32::<LE>()?;
        }

        let mut frame = 1i8;
        let has_frame = reader.read_u8()? != 0;
        if has_frame {
            frame = reader.read_i8()?;
        }

        let opacity = if !opaque {
            reader.read_u8()? as f64 / 255f64
        } else {
            1.0
        };

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

    pub(super) fn write<W: Write>(
        &self,
        mut writer: W,
        prev_bone: Option<&Self>,
    ) -> Result<(), AnmWritingError> {
        writer.write_i16::<LE>(self.id)?;
        let opaque = self.opacity == 1.;
        writer.write_u8(if opaque { 1 } else { 0 })?;

        let copy_transform = if let Some(prev) = prev_bone {
            self.has_same_transform_as(&prev)
        } else {
            false
        };
        if copy_transform {
            writer.write_u8(1)?;
        } else {
            writer.write_u8(0)?;
            let is_identity = self.is_identity();
            let is_symmetric = self.is_symmetric();
            if is_identity || is_symmetric {
                writer.write_u8(1)?;
                if is_identity {
                    writer.write_u8(1)?;
                } else {
                    writer.write_u8(0)?;
                }
            } else {
                writer.write_u8(0)?;
            }

            if !is_identity {
                writer.write_f32::<LE>(self.scale_x)?;
                writer.write_f32::<LE>(self.rotate_skew0)?;
                if !is_symmetric {
                    writer.write_f32::<LE>(self.rotate_skew1)?;
                    writer.write_f32::<LE>(self.scale_y)?;
                }
            }
        }

        let copy_position = if let Some(prev) = prev_bone {
            self.has_same_position_as(&prev)
        } else {
            false
        };
        if copy_position {
            writer.write_u8(1)?;
        } else {
            writer.write_u8(0)?;
            writer.write_f32::<LE>(self.x)?;
            writer.write_f32::<LE>(self.y)?;
        }

        if self.frame == 1 {
            writer.write_u8(0)?;
        } else {
            writer.write_u8(1)?;
            writer.write_i8(self.frame)?;
        }

        if !opaque {
            let opacity_byte = f64::round(self.opacity * 255.) as u8;
            writer.write_u8(opacity_byte)?;
        }

        Ok(())
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

    pub(super) fn get_byte_size(&self, prev_bone: Option<&Self>) -> usize {
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
