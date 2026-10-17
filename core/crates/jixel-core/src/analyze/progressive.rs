//! Progressive loading: what a visitor sees while the file downloads.
//!
//! A JPEG XL shows a complete, blurry picture as soon as its 1:8 "DC" image
//! has arrived (typically 10–20% of the file, less for smooth photos) and a
//! progressive one sharpens in several passes as more arrives. With a focus point the
//! subject sharpens first. [`preview`] decodes truncated copies of a file
//! to make that visible.

use serde::Serialize;

use crate::codec::jxl;
use crate::{Raster, Result};

/// Default download fractions shown by the app.
pub const DEFAULT_STEPS: [f32; 5] = [0.05, 0.15, 0.30, 0.50, 1.0];

/// The image after `fraction` of the file has arrived.
#[derive(Clone, Debug, Serialize)]
pub struct ProgressiveStep {
    pub fraction: f32,
    pub bytes: usize,
    /// `None` when nothing can be shown yet.
    #[serde(skip)]
    pub raster: Option<Raster>,
}

/// Decodes the first `fraction` of `jxl_bytes` for each fraction (0–1].
pub fn preview(jxl_bytes: &[u8], fractions: &[f32]) -> Result<Vec<ProgressiveStep>> {
    fractions
        .iter()
        .map(|&fraction| {
            let n = ((jxl_bytes.len() as f64 * fraction.clamp(0.0, 1.0) as f64) as usize)
                .min(jxl_bytes.len());
            let raster = if n == jxl_bytes.len() {
                Some(jxl::decode(jxl_bytes)?.into_first())
            } else {
                jxl::decode_prefix(&jxl_bytes[..n])?
            };
            Ok(ProgressiveStep {
                fraction,
                bytes: n,
                raster,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::jxl::JxlEncodeOptions;
    use crate::test_support::photo;

    #[test]
    fn progressive_file_shows_something_early() {
        let f = jxl::encode(
            &photo(512, 384, 3),
            &JxlEncodeOptions {
                progressive: true,
                effort: 3,
                ..Default::default()
            },
        )
        .unwrap();
        let steps = preview(&f, &DEFAULT_STEPS).unwrap();
        assert_eq!(steps.len(), 5);
        assert!(
            steps[2].raster.is_some(),
            "30% of a progressive file is viewable"
        );
        assert!(steps[4].raster.is_some());
    }
}
