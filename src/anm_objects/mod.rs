use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnmReadingError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("A bone tries to copy transform from a previous bone, but there is no previous bone")]
    NoPrevBoneTransformError(),
    #[error("A bone tries to copy position from a previous bone, but there is no previous bone")]
    NoPrevBonePositionError(),
    #[error(
        "A frame tries to duplicate a bone from a previous frame, but there is no previous frame"
    )]
    NoPrevFrameError(),
    #[error(
        "A frame tries to duplicate a bone from a previous frame, but there is no matching bone"
    )]
    NoPrevFrameBoneError(),
    #[error("A frame has a negative number of bones: ({bone_count:?})")]
    NegativeBoneCountError { bone_count: i16 },
}

mod anm_bone;
pub use anm_bone::AnmBone;
mod anm_frame;
pub use anm_frame::AnmFrame;
mod anm_animation;
pub use anm_animation::AnmAnimation;
mod anm_class;
pub use anm_class::AnmClass;
mod anm_file;
pub use anm_file::AnmFile;
