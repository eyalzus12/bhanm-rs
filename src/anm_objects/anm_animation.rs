use super::AnmFrame;

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

