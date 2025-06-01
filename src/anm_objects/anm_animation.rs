use byteorder::{LittleEndian as LE, ReadBytesExt};
use std::io::Read;

use super::{AnmFrame, AnmReadingError};

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
        let mut name_buf = Vec::with_capacity(name_length);
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
        for i in 0..frame_count {
            let prev_frame = if i == 0 { None } else { Some(&frames[i - 1]) };
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
}
