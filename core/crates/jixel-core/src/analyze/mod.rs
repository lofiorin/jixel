//! Looking inside images and measuring quality.
//!
//! * [`inspect`]: what is in a file (size, depth, colour, metadata, JPEG XL
//!   features) plus plain-language suggestions.
//! * [`compare`]: SSIMULACRA2 scores and the "format battle": JPEG XL
//!   against AVIF, WebP and JPEG on the same picture.
//! * [`progressive`]: what a visitor sees after 5%, 15%, 30%… of the file.

pub mod compare;
pub mod inspect;
pub mod progressive;
