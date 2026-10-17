//! Encoders and decoders.
//!
//! * [`jxl`]: JPEG XL, through libjxl. This is where the format's special
//!   features (lossless JPEG recompression, subject-first group order,
//!   layers, progressive decoding, HDR) are exposed.
//! * [`raster`]: every other format, through pure-Rust crates plus libwebp.

pub mod jxl;
pub mod raster;
