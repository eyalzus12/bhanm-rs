//! # bhanm-rs
//!
//! A Rust library for reading and writing Brawlhalla's anm files.
//!
//! # Organization
//!
//! This library exports the following:
//! * `AnmBone`: A sprite to be positioned in the world.
//! * `AnmFrame`: A single frame of animation.
//! * `AnmAnimation`: A complete animation.
//! * `AnmClass`: A collection of animations, indexed by their name.
//! * `AnmFile`: A collection of animation classes.

mod anm_objects;

// Re-exports
pub use anm_objects::*;
