#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!
pub mod decode;
pub mod decompress;
pub mod error;
pub mod file_metadata;
pub mod model;

// Expose more useful things
pub use decode::DataFile;
pub use model::Product;

#[cfg(feature = "download")]
pub mod download;

#[cfg(test)]
mod test;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
