//!
//! Contains the Error types for NEXRAD specific operations.
//!
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot decompress uncompressed data")]
    DecompressUnsupportedFile,

    #[error("unhandled product type encountered")]
    UnhandledProduct,
}
