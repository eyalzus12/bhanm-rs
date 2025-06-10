use super::{AnmFrame, AnmReadingError, AnmWritingError};
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct AnmAnimation {
    pub name: String,
    pub loop_start: u32,
    pub recovery_start: u32,
    pub free_start: u32,
    pub preview_frame: u32,
    pub base_start: u32,
    pub data: Vec<u32>,
    pub frames: Vec<AnmFrame>,
}

impl AnmAnimation {
    pub(super) fn read<R: Read>(mut reader: R) -> Result<Self, AnmReadingError> {
        let name_length = reader.read_u16::<LE>()? as usize;
        let mut name_buf = vec![0u8; name_length];
        reader.read_exact(&mut name_buf)?;
        let name = String::from_utf8(name_buf)?;

        let frame_count = reader.read_u32::<LE>()? as usize;
        let loop_start = reader.read_u32::<LE>()?;
        let recovery_start = reader.read_u32::<LE>()?;
        let free_start = reader.read_u32::<LE>()?;
        let preview_frame = reader.read_u32::<LE>()?;
        let base_start = reader.read_u32::<LE>()?;

        let data_size = reader.read_u32::<LE>()? as usize;
        let mut data = Vec::with_capacity(data_size);
        for _ in 0..data_size {
            data.push(reader.read_u32::<LE>()?);
        }

        /*
        this field stores the size of the frames array.
        it is used by the game to skip parsing the frames until it needs them.
        we don't need it here, so we discard it.
        */
        _ = reader.read_u32::<LE>()?;

        let mut frames = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            let prev_frame = frames.last();
            frames.push(AnmFrame::read(&mut reader, prev_frame)?);
        }

        Ok(Self {
            name,
            loop_start,
            recovery_start,
            free_start,
            preview_frame,
            base_start,
            data,
            frames,
        })
    }

    pub(super) fn write<W: Write>(&self, mut writer: W) -> Result<(), AnmWritingError> {
        let name_length = self.name.len();
        let name_length = match name_length.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::LongAnimNameError { name_length }),
        };

        let frame_count = self.frames.len();
        let frame_count = match frame_count.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::TooManyFramesError { frame_count }),
        };

        let data_length = self.data.len();
        let data_length = match data_length.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::DataArrayTooLongError { data_length }),
        };

        let byte_count = self.get_frames_byte_size();
        let byte_count = match byte_count.try_into() {
            Ok(v) => v,
            Err(_) => return Err(AnmWritingError::AnimationDataTooLargeError { byte_count }),
        };

        writer.write_u16::<LE>(name_length)?;
        writer.write_all(self.name.as_bytes())?;

        writer.write_u32::<LE>(frame_count)?;
        writer.write_u32::<LE>(self.loop_start)?;
        writer.write_u32::<LE>(self.recovery_start)?;
        writer.write_u32::<LE>(self.free_start)?;
        writer.write_u32::<LE>(self.preview_frame)?;
        writer.write_u32::<LE>(self.base_start)?;
        writer.write_u32::<LE>(data_length)?;
        for datum in &self.data {
            writer.write_u32::<LE>(*datum)?;
        }
        writer.write_u32::<LE>(byte_count)?;
        for (i, frame) in self.frames.iter().enumerate() {
            let prev_frame = if i == 0 {
                None
            } else {
                Some(&self.frames[i - 1])
            };
            frame.write(&mut writer, prev_frame)?;
        }

        Ok(())
    }

    fn get_frames_byte_size(&self) -> usize {
        let mut result = 0usize;
        for (i, frame) in self.frames.iter().enumerate() {
            let prev_frame = if i == 0 {
                None
            } else {
                Some(&self.frames[i - 1])
            };
            result += frame.get_byte_size(prev_frame);
        }
        return result;
    }
}
