//! [`Raster`]: the decoded, in-memory image that flows between modules.
//!
//! A raster is deliberately simple: interleaved samples, a channel count, a
//! sample type, and the colour/metadata information needed to write it back
//! out faithfully. High bit depth and floating point (HDR) data are kept as
//! they are; nothing is silently reduced to 8 bits.

use image::{DynamicImage, ImageBuffer};

use crate::metadata::Metadata;
use crate::{Error, Result};

/// Interleaved pixel samples. The variant is the sample type.
#[derive(Clone, Debug, PartialEq)]
pub enum Samples {
    /// 8-bit samples (0–255). The common case for photos on the web.
    U8(Vec<u8>),
    /// 16-bit samples (0–65535), e.g. 16-bit PNG or TIFF.
    U16(Vec<u16>),
    /// 32-bit float samples, nominally 0.0–1.0; may exceed 1.0 for HDR.
    F32(Vec<f32>),
}

impl Samples {
    /// Number of samples (not pixels).
    pub fn len(&self) -> usize {
        match self {
            Samples::U8(v) => v.len(),
            Samples::U16(v) => v.len(),
            Samples::F32(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Bits used by one sample in memory: 8, 16 or 32.
    pub fn bits(&self) -> u32 {
        match self {
            Samples::U8(_) => 8,
            Samples::U16(_) => 16,
            Samples::F32(_) => 32,
        }
    }
}

/// A decoded image.
#[derive(Clone, Debug, PartialEq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    /// 1 = gray, 2 = gray + alpha, 3 = RGB, 4 = RGBA.
    pub channels: u8,
    pub samples: Samples,
    /// Significant bits of the *source* (e.g. 10 or 12 for a 16-bit buffer
    /// holding 12-bit data). Used to tell the JXL encoder the true depth.
    pub source_bits: u32,
    /// Embedded ICC colour profile. `None` means sRGB.
    pub icc: Option<Vec<u8>>,
    /// EXIF orientation (1–8). 1 means "as stored".
    pub orientation: u8,
    /// EXIF / XMP metadata to carry over (or strip) when encoding.
    pub metadata: Metadata,
}

impl Raster {
    /// Creates an 8-bit raster. Panics if `data` does not match the size.
    pub fn from_u8(width: u32, height: u32, channels: u8, data: Vec<u8>) -> Self {
        assert_eq!(data.len(), width as usize * height as usize * channels as usize);
        Self {
            width,
            height,
            channels,
            samples: Samples::U8(data),
            source_bits: 8,
            icc: None,
            orientation: 1,
            metadata: Metadata::default(),
        }
    }

    pub fn has_alpha(&self) -> bool {
        self.channels == 2 || self.channels == 4
    }

    pub fn is_gray(&self) -> bool {
        self.channels <= 2
    }

    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// True when the alpha channel exists *and* at least one pixel is not
    /// fully opaque. Used to drop a useless alpha channel before encoding.
    pub fn alpha_is_used(&self) -> bool {
        if !self.has_alpha() {
            return false;
        }
        let c = self.channels as usize;
        match &self.samples {
            Samples::U8(v) => v.chunks_exact(c).any(|p| p[c - 1] != u8::MAX),
            Samples::U16(v) => v.chunks_exact(c).any(|p| p[c - 1] != u16::MAX),
            Samples::F32(v) => v.chunks_exact(c).any(|p| p[c - 1] < 1.0),
        }
    }

    /// Converts to an [`image::DynamicImage`], keeping bit depth.
    /// Colour profile, orientation and metadata are not part of the result.
    pub fn to_dynamic(&self) -> Result<DynamicImage> {
        let (w, h) = (self.width, self.height);
        let bad = || Error::InvalidInput("raster size does not match its samples".into());
        Ok(match (&self.samples, self.channels) {
            (Samples::U8(v), 1) => DynamicImage::ImageLuma8(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U8(v), 2) => DynamicImage::ImageLumaA8(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U8(v), 3) => DynamicImage::ImageRgb8(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U8(v), 4) => DynamicImage::ImageRgba8(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U16(v), 1) => DynamicImage::ImageLuma16(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U16(v), 2) => DynamicImage::ImageLumaA16(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U16(v), 3) => DynamicImage::ImageRgb16(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::U16(v), 4) => DynamicImage::ImageRgba16(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::F32(v), 3) => DynamicImage::ImageRgb32F(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            (Samples::F32(v), 4) => DynamicImage::ImageRgba32F(ImageBuffer::from_raw(w, h, v.clone()).ok_or_else(bad)?),
            // `image` has no float gray types: expand gray to RGB.
            (Samples::F32(v), 1) => {
                let rgb = v.iter().flat_map(|&g| [g, g, g]).collect();
                DynamicImage::ImageRgb32F(ImageBuffer::from_raw(w, h, rgb).ok_or_else(bad)?)
            }
            (Samples::F32(v), 2) => {
                let rgba = v.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect();
                DynamicImage::ImageRgba32F(ImageBuffer::from_raw(w, h, rgba).ok_or_else(bad)?)
            }
            _ => return Err(Error::InvalidInput(format!("unsupported channel count {}", self.channels))),
        })
    }

    /// Builds a raster from an [`image::DynamicImage`], keeping bit depth.
    pub fn from_dynamic(img: DynamicImage) -> Self {
        let (width, height) = (img.width(), img.height());
        let (channels, samples, bits) = match img {
            DynamicImage::ImageLuma8(b) => (1, Samples::U8(b.into_raw()), 8),
            DynamicImage::ImageLumaA8(b) => (2, Samples::U8(b.into_raw()), 8),
            DynamicImage::ImageRgb8(b) => (3, Samples::U8(b.into_raw()), 8),
            DynamicImage::ImageRgba8(b) => (4, Samples::U8(b.into_raw()), 8),
            DynamicImage::ImageLuma16(b) => (1, Samples::U16(b.into_raw()), 16),
            DynamicImage::ImageLumaA16(b) => (2, Samples::U16(b.into_raw()), 16),
            DynamicImage::ImageRgb16(b) => (3, Samples::U16(b.into_raw()), 16),
            DynamicImage::ImageRgba16(b) => (4, Samples::U16(b.into_raw()), 16),
            DynamicImage::ImageRgb32F(b) => (3, Samples::F32(b.into_raw()), 32),
            DynamicImage::ImageRgba32F(b) => (4, Samples::F32(b.into_raw()), 32),
            // `DynamicImage` is non-exhaustive: normalise anything new to RGBA8.
            other => (4, Samples::U8(other.into_rgba8().into_raw()), 8),
        };
        Self {
            width,
            height,
            channels,
            samples,
            source_bits: bits,
            icc: None,
            orientation: 1,
            metadata: Metadata::default(),
        }
    }

    /// Returns a copy with the same colour info and metadata but new pixels.
    pub(crate) fn with_pixels(&self, img: DynamicImage) -> Self {
        let mut out = Raster::from_dynamic(img);
        out.icc = self.icc.clone();
        out.metadata = self.metadata.clone();
        out.orientation = self.orientation;
        out.source_bits = self.source_bits.min(out.samples.bits());
        out
    }

    /// 8-bit RGBA copy, e.g. for display, AI models or 8-bit-only encoders.
    pub fn to_rgba8(&self) -> Result<image::RgbaImage> {
        Ok(self.to_dynamic()?.into_rgba8())
    }

    /// Rotates/flips the pixels so that orientation becomes 1, and resets the
    /// EXIF orientation tag to match. Formats without an orientation field
    /// (or viewers that ignore it) then show the image the right way up.
    pub fn apply_orientation(&mut self) -> Result<()> {
        if self.orientation <= 1 || self.orientation > 8 {
            self.orientation = 1;
            return Ok(());
        }
        let mut img = self.to_dynamic()?;
        if let Some(o) = image::metadata::Orientation::from_exif(self.orientation) {
            img.apply_orientation(o);
        }
        let mut out = self.with_pixels(img);
        out.orientation = 1;
        out.metadata.reset_exif_orientation();
        *self = out;
        Ok(())
    }

    /// Removes the alpha channel when every pixel is opaque. Saves bytes in
    /// every format and avoids "fake transparency" in exported files.
    pub fn drop_unused_alpha(&mut self) -> Result<()> {
        if !self.has_alpha() || self.alpha_is_used() {
            return Ok(());
        }
        let img = self.to_dynamic()?;
        let img = match (self.channels, &self.samples) {
            (2, Samples::U8(_)) => DynamicImage::ImageLuma8(img.into_luma8()),
            (2, Samples::U16(_)) => DynamicImage::ImageLuma16(img.into_luma16()),
            (_, Samples::U8(_)) => DynamicImage::ImageRgb8(img.into_rgb8()),
            (_, Samples::U16(_)) => DynamicImage::ImageRgb16(img.into_rgb16()),
            (_, Samples::F32(_)) => DynamicImage::ImageRgb32F(img.into_rgb32f()),
        };
        *self = self.with_pixels(img);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_round_trip_keeps_16_bit() {
        let data: Vec<u16> = (0..2 * 2 * 3).map(|i| i * 1000).collect();
        let img = DynamicImage::ImageRgb16(ImageBuffer::from_raw(2, 2, data.clone()).unwrap());
        let r = Raster::from_dynamic(img);
        assert_eq!(r.samples, Samples::U16(data));
        assert_eq!(r.to_dynamic().unwrap().color(), image::ColorType::Rgb16);
    }

    #[test]
    fn opaque_alpha_is_dropped() {
        let mut r = Raster::from_u8(1, 2, 4, vec![1, 2, 3, 255, 4, 5, 6, 255]);
        r.drop_unused_alpha().unwrap();
        assert_eq!(r.channels, 3);
        assert_eq!(r.samples, Samples::U8(vec![1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn real_alpha_is_kept() {
        let mut r = Raster::from_u8(1, 1, 4, vec![1, 2, 3, 10]);
        r.drop_unused_alpha().unwrap();
        assert_eq!(r.channels, 4);
    }
}
