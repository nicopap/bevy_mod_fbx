//! FBX post processing.
//!
//! # Why not distribute my FBX files with my game?
//!
//! FBX is a format designed and made for Autodesk 3d software.
//! It is in no way appropriate for video game assets.
//!
//! FBX stores a lot of extraneous data you don't need in your game,
//! and don't want the players to know.
//! For example, FBX stores a lot of file paths.
//!
//! Furthermore, FBX's scene format is ~~bizantine~~ extremely flexible, and
//! doesn't map cleanly to bevy's scene representation. As a result, `bevy_mod_fbx`
//! dedicates a lot of memory and processing time just to convert a FBX scene
//! into a bevy scene.
//!
//! Since `0.11` bevy supports asset pre-processing. This module implements all
//! that is necessary to convert your FBX file into a very quick-to-load format
//! based on [TMF] and `texture-format-to-be-specified`.
