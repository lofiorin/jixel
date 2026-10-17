//! High-quality resizing.

use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::{Error, Raster, Result};

/// "Fit inside this box", keeping the aspect ratio. Never enlarges.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fit {
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Fit {
    pub fn width(w: u32) -> Self {
        Fit {
            max_width: Some(w),
            max_height: None,
        }
    }

    pub fn within(w: u32, h: u32) -> Self {
        Fit {
            max_width: Some(w),
            max_height: Some(h),
        }
    }

    /// Target size for a `width × height` source.
    pub fn target(&self, width: u32, height: u32) -> (u32, u32) {
        let sx = self.max_width.map_or(1.0, |m| m as f64 / width as f64);
        let sy = self.max_height.map_or(1.0, |m| m as f64 / height as f64);
        let s = sx.min(sy).min(1.0);
        (
            ((width as f64 * s).round() as u32).max(1),
            ((height as f64 * s).round() as u32).max(1),
        )
    }
}

/// Resizes to fit `fit`. Orientation is applied first, so `fit` refers to
/// the image as displayed.
pub fn fit(raster: &Raster, fit: Fit) -> Result<Raster> {
    let mut r = raster.clone();
    r.apply_orientation()?;
    let (w, h) = fit.target(r.width, r.height);
    if (w, h) == (r.width, r.height) {
        return Ok(r);
    }
    resize_exact(&r, w, h)
}

/// Resizes to exactly `width × height` with Lanczos3 (alpha-aware: colours
/// of transparent pixels do not bleed into visible ones).
pub fn resize_exact(raster: &Raster, width: u32, height: u32) -> Result<Raster> {
    if width == 0 || height == 0 {
        return Err(Error::InvalidInput("size must be at least 1×1".into()));
    }
    let src = raster.to_dynamic()?;
    let mut dst = DynamicImage::new(width, height, src.color());
    let options = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3));
    Resizer::new()
        .resize(&src, &mut dst, &options)
        .map_err(|e| Error::InvalidInput(format!("resize failed: {e}")))?;
    Ok(raster.with_pixels(dst))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::photo;

    #[test]
    fn fit_keeps_aspect_and_never_enlarges() {
        assert_eq!(Fit::width(100).target(400, 200), (100, 50));
        assert_eq!(Fit::within(100, 100).target(400, 200), (100, 50));
        assert_eq!(Fit::width(1000).target(400, 200), (400, 200));
    }

    #[test]
    fn resize_works_for_all_sample_types() {
        let r = photo(64, 32, 4);
        let out = fit(&r, Fit::width(16)).unwrap();
        assert_eq!((out.width, out.height, out.channels), (16, 8, 4));
        let mut r16 = r.clone();
        r16.samples = crate::Samples::U16(vec![1000; 64 * 32 * 4]);
        assert_eq!(resize_exact(&r16, 8, 8).unwrap().samples.bits(), 16);
    }
}
