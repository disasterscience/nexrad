//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!

extern crate core;

pub mod decode;
pub mod decompress;
pub mod error;
pub mod file;
pub mod model;

#[cfg(feature = "download")]
pub mod download;

#[cfg(test)]
mod test;
