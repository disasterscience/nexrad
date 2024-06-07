//!
//! Contains the Error types for NEXRAD specific operations.
//!
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NexradError {
    #[error("cannot decompress uncompressed data")]
    DecompressUnsupportedFile,
}
