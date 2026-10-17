//! Safe JPEG XL encoding and decoding on top of libjxl.
//!
//! All `unsafe` code of the crate lives in this module. Each libjxl object is
//! owned by a small RAII wrapper ([`Runner`], `Encoder`, `Decoder`) whose
//! `Drop` frees it, so early returns with `?` never leak.
//!
//! ## Vocabulary
//!
//! * **Distance**: libjxl's quality knob, the Butteraugli distance. `0` is
//!   mathematically lossless, `1.0` is "visually lossless" for most people,
//!   `2–3` is good web quality. Users see a 0–100 quality instead; see
//!   [`JxlQuality::from_percent`].
//! * **Effort**: how hard the encoder searches (1 fast … 10 slowest). It
//!   changes size and encode time, never decode time.
//! * **Group**: JPEG XL splits an image into 256×256 groups that can be
//!   decoded independently and stored in any order. We store them starting
//!   from the subject ([`FocusPoint`]), so it sharpens first while loading.
//! * **Frame / layer**: one codestream can hold several frames. Frames with
//!   duration 0 are layers that the decoder composites into one image.

mod decode;
mod encode;

pub use decode::{decode, decode_prefix, probe, reconstruct_jpeg, JxlFrame, JxlImage, JxlProbe};
pub use encode::{encode, encode_animation, encode_layers, transcode_jpeg, Layer};

use std::ffi::c_void;

use jixel_libjxl_sys as sys;
use serde::{Deserialize, Serialize};

use crate::metadata::MetadataPolicy;

/// Raw libjxl version number, e.g. `12000` for 0.12.0.
pub fn libjxl_version_number() -> u32 {
    // SAFETY: pure function without arguments.
    unsafe { sys::JxlEncoderVersion() }
}

/// How much quality to keep.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum JxlQuality {
    /// Pixel-exact. Uses JPEG XL's modular mode.
    Lossless,
    /// Lossy (VarDCT) with a Butteraugli distance, `0.05..=25.0`.
    Distance { distance: f32 },
}

impl JxlQuality {
    /// Maps a familiar 0–100 "quality" to a distance, using libjxl's own
    /// mapping (the same one `cjxl -q` uses). 100 means lossless.
    pub fn from_percent(quality: f32) -> Self {
        if quality >= 100.0 {
            JxlQuality::Lossless
        } else {
            // SAFETY: pure function.
            let distance = unsafe { sys::JxlEncoderDistanceFromQuality(quality.max(0.0)) };
            JxlQuality::Distance { distance }
        }
    }

    /// The distance, with 0 for lossless.
    pub fn distance(self) -> f32 {
        match self {
            JxlQuality::Lossless => 0.0,
            JxlQuality::Distance { distance } => distance,
        }
    }
}

impl Default for JxlQuality {
    /// Distance 1.0: visually lossless, about quality 90.
    fn default() -> Self {
        JxlQuality::Distance { distance: 1.0 }
    }
}

/// A point in image pixels. The groups around it are stored (and therefore
/// decoded) first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusPoint {
    pub x: u32,
    pub y: u32,
}

/// Options for every JPEG XL encode.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JxlEncodeOptions {
    pub quality: JxlQuality,
    /// 1 (fastest) to 10 (smallest files). Default 7.
    pub effort: u8,
    /// Store extra passes so a partially downloaded file already shows a
    /// complete, gradually sharpening picture. Costs ~1–3% size.
    pub progressive: bool,
    /// Store groups starting from this point (subject-first loading).
    pub focus: Option<FocusPoint>,
    /// 0 (default, best density) to 4 (fastest decoding).
    pub decoding_speed: u8,
    /// What to do with EXIF / XMP.
    pub metadata: MetadataPolicy,
}

impl Default for JxlEncodeOptions {
    fn default() -> Self {
        Self {
            quality: JxlQuality::default(),
            effort: 7,
            progressive: false,
            focus: None,
            decoding_speed: 0,
            metadata: MetadataPolicy::default(),
        }
    }
}

/// libjxl's multi-threaded runner. One per encode/decode.
pub(crate) struct Runner(*mut c_void);

impl Runner {
    pub(crate) fn new() -> Self {
        // SAFETY: null memory manager = default allocator. Returns null only on
        // allocation failure, which libjxl treats as "no runner" (single thread).
        Runner(unsafe {
            sys::JxlThreadParallelRunnerCreate(
                std::ptr::null(),
                sys::JxlThreadParallelRunnerDefaultNumWorkerThreads(),
            )
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: created by JxlThreadParallelRunnerCreate, destroyed once.
            unsafe { sys::JxlThreadParallelRunnerDestroy(self.0) }
        }
    }
}

/// The four-character box type as libjxl wants it.
pub(crate) fn box_type(name: &[u8; 4]) -> [std::ffi::c_char; 4] {
    name.map(|b| b as std::ffi::c_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_mapping_matches_cjxl() {
        assert_eq!(JxlQuality::from_percent(100.0), JxlQuality::Lossless);
        assert!((JxlQuality::from_percent(90.0).distance() - 1.0).abs() < 0.01);
        assert!(JxlQuality::from_percent(50.0).distance() > 3.0);
    }
}

#[cfg(test)]
mod roundtrip_tests;
