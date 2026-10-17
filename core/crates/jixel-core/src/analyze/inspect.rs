//! "What is in this file?", answered for people, not just for engineers.

use serde::Serialize;

use crate::codec::jxl::{self, JxlProbe};
use crate::convert::decode_any;
use crate::metadata::MetadataPolicy;
use crate::{Format, Result};

/// Everything Jixel can tell about a file.
#[derive(Clone, Debug, Serialize)]
pub struct Inspection {
    pub format: Format,
    pub file_bytes: usize,
    /// Displayed size (after EXIF orientation).
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    /// Significant bits per sample of the source (8, 10, 12, 16, 32…).
    pub bits_per_sample: u32,
    pub has_alpha: bool,
    /// Alpha channel present and actually used (not fully opaque).
    pub alpha_used: bool,
    pub frames: usize,
    pub has_icc: bool,
    pub has_exif: bool,
    pub has_xmp: bool,
    /// The file contains GPS coordinates (EXIF or XMP).
    pub has_gps: bool,
    pub orientation: u8,
    /// Bits per pixel: how much space each pixel costs.
    pub bits_per_pixel: f64,
    /// Present for JPEG XL files: the codec-level details.
    pub jxl: Option<JxlProbe>,
    /// Plain-language advice, e.g. "can be made 20% smaller without loss".
    pub suggestions: Vec<String>,
}

/// Inspects any supported file.
pub fn inspect(bytes: &[u8]) -> Result<Inspection> {
    let decoded = decode_any(bytes)?;
    let r = &decoded.raster;
    let (width, height) = if r.orientation > 4 {
        (r.height, r.width)
    } else {
        (r.width, r.height)
    };
    let jxl_probe = (decoded.format == Format::Jxl)
        .then(|| jxl::probe(bytes))
        .transpose()?;
    let has_gps = r.metadata.with_policy(MetadataPolicy::StripGps) != r.metadata;
    let alpha_used = r.alpha_is_used();
    let frames = decoded.animation.as_ref().map_or(1, |a| a.frames.len());

    let mut suggestions = Vec::new();
    match decoded.format {
        Format::Jpeg => suggestions.push(
            "Convert to JPEG XL with default settings: about 20% smaller with zero loss, and the original JPEG can be restored bit for bit.".into(),
        ),
        Format::Png | Format::Bmp | Format::Tiff | Format::Tga | Format::Pnm | Format::Qoi if frames == 1 => suggestions.push(
            "Lossless JPEG XL (quality 100) keeps every pixel and is usually 30–50% smaller than PNG.".into(),
        ),
        Format::Gif if frames > 1 => suggestions.push(
            "Animated JPEG XL keeps full colour (GIF is limited to 256) and is usually much smaller.".into(),
        ),
        Format::Webp => suggestions.push(
            "WebP is limited to 8 bits and 4:2:0 colour in lossy mode; JPEG XL at the same size keeps more detail.".into(),
        ),
        _ => {}
    }
    if has_gps {
        suggestions.push("This file contains the GPS location where it was taken. Jixel removes it by default when exporting.".into());
    }
    if r.has_alpha() && !alpha_used {
        suggestions.push("The transparency channel is fully opaque: dropping it saves space (Jixel does this automatically).".into());
    }
    if let Some(p) = &jxl_probe {
        if p.jpeg_reconstruction {
            suggestions.push("This JPEG XL contains a recompressed JPEG: it can be turned back into the identical original JPEG.".into());
        }
        if p.is_hdr() {
            suggestions.push("This is an HDR image. Keep it as JPEG XL or 16-bit PNG/TIFF to preserve the highlights.".into());
        }
    }
    if width.max(height) > 2560 {
        suggestions.push(format!("At {width}×{height} this image is larger than most screens: \"Optimize for web\" creates right-sized versions."));
    }

    Ok(Inspection {
        format: decoded.format,
        file_bytes: bytes.len(),
        width,
        height,
        channels: r.channels,
        bits_per_sample: jxl_probe
            .as_ref()
            .map_or(r.source_bits, |p| p.bits_per_sample),
        has_alpha: r.has_alpha(),
        alpha_used,
        frames,
        has_icc: r.icc.is_some(),
        has_exif: r.metadata.exif.is_some(),
        has_xmp: r.metadata.xmp.is_some(),
        has_gps,
        orientation: r.orientation,
        bits_per_pixel: bytes.len() as f64 * 8.0 / (width as f64 * height as f64),
        jxl: jxl_probe,
        suggestions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::raster::{encode, RasterEncodeOptions};
    use crate::test_support::photo;

    #[test]
    fn jpeg_gets_the_recompression_tip() {
        let jpeg = encode(
            &photo(64, 48, 3),
            Format::Jpeg,
            &RasterEncodeOptions::default(),
        )
        .unwrap();
        let i = inspect(&jpeg).unwrap();
        assert_eq!((i.format, i.width, i.height), (Format::Jpeg, 64, 48));
        assert!(i.suggestions[0].contains("JPEG XL"));
        assert!(i.jxl.is_none());
    }

    #[test]
    fn jxl_has_probe_details() {
        let f = jxl::encode(&photo(32, 32, 4), &Default::default()).unwrap();
        let i = inspect(&f).unwrap();
        assert!(i.jxl.is_some() && i.has_alpha && i.alpha_used);
    }
}
