//! The single error type used across the crate.

use crate::Format;

/// Everything that can go wrong in jixel-core.
///
/// Messages are written for end users: the app shows them as they are.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("This file type is not recognised or not supported.")]
    UnknownFormat,

    #[error("{0} files can be written but not read yet.")]
    CannotDecode(Format),

    #[error("{0} files can be read but not written.")]
    CannotEncode(Format),

    #[error("The image could not be decoded: {0}")]
    Decode(String),

    #[error("The image could not be encoded: {0}")]
    Encode(String),

    #[error("libjxl reported an error while {context} (code {code}).")]
    Libjxl { context: &'static str, code: i32 },

    #[error("Invalid parameter: {0}")]
    InvalidInput(String),

    #[error("This file was not created from a JPEG, so the original JPEG cannot be restored.")]
    NoJpegReconstruction,

    #[error("Background removal is not available in this build.")]
    BackgroundRemovalUnavailable,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        match e {
            image::ImageError::Encoding(e) => Error::Encode(e.to_string()),
            image::ImageError::Unsupported(e) => Error::Decode(e.to_string()),
            other => Error::Decode(other.to_string()),
        }
    }
}

/// Shorthand used by every fallible function in the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
