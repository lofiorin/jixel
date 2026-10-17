//! One call to convert any supported format into any other.
//!
//! [`convert`] picks the best path automatically and says which one it took
//! ([`Fidelity`]), so the UI never has to guess whether a result is lossless:
//!
//! | Input → output | Path | Fidelity |
//! |---|---|---|
//! | JPEG → JXL, no resize | lossless recompression | [`Fidelity::Reversible`] |
//! | recompressed JXL → JPEG, no resize | JPEG reconstruction | [`Fidelity::Reversible`] |
//! | anything → lossless JXL/PNG/WebP… | decode + lossless encode | [`Fidelity::PixelExact`] |
//! | anything → lossy | decode + lossy encode | [`Fidelity::Lossy`] |

use serde::{Deserialize, Serialize};

use crate::codec::jxl::{self, FocusPoint, JxlEncodeOptions, JxlQuality};
use crate::codec::raster::{self, Animation, RasterEncodeOptions};
use crate::metadata::MetadataPolicy;
use crate::ops::resize::{self, Fit};
use crate::{Error, Format, Raster, Result};

/// Conversion settings. Start from [`ConvertOptions::to`] and change only
/// what you need.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConvertOptions {
    pub target: Format,
    /// 0–100. 100 means lossless where the format allows it.
    pub quality: f32,
    /// JPEG → JXL: repack the JPEG losslessly instead of re-encoding.
    pub jpeg_recompression: bool,
    /// JXL → JPEG: restore the original JPEG when the file contains it.
    pub jpeg_reconstruction: bool,
    /// JPEG XL effort 1–10 (AVIF/WebP speed is derived from it).
    pub effort: u8,
    pub progressive: bool,
    /// Subject position for subject-first loading (JPEG XL only).
    pub focus: Option<FocusPoint>,
    pub resize: Option<Fit>,
    pub metadata: MetadataPolicy,
    /// Keep animations animated when the target supports it.
    pub keep_animation: bool,
    /// Background colour for formats without transparency.
    pub matte: [u8; 3],
}

impl ConvertOptions {
    /// Sensible defaults for converting to `target`.
    pub fn to(target: Format) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            target: Format::Jxl,
            quality: 90.0,
            jpeg_recompression: true,
            jpeg_reconstruction: true,
            effort: 7,
            progressive: false,
            focus: None,
            resize: None,
            metadata: MetadataPolicy::default(),
            keep_animation: true,
            matte: [255, 255, 255],
        }
    }
}

/// How faithful the output is to the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fidelity {
    /// The original file can be restored bit for bit (JPEG ⇄ JXL).
    Reversible,
    /// Every pixel is identical to the decoded input.
    PixelExact,
    /// Re-encoded with a lossy codec.
    Lossy,
}

/// Result of a conversion.
#[derive(Clone, Debug, Serialize)]
pub struct ConvertOutput {
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub format: Format,
    pub source_format: Format,
    pub width: u32,
    pub height: u32,
    pub frames: usize,
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub fidelity: Fidelity,
    /// Things the user should know, e.g. "transparency replaced with white".
    pub notes: Vec<String>,
}

impl ConvertOutput {
    /// Size change in percent: −35.0 means 35% smaller.
    pub fn saving_percent(&self) -> f64 {
        if self.input_bytes == 0 {
            return 0.0;
        }
        (self.output_bytes as f64 / self.input_bytes as f64 - 1.0) * 100.0
    }
}

/// A decoded input file.
#[derive(Clone, Debug)]
pub struct Decoded {
    pub format: Format,
    /// The still image, or the first frame of an animation.
    pub raster: Raster,
    pub animation: Option<Animation>,
}

/// Decodes any supported file, detecting the format from its content.
pub fn decode_any(bytes: &[u8]) -> Result<Decoded> {
    let format = Format::detect(bytes).ok_or(Error::UnknownFormat)?;
    if !format.can_decode() {
        return Err(Error::CannotDecode(format));
    }
    if format == Format::Jxl {
        let img = jxl::decode(bytes)?;
        let loops = img.loops;
        let animated = img.probe.is_animated && img.frames.len() > 1;
        let frames: Vec<(Raster, u32)> = img
            .frames
            .into_iter()
            .map(|f| (f.raster, f.duration_ms))
            .collect();
        let raster = frames[0].0.clone();
        let animation = animated.then_some(Animation { frames, loops });
        return Ok(Decoded {
            format,
            raster,
            animation,
        });
    }
    let raster = raster::decode(bytes, format)?;
    let animation = raster::decode_animation(bytes, format)?;
    Ok(Decoded {
        format,
        raster,
        animation,
    })
}

/// Converts `input` according to `opts`.
pub fn convert(input: &[u8], opts: &ConvertOptions) -> Result<ConvertOutput> {
    let source = Format::detect(input).ok_or(Error::UnknownFormat)?;
    if !opts.target.can_encode() {
        return Err(Error::CannotEncode(opts.target));
    }
    let output = |bytes: Vec<u8>, w, h, frames, fidelity, notes| ConvertOutput {
        output_bytes: bytes.len(),
        bytes,
        format: opts.target,
        source_format: source,
        width: w,
        height: h,
        frames,
        input_bytes: input.len(),
        fidelity,
        notes,
    };

    // Fast, reversible paths between JPEG and JPEG XL.
    if opts.resize.is_none() {
        if source == Format::Jpeg && opts.target == Format::Jxl && opts.jpeg_recompression {
            let jxl_opts = jxl_options(opts, JxlQuality::Lossless);
            if let Ok(bytes) = jxl::transcode_jpeg(input, &jxl_opts) {
                let p = jxl::probe(&bytes)?;
                return Ok(output(
                    bytes,
                    p.width,
                    p.height,
                    1,
                    Fidelity::Reversible,
                    vec![],
                ));
            }
            // Unusual JPEGs (e.g. arithmetic coded) fall through to re-encoding.
        }
        if source == Format::Jxl && opts.target == Format::Jpeg && opts.jpeg_reconstruction {
            if let Ok(bytes) = jxl::reconstruct_jpeg(input) {
                let p = jxl::probe(input)?;
                return Ok(output(
                    bytes,
                    p.width,
                    p.height,
                    1,
                    Fidelity::Reversible,
                    vec![],
                ));
            }
        }
    }

    let decoded = decode_any(input)?;
    let mut notes = Vec::new();
    let lossless = opts.quality >= 100.0 && opts.target.is_lossless_capable();
    let fidelity = if lossless && opts.resize.is_none() {
        Fidelity::PixelExact
    } else {
        Fidelity::Lossy
    };

    // Animations.
    if let Some(anim) = decoded.animation.filter(|_| opts.keep_animation) {
        let frames: Vec<(Raster, u32)> = match opts.resize {
            Some(fit) => anim
                .frames
                .iter()
                .map(|(r, d)| Ok((resize::fit(r, fit)?, *d)))
                .collect::<Result<_>>()?,
            None => anim.frames,
        };
        let (w, h, n) = (frames[0].0.width, frames[0].0.height, frames.len());
        match opts.target {
            Format::Jxl => {
                let bytes = jxl::encode_animation(
                    &frames,
                    anim.loops,
                    &jxl_options(opts, jxl_quality(opts)),
                )?;
                return Ok(output(bytes, w, h, n, fidelity, notes));
            }
            Format::Gif => {
                let bytes = raster::encode_gif_animation(&frames, anim.loops)?;
                return Ok(output(bytes, w, h, n, Fidelity::Lossy, notes));
            }
            _ => notes.push(format!(
                "{} output keeps only the first frame of the animation.",
                opts.target
            )),
        }
    }

    let mut image = decoded.raster;
    if let Some(fit) = opts.resize {
        image = resize::fit(&image, fit)?;
    }
    image.drop_unused_alpha()?;
    if image.has_alpha() && !opts.target.supports_alpha() {
        notes.push("Transparency was replaced with the background colour.".into());
    }
    if image.samples.bits() > 8
        && matches!(
            opts.target,
            Format::Jpeg | Format::Webp | Format::Avif | Format::Gif | Format::Bmp
        )
    {
        notes.push(format!(
            "{} stores 8 bits per channel: high bit depth was reduced.",
            opts.target
        ));
    }

    let bytes = encode_raster(&image, opts)?;
    // Report the size as displayed (orientations 5–8 swap width and height).
    let (w, h) = if image.orientation > 4 {
        (image.height, image.width)
    } else {
        (image.width, image.height)
    };
    let fidelity = if opts.target == Format::Jpeg || (!lossless && opts.target.is_lossy_capable()) {
        Fidelity::Lossy
    } else {
        fidelity
    };
    Ok(output(bytes, w, h, 1, fidelity, notes))
}

/// Encodes one image to `opts.target` (no special paths).
pub fn encode_raster(image: &Raster, opts: &ConvertOptions) -> Result<Vec<u8>> {
    if opts.target == Format::Jxl {
        return jxl::encode(image, &jxl_options(opts, jxl_quality(opts)));
    }
    let raster_opts = RasterEncodeOptions {
        quality: opts.quality,
        lossless: opts.quality >= 100.0,
        // Effort 1..10 (higher = slower) maps to speed 10..1.
        speed: 11 - opts.effort.clamp(1, 10),
        metadata: opts.metadata,
        matte: opts.matte,
    };
    raster::encode(image, opts.target, &raster_opts)
}

fn jxl_quality(opts: &ConvertOptions) -> JxlQuality {
    JxlQuality::from_percent(opts.quality)
}

fn jxl_options(opts: &ConvertOptions, quality: JxlQuality) -> JxlEncodeOptions {
    JxlEncodeOptions {
        quality,
        effort: opts.effort,
        progressive: opts.progressive,
        focus: opts.focus,
        decoding_speed: 0,
        metadata: opts.metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::raster::RasterEncodeOptions;

    fn sample_jpeg() -> Vec<u8> {
        let img = crate::test_support::photo(120, 80, 3);
        raster::encode(
            &img,
            Format::Jpeg,
            &RasterEncodeOptions {
                quality: 92.0,
                ..Default::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn jpeg_to_jxl_and_back_is_reversible() {
        let jpeg = sample_jpeg();
        let jxl = convert(&jpeg, &ConvertOptions::to(Format::Jxl)).unwrap();
        assert_eq!(jxl.fidelity, Fidelity::Reversible);
        assert!(
            jxl.saving_percent() < 0.0,
            "smaller: {}",
            jxl.saving_percent()
        );
        let back = convert(
            &jxl.bytes,
            &ConvertOptions {
                metadata: MetadataPolicy::Keep,
                ..ConvertOptions::to(Format::Jpeg)
            },
        )
        .unwrap();
        assert_eq!(back.fidelity, Fidelity::Reversible);
        assert_eq!(back.bytes, jpeg);
    }

    #[test]
    fn resizing_disables_the_reversible_path() {
        let opts = ConvertOptions {
            resize: Some(Fit::width(60)),
            ..ConvertOptions::to(Format::Jxl)
        };
        let out = convert(&sample_jpeg(), &opts).unwrap();
        assert_eq!(out.fidelity, Fidelity::Lossy);
        assert_eq!((out.width, out.height), (60, 40));
    }

    #[test]
    fn png_to_lossless_jxl_is_pixel_exact() {
        let img = crate::test_support::photo(50, 50, 4);
        let png = raster::encode(&img, Format::Png, &RasterEncodeOptions::default()).unwrap();
        let out = convert(
            &png,
            &ConvertOptions {
                quality: 100.0,
                ..ConvertOptions::to(Format::Jxl)
            },
        )
        .unwrap();
        assert_eq!(out.fidelity, Fidelity::PixelExact);
        let back = decode_any(&out.bytes).unwrap().raster;
        assert_eq!(back.samples, img.samples);
    }

    #[test]
    fn every_target_works_from_jxl() {
        let img = crate::test_support::photo(40, 40, 4);
        let src = jxl::encode(&img, &JxlEncodeOptions::default()).unwrap();
        for target in Format::ALL.into_iter().filter(|f| f.can_encode()) {
            let out = convert(
                &src,
                &ConvertOptions {
                    effort: 9,
                    ..ConvertOptions::to(target)
                },
            )
            .unwrap();
            assert!(!out.bytes.is_empty(), "{target}");
        }
    }

    #[test]
    fn animated_gif_becomes_animated_jxl() {
        let f = crate::test_support::photo(12, 12, 4);
        let gif = raster::encode_gif_animation(&[(f.clone(), 100), (f, 300)], 0).unwrap();
        let out = convert(
            &gif,
            &ConvertOptions {
                quality: 100.0,
                ..ConvertOptions::to(Format::Jxl)
            },
        )
        .unwrap();
        assert_eq!(out.frames, 2);
        let back = decode_any(&out.bytes).unwrap();
        assert_eq!(back.animation.unwrap().frames[1].1, 300);
    }

    #[test]
    fn unknown_input_is_a_clear_error() {
        assert!(matches!(
            convert(b"hello", &ConvertOptions::default()),
            Err(Error::UnknownFormat)
        ));
    }
}
