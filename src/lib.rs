//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!

extern crate core;

pub mod decode;
pub mod decompress;
pub mod file;
pub mod model;
pub mod error;

#[cfg(feature = "download")]
pub mod download;
