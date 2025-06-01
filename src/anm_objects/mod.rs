use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnmReadingError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("bone tries to copy transform from a previous bone, but there is no previous bone")]
    NoPrevTransformError(),
    #[error("bone tries to copy position from a previous bone, but there is no previous bone")]
    NoPrevPositionError(),
}

mod anm_bone;
pub use anm_bone::AnmBone;
