//! # jixel-core
//!
//! The image engine behind the Jixel app and the `jixel` CLI.
//!
//! Everything a user can do in Jixel is a plain function in this crate, so it
//! can be tested without a UI and reused from any front end.
//!
//! ## Map of the crate
//!
//! | Module | What it does |
//! |---|---|
//! | [`raster`] | [`Raster`]: the in-memory image every module passes around. |
//! | [`format`] | [`Format`]: file-format detection and capabilities. |
//! | [`codec::jxl`] | Safe JPEG XL encoder/decoder on top of libjxl. |
//! | [`codec::raster`] | Every other format (PNG, JPEG, WebP, AVIF, GIF…). |
//! | [`convert`] | One-call conversion between any two formats. |
//! | [`ops`] | Resize, crop, tiles/Deep Zoom, subject focus, background removal. |
//! | [`analyze`] | Inspect files, compare formats (SSIMULACRA2), progressive preview. |
//! | [`web`] | Responsive web export, HTML/Next.js/WordPress snippets, server configs. |
//!
//! ## A first example
//!
//! ```no_run
//! use jixel_core::convert::{convert, ConvertOptions};
//! use jixel_core::Format;
//!
//! let jpeg = std::fs::read("photo.jpg")?;
//! // JPEG -> JPEG XL with default options is *lossless recompression*:
//! // about 20% smaller and the original JPEG can be restored bit for bit.
//! let out = convert(&jpeg, &ConvertOptions::to(Format::Jxl))?;
//! std::fs::write("photo.jxl", &out.bytes)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod analyze;
pub mod codec;
pub mod convert;
pub mod error;
pub mod format;
pub mod metadata;
pub mod ops;
pub mod raster;
pub mod web;

pub use error::{Error, Result};
pub use format::Format;
pub use raster::{Raster, Samples};

/// Version of libjxl this build is linked against, e.g. `"0.12.0"`.
pub fn libjxl_version() -> String {
    let v = codec::jxl::libjxl_version_number();
    format!("{}.{}.{}", v / 1_000_000, (v / 1000) % 1000, v % 1000)
}
