//! Cropping, with an explicit promise about quality.
//!
//! JPEG XL makes **pixel-exact** crops practical: the cropped area is
//! decoded once and stored losslessly, so its pixels are *identical* to what
//! you saw in the original, whatever the source format. The alternative is a
//! normal **re-encoded** crop, which is smaller but loses a little more.

use serde::{Deserialize, Serialize};

use super::Rect;
use crate::codec::jxl::{self, JxlEncodeOptions, JxlQuality};
use crate::convert::{decode_any, Fidelity};
use crate::metadata::MetadataPolicy;
use crate::{Error, Raster, Result, Samples};

/// How a crop is stored.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CropMode {
    /// Lossless JPEG XL of the decoded pixels: no additional loss.
    PixelExact,
    /// Lossy JPEG XL at this quality (0–100).
    Reencode { quality: f32 },
}

/// Cuts `rect` out of `raster`. Orientation is applied first, so `rect` is
/// in displayed coordinates.
pub fn crop(raster: &Raster, rect: Rect) -> Result<Raster> {
    let mut r = raster.clone();
    r.apply_orientation()?;
    if !rect.fits_in(r.width, r.height) {
        return Err(Error::InvalidInput(format!(
            "crop {}×{} at ({}, {}) is outside the {}×{} image",
            rect.width, rect.height, rect.x, rect.y, r.width, r.height
        )));
    }
    let c = r.channels as usize;
    let (w, x0, y0) = (r.width as usize, rect.x as usize, rect.y as usize);
    let (cw, ch) = (rect.width as usize, rect.height as usize);
    fn cut<T: Copy>(
        v: &[T],
        w: usize,
        c: usize,
        x0: usize,
        y0: usize,
        cw: usize,
        ch: usize,
    ) -> Vec<T> {
        let mut out = Vec::with_capacity(cw * ch * c);
        for y in y0..y0 + ch {
            let start = (y * w + x0) * c;
            out.extend_from_slice(&v[start..start + cw * c]);
        }
        out
    }
    r.samples = match &r.samples {
        Samples::U8(v) => Samples::U8(cut(v, w, c, x0, y0, cw, ch)),
        Samples::U16(v) => Samples::U16(cut(v, w, c, x0, y0, cw, ch)),
        Samples::F32(v) => Samples::F32(cut(v, w, c, x0, y0, cw, ch)),
    };
    r.width = rect.width;
    r.height = rect.height;
    Ok(r)
}

/// Crops an encoded file to JPEG XL and reports the resulting fidelity.
pub fn crop_file(
    input: &[u8],
    rect: Rect,
    mode: CropMode,
    metadata: MetadataPolicy,
) -> Result<(Vec<u8>, Fidelity)> {
    let cropped = crop(&decode_any(input)?.raster, rect)?;
    let (quality, fidelity) = match mode {
        CropMode::PixelExact => (JxlQuality::Lossless, Fidelity::PixelExact),
        CropMode::Reencode { quality } => (JxlQuality::from_percent(quality), Fidelity::Lossy),
    };
    let bytes = jxl::encode(
        &cropped,
        &JxlEncodeOptions {
            quality,
            metadata,
            ..Default::default()
        },
    )?;
    Ok((bytes, fidelity))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::photo;

    #[test]
    fn crop_takes_the_right_pixels() {
        let r = Raster::from_u8(3, 2, 1, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(
            crop(&r, Rect::new(1, 0, 2, 2)).unwrap().samples,
            Samples::U8(vec![2, 3, 5, 6])
        );
        assert!(crop(&r, Rect::new(2, 0, 2, 2)).is_err());
    }

    #[test]
    fn pixel_exact_crop_of_a_lossy_file_matches_the_decoded_source() {
        let lossy = jxl::encode(&photo(300, 200, 3), &JxlEncodeOptions::default()).unwrap();
        let rect = Rect::new(37, 21, 101, 77);
        let (bytes, fidelity) =
            crop_file(&lossy, rect, CropMode::PixelExact, MetadataPolicy::StripAll).unwrap();
        assert_eq!(fidelity, Fidelity::PixelExact);
        let expected = crop(&jxl::decode(&lossy).unwrap().into_first(), rect).unwrap();
        assert_eq!(
            jxl::decode(&bytes).unwrap().into_first().samples,
            expected.samples
        );
    }
}
