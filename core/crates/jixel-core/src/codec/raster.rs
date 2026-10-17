//! Every format except JPEG XL: decode to a [`Raster`], encode from one.
//!
//! Decoding goes through the `image` crate. Encoding uses `image` for the
//! lossless/simple formats, libwebp (via the `webp` crate) for WebP, and
//! `ravif` (rav1e) for AVIF.

use std::io::Cursor;

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngDecoder, PngEncoder};
use image::codecs::webp::WebPDecoder;
use image::{
    AnimationDecoder, Delay, DynamicImage, Frame, ImageDecoder, ImageEncoder, ImageFormat,
    ImageReader,
};
use serde::{Deserialize, Serialize};

use crate::metadata::{Metadata, MetadataPolicy};
use crate::{Error, Format, Raster, Result};

/// Maps our format to the `image` crate's, for the formats it decodes.
fn image_format(format: Format) -> Option<ImageFormat> {
    Some(match format {
        Format::Jpeg => ImageFormat::Jpeg,
        Format::Png => ImageFormat::Png,
        Format::Webp => ImageFormat::WebP,
        Format::Gif => ImageFormat::Gif,
        Format::Tiff => ImageFormat::Tiff,
        Format::Bmp => ImageFormat::Bmp,
        Format::Qoi => ImageFormat::Qoi,
        Format::Ico => ImageFormat::Ico,
        Format::Tga => ImageFormat::Tga,
        Format::Pnm => ImageFormat::Pnm,
        Format::Jxl | Format::Avif | Format::Heic => return None,
    })
}

/// Decodes a still image (the first frame of an animation), with its colour
/// profile, orientation and metadata.
pub fn decode(bytes: &[u8], format: Format) -> Result<Raster> {
    let fmt = image_format(format).ok_or(Error::CannotDecode(format))?;
    let mut decoder = ImageReader::with_format(Cursor::new(bytes), fmt).into_decoder()?;
    // Metadata is optional: a broken EXIF block must not prevent decoding.
    let icc = decoder.icc_profile().ok().flatten();
    let exif = decoder
        .exif_metadata()
        .ok()
        .flatten()
        .and_then(|e| Metadata::from_exif_any(&e));
    let xmp = decoder.xmp_metadata().ok().flatten();
    let orientation = decoder.orientation().map(|o| o.to_exif()).unwrap_or(1);
    let img = DynamicImage::from_decoder(decoder)?;
    let mut raster = Raster::from_dynamic(img);
    raster.icc = icc;
    raster.orientation = orientation;
    raster.metadata = Metadata { exif, xmp };
    Ok(raster)
}

/// An animation: frames with their duration in milliseconds, plus the loop
/// count (0 = forever).
#[derive(Clone, Debug)]
pub struct Animation {
    pub frames: Vec<(Raster, u32)>,
    pub loops: u32,
}

/// Decodes all frames of an animated GIF, APNG or WebP. Returns `None` for
/// still images (including single-frame GIFs).
pub fn decode_animation(bytes: &[u8], format: Format) -> Result<Option<Animation>> {
    let frames = match format {
        Format::Gif => GifDecoder::new(Cursor::new(bytes))?
            .into_frames()
            .collect_frames()?,
        Format::Png => {
            let dec = PngDecoder::new(Cursor::new(bytes))?;
            if !dec.is_apng()? {
                return Ok(None);
            }
            dec.apng()?.into_frames().collect_frames()?
        }
        Format::Webp => {
            let dec = WebPDecoder::new(Cursor::new(bytes))?;
            if !dec.has_animation() {
                return Ok(None);
            }
            dec.into_frames().collect_frames()?
        }
        _ => return Ok(None),
    };
    if frames.len() < 2 {
        return Ok(None);
    }
    let frames = frames
        .into_iter()
        .map(|f| {
            let (num, den) = f.delay().numer_denom_ms();
            let ms = num.checked_div(den).unwrap_or(100);
            (
                Raster::from_dynamic(DynamicImage::ImageRgba8(f.into_buffer())),
                ms,
            )
        })
        .collect();
    Ok(Some(Animation { frames, loops: 0 }))
}

/// Options for non-JXL encoders.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RasterEncodeOptions {
    /// 1–100 for JPEG, WebP and AVIF. Ignored by lossless-only formats.
    pub quality: f32,
    /// WebP: lossless mode. Ignored by other formats.
    pub lossless: bool,
    /// AVIF: 1 (slowest, smallest) to 10 (fastest). WebP uses it inverted.
    pub speed: u8,
    pub metadata: MetadataPolicy,
    /// Colour that replaces transparency in formats without alpha (JPEG).
    pub matte: [u8; 3],
}

impl Default for RasterEncodeOptions {
    fn default() -> Self {
        Self {
            quality: 85.0,
            lossless: false,
            speed: 6,
            metadata: MetadataPolicy::default(),
            matte: [255, 255, 255],
        }
    }
}

/// Encodes a still image.
///
/// The raster's orientation is applied to the pixels first, because most
/// of these formats (or the apps that read them) ignore EXIF orientation.
pub fn encode(raster: &Raster, format: Format, opts: &RasterEncodeOptions) -> Result<Vec<u8>> {
    let mut raster = raster.clone();
    raster.apply_orientation()?;
    let metadata = raster.metadata.with_policy(opts.metadata);
    let quality = opts.quality.clamp(1.0, 100.0);
    let mut out = Vec::new();
    match format {
        Format::Png => {
            let mut enc =
                PngEncoder::new_with_quality(&mut out, CompressionType::Best, FilterType::Adaptive);
            if let Some(icc) = &raster.icc {
                let _ = enc.set_icc_profile(icc.clone());
            }
            if let Some(exif) = metadata.exif {
                let _ = enc.set_exif_metadata(exif);
            }
            raster.to_dynamic()?.write_with_encoder(enc)?;
        }
        Format::Jpeg => {
            let rgb = flatten(&raster, opts.matte)?;
            let mut enc = JpegEncoder::new_with_quality(&mut out, quality.round() as u8);
            if let Some(icc) = &raster.icc {
                let _ = enc.set_icc_profile(icc.clone());
            }
            if let Some(exif) = metadata.exif {
                let _ = enc.set_exif_metadata(exif);
            }
            enc.write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )?;
        }
        Format::Webp => out = encode_webp(&raster, quality, opts)?,
        Format::Avif => out = encode_avif(&raster, quality, opts)?,
        Format::Gif => raster
            .to_dynamic()?
            .into_rgba8()
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Gif)?,
        Format::Ico => {
            // ICO entries are at most 256×256.
            let mut img = raster.to_dynamic()?.into_rgba8();
            if img.width() > 256 || img.height() > 256 {
                img = DynamicImage::ImageRgba8(img)
                    .resize(256, 256, image::imageops::FilterType::Lanczos3)
                    .into_rgba8();
            }
            img.write_to(&mut Cursor::new(&mut out), ImageFormat::Ico)?;
        }
        Format::Tiff => raster
            .to_dynamic()?
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Tiff)?,
        Format::Bmp | Format::Qoi | Format::Tga => {
            let img = raster.to_dynamic()?;
            let img = if raster.has_alpha() {
                DynamicImage::ImageRgba8(img.into_rgba8())
            } else {
                DynamicImage::ImageRgb8(img.into_rgb8())
            };
            img.write_to(
                &mut Cursor::new(&mut out),
                image_format(format).expect("decodable format"),
            )?;
        }
        Format::Pnm => {
            let img = DynamicImage::ImageRgb8(raster.to_dynamic()?.into_rgb8());
            img.write_to(&mut Cursor::new(&mut out), ImageFormat::Pnm)?;
        }
        Format::Jxl | Format::Heic => return Err(Error::CannotEncode(format)),
    }
    Ok(out)
}

/// Encodes an animated GIF.
pub fn encode_gif_animation(frames: &[(Raster, u32)], loops: u32) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = GifEncoder::new_with_speed(&mut out, 10);
        enc.set_repeat(if loops == 0 {
            Repeat::Infinite
        } else {
            Repeat::Finite(loops.min(u16::MAX as u32) as u16)
        })?;
        for (raster, ms) in frames {
            let delay = Delay::from_numer_denom_ms(*ms, 1);
            enc.encode_frame(Frame::from_parts(raster.to_rgba8()?, 0, 0, delay))?;
        }
    }
    Ok(out)
}

/// Composites transparency over `matte` and returns 8-bit RGB.
fn flatten(raster: &Raster, matte: [u8; 3]) -> Result<image::RgbImage> {
    let img = raster.to_dynamic()?;
    if !raster.has_alpha() {
        return Ok(img.into_rgb8());
    }
    let rgba = img.into_rgba8();
    let mut rgb = image::RgbImage::new(rgba.width(), rgba.height());
    for (dst, src) in rgb.pixels_mut().zip(rgba.pixels()) {
        let a = src[3] as u32;
        for c in 0..3 {
            dst[c] = ((src[c] as u32 * a + matte[c] as u32 * (255 - a) + 127) / 255) as u8;
        }
    }
    Ok(rgb)
}

fn encode_webp(raster: &Raster, quality: f32, opts: &RasterEncodeOptions) -> Result<Vec<u8>> {
    let img = raster.to_dynamic()?;
    let (data, alpha) = if raster.has_alpha() {
        (img.into_rgba8().into_raw(), true)
    } else {
        (img.into_rgb8().into_raw(), false)
    };
    let enc = if alpha {
        webp::Encoder::from_rgba(&data, raster.width, raster.height)
    } else {
        webp::Encoder::from_rgb(&data, raster.width, raster.height)
    };
    let mut config =
        webp::WebPConfig::new().map_err(|_| Error::Encode("WebP is unavailable".into()))?;
    config.lossless = i32::from(opts.lossless);
    config.quality = if opts.lossless { 100.0 } else { quality };
    // libwebp "method": 0 = fastest … 6 = smallest. Map our speed onto it.
    config.method = (10 - opts.speed.clamp(1, 10) as i32) * 6 / 9;
    config.alpha_compression = i32::from(!opts.lossless);
    let mem = enc
        .encode_advanced(&config)
        .map_err(|e| Error::Encode(format!("WebP: {e:?}")))?;
    Ok(mem.to_vec())
}

fn encode_avif(raster: &Raster, quality: f32, opts: &RasterEncodeOptions) -> Result<Vec<u8>> {
    let rgba = raster.to_rgba8()?;
    let pixels: Vec<ravif::RGBA8> = rgba
        .pixels()
        .map(|p| ravif::RGBA8::new(p[0], p[1], p[2], p[3]))
        .collect();
    let encoded = ravif::Encoder::new()
        .with_quality(quality)
        .with_alpha_quality(quality)
        .with_speed(opts.speed.clamp(1, 10))
        .encode_rgba(ravif::Img::new(
            &pixels[..],
            rgba.width() as usize,
            rgba.height() as usize,
        ))
        .map_err(|e| Error::Encode(format!("AVIF: {e}")))?;
    Ok(encoded.avif_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> Raster {
        let data = (0..w * h)
            .flat_map(|i| [(i % w * 255 / w) as u8, (i / w * 255 / h) as u8, 128, 255])
            .collect();
        Raster::from_u8(w, h, 4, data)
    }

    #[test]
    fn every_encodable_format_round_trips_dimensions() {
        let r = gradient(40, 30);
        for format in Format::ALL
            .into_iter()
            .filter(|f| f.can_encode() && *f != Format::Jxl)
        {
            let bytes = encode(&r, format, &RasterEncodeOptions::default()).unwrap();
            if format != Format::Tga {
                // TGA has no signature to detect.
                assert_eq!(Format::detect(&bytes), Some(format), "{format}");
            }
            if format.can_decode() {
                let back = decode(&bytes, format).unwrap();
                assert_eq!((back.width, back.height), (40, 30), "{format}");
            }
        }
    }

    #[test]
    fn png_is_lossless() {
        let r = gradient(16, 16);
        let back = decode(
            &encode(&r, Format::Png, &RasterEncodeOptions::default()).unwrap(),
            Format::Png,
        )
        .unwrap();
        assert_eq!(back.to_rgba8().unwrap(), r.to_rgba8().unwrap());
    }

    #[test]
    fn jpeg_flattens_alpha_on_matte() {
        let r = Raster::from_u8(8, 8, 4, [0, 0, 0, 0].repeat(64));
        let back = decode(
            &encode(&r, Format::Jpeg, &RasterEncodeOptions::default()).unwrap(),
            Format::Jpeg,
        )
        .unwrap();
        assert!(
            back.to_rgba8().unwrap().pixels().all(|p| p[0] > 250),
            "transparent becomes white"
        );
    }

    #[test]
    fn animated_gif_round_trip() {
        let frames = vec![(gradient(10, 10), 100), (gradient(10, 10), 200)];
        let gif = encode_gif_animation(&frames, 0).unwrap();
        let anim = decode_animation(&gif, Format::Gif)
            .unwrap()
            .expect("animated");
        assert_eq!(anim.frames.len(), 2);
        assert_eq!(anim.frames[1].1, 200);
    }
}
