//! The main subject of a photo, and two ways JPEG XL can favour it.
//!
//! 1. **Subject-first loading** ([`FocusPoint`]): the 256×256 groups around
//!    the subject are stored first, so on a slow connection the face or the
//!    product sharpens before the background. Free: the file size is the same.
//! 2. **Subject boost** ([`encode_with_subject`]): the image is stored as two
//!    layers that the decoder composites into one picture. The background
//!    layer uses a lower quality; the subject layer, on top, a higher one.
//!    The area hidden under the subject is flattened in the background
//!    layer, so it costs almost nothing.
//!
//! Subject boost is measured, not assumed: both variants are encoded and
//! scored with SSIMULACRA2, and [`BoostResult`] reports the numbers, so the
//! app can show the user exactly what they gain.

use serde::{Deserialize, Serialize};

use super::crop::crop;
use super::resize::resize_exact;
use super::Rect;
use crate::analyze::compare::ssimulacra2;
use crate::codec::jxl::{self, FocusPoint, JxlEncodeOptions, JxlQuality, Layer};
use crate::metadata::MetadataPolicy;
use crate::{Error, Raster, Result};

/// Where the subject information came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectSource {
    /// Found by the on-device model.
    Detected,
    /// Chosen by the user.
    Manual,
    /// No subject found (or model unavailable): the image centre.
    Center,
}

/// The main subject of an image, in displayed pixel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    pub bbox: Rect,
    /// Centre of mass of the subject: the best point to load first.
    pub focus: FocusPoint,
    /// Fraction of the image covered by the subject (0–1).
    pub coverage: f32,
    pub source: SubjectSource,
}

impl Subject {
    /// A user-drawn subject rectangle.
    pub fn manual(rect: Rect, width: u32, height: u32) -> Result<Subject> {
        if !rect.fits_in(width, height) {
            return Err(Error::InvalidInput(
                "the subject rectangle is outside the image".into(),
            ));
        }
        Ok(Subject {
            bbox: rect,
            focus: FocusPoint {
                x: rect.x + rect.width / 2,
                y: rect.y + rect.height / 2,
            },
            coverage: rect.area() as f32 / (width as f32 * height as f32),
            source: SubjectSource::Manual,
        })
    }

    /// The whole image, focused on its centre.
    pub fn center(width: u32, height: u32) -> Subject {
        Subject {
            bbox: Rect::new(0, 0, width, height),
            focus: FocusPoint {
                x: width / 2,
                y: height / 2,
            },
            coverage: 1.0,
            source: SubjectSource::Center,
        }
    }

    /// Derives the subject from a mask (values ≥ `threshold` are subject).
    pub fn from_mask(mask: &super::background::Mask, threshold: u8) -> Option<Subject> {
        let (w, h) = (mask.width, mask.height);
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
        let (mut sum, mut sx, mut sy, mut count) = (0f64, 0f64, 0f64, 0u64);
        for (i, &m) in mask.data.iter().enumerate() {
            if m >= threshold {
                let (x, y) = (i as u32 % w, i as u32 / w);
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
                count += 1;
                let weight = m as f64;
                sum += weight;
                sx += x as f64 * weight;
                sy += y as f64 * weight;
            }
        }
        // Ignore specks: a subject must cover at least 0.5% of the image.
        if count < (w as u64 * h as u64) / 200 || sum == 0.0 {
            return None;
        }
        Some(Subject {
            bbox: Rect::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1),
            focus: FocusPoint {
                x: (sx / sum) as u32,
                y: (sy / sum) as u32,
            },
            coverage: count as f32 / (w as f32 * h as f32),
            source: SubjectSource::Detected,
        })
    }
}

/// Finds the main subject. Falls back to the image centre when nothing
/// stands out or when background removal is not compiled in.
pub fn detect(raster: &Raster) -> Subject {
    let (w, h) = if raster.orientation > 4 {
        (raster.height, raster.width)
    } else {
        (raster.width, raster.height)
    };
    super::background::subject_mask(raster, false)
        .ok()
        .and_then(|m| Subject::from_mask(&m, 128))
        .unwrap_or_else(|| Subject::center(w, h))
}

/// Settings for [`encode_with_subject`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BoostOptions {
    /// Baseline quality (0–99): what a normal encode would use.
    pub quality: f32,
    /// Subject distance = baseline distance × this (lower = sharper subject).
    pub subject_factor: f32,
    /// Background distance = baseline distance × this (higher = smaller file).
    pub background_factor: f32,
    pub effort: u8,
    pub progressive: bool,
    pub metadata: MetadataPolicy,
}

impl Default for BoostOptions {
    fn default() -> Self {
        Self {
            quality: 85.0,
            subject_factor: 0.7,
            background_factor: 1.8,
            effort: 7,
            progressive: true,
            metadata: MetadataPolicy::default(),
        }
    }
}

/// The two candidates and their scores.
#[derive(Clone, Debug, Serialize)]
pub struct BoostResult {
    /// Single-layer encode at the baseline quality, subject-first order.
    #[serde(skip)]
    pub single: Vec<u8>,
    /// Two-layer encode, or `None` when the subject covers too much of the
    /// image for layering to make sense.
    #[serde(skip)]
    pub boosted: Option<Vec<u8>>,
    pub single_bytes: usize,
    pub boosted_bytes: Option<usize>,
    /// SSIMULACRA2 (higher is better; 90 ≈ visually lossless) of the subject
    /// area and of the whole image, for each candidate.
    pub single_subject_score: f64,
    pub single_full_score: f64,
    pub boosted_subject_score: Option<f64>,
    pub boosted_full_score: Option<f64>,
    /// The layer rectangle actually used.
    pub layer_rect: Option<Rect>,
    /// True when the boosted file is not larger and its subject is at least
    /// as good: the recommendation the app pre-selects.
    pub recommend_boost: bool,
}

impl BoostResult {
    /// The recommended file.
    pub fn best(&self) -> &[u8] {
        match (&self.boosted, self.recommend_boost) {
            (Some(b), true) => b,
            _ => &self.single,
        }
    }
}

/// Encodes with subject-first loading and, when it pays off, subject boost.
pub fn encode_with_subject(
    raster: &Raster,
    subject: &Subject,
    opts: &BoostOptions,
) -> Result<BoostResult> {
    let mut image = raster.clone();
    image.apply_orientation()?;
    image.drop_unused_alpha()?;
    let (w, h) = (image.width, image.height);
    let distance = JxlQuality::from_percent(opts.quality.min(99.0)).distance();
    let base_opts = JxlEncodeOptions {
        quality: JxlQuality::Distance { distance },
        effort: opts.effort,
        progressive: opts.progressive,
        focus: Some(subject.focus),
        decoding_speed: 0,
        metadata: opts.metadata,
    };
    let single = jxl::encode(&image, &base_opts)?;
    let single_decoded = jxl::decode(&single)?.into_first();
    let score_rect = subject.bbox;
    let subject_score = |decoded: &Raster| -> Result<f64> {
        ssimulacra2(&crop(&image, score_rect)?, &crop(decoded, score_rect)?)
    };
    let mut result = BoostResult {
        single_bytes: single.len(),
        single_subject_score: subject_score(&single_decoded)?,
        single_full_score: ssimulacra2(&image, &single_decoded)?,
        single,
        boosted: None,
        boosted_bytes: None,
        boosted_subject_score: None,
        boosted_full_score: None,
        layer_rect: None,
        recommend_boost: false,
    };

    // Layering only helps when there is a real background to save on.
    let rect = subject.bbox.expand(16, w, h).align_outwards(8, w, h);
    if rect.area() as f64 > 0.7 * (w as f64 * h as f64) || rect.width < 16 || rect.height < 16 {
        return Ok(result);
    }

    let background = flatten_inside(&image, rect)?;
    let layer = Layer {
        raster: crop(&image, rect)?,
        x: rect.x as i32,
        y: rect.y as i32,
        quality: JxlQuality::Distance {
            distance: (distance * opts.subject_factor).max(0.1),
        },
    };
    let bg_opts = JxlEncodeOptions {
        quality: JxlQuality::Distance {
            distance: (distance * opts.background_factor).min(15.0),
        },
        ..base_opts
    };
    let boosted = jxl::encode_layers(&background, &[layer], &bg_opts)?;
    let boosted_decoded = jxl::decode(&boosted)?.into_first();
    let b_subject = subject_score(&boosted_decoded)?;
    result.recommend_boost =
        boosted.len() <= result.single_bytes && b_subject >= result.single_subject_score;
    result.boosted_bytes = Some(boosted.len());
    result.boosted_subject_score = Some(b_subject);
    result.boosted_full_score = Some(ssimulacra2(&image, &boosted_decoded)?);
    result.layer_rect = Some(rect);
    result.boosted = Some(boosted);
    Ok(result)
}

/// Returns a copy where the inside of `rect` (minus a 16 px border, which
/// stays real so the layer seam is invisible) is replaced by a very blurred
/// version of itself. That area is hidden by the subject layer; smooth
/// content there costs almost no bytes.
fn flatten_inside(image: &Raster, rect: Rect) -> Result<Raster> {
    let inner = Rect::new(
        rect.x + 16,
        rect.y + 16,
        rect.width.saturating_sub(32),
        rect.height.saturating_sub(32),
    );
    if inner.width < 8 || inner.height < 8 {
        return Ok(image.clone());
    }
    let region = crop(image, inner)?;
    let tiny = resize_exact(
        &region,
        (inner.width / 32).max(1),
        (inner.height / 32).max(1),
    )?;
    let blurred = resize_exact(&tiny, inner.width, inner.height)?;
    let mut out = image.clone();
    paste(&mut out, &blurred, inner.x, inner.y);
    Ok(out)
}

/// Copies `src` into `dst` at (x, y). Both must share the pixel layout.
fn paste(dst: &mut Raster, src: &Raster, x: u32, y: u32) {
    use crate::Samples::*;
    let c = dst.channels as usize;
    let (dw, sw) = (dst.width as usize, src.width as usize);
    macro_rules! copy_rows {
        ($d:expr, $s:expr) => {
            for row in 0..src.height as usize {
                let d0 = ((y as usize + row) * dw + x as usize) * c;
                let s0 = row * sw * c;
                $d[d0..d0 + sw * c].copy_from_slice(&$s[s0..s0 + sw * c]);
            }
        };
    }
    match (&mut dst.samples, &src.samples) {
        (U8(d), U8(s)) => copy_rows!(d, s),
        (U16(d), U16(s)) => copy_rows!(d, s),
        (F32(d), F32(s)) => copy_rows!(d, s),
        _ => unreachable!("paste requires matching sample types"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::background::Mask;
    use crate::test_support::photo;

    #[test]
    fn subject_from_mask_finds_box_and_centre() {
        let (w, h) = (100, 80);
        let data = (0..w * h)
            .map(|i| {
                if (20..40).contains(&(i % w)) && (10..30).contains(&(i / w)) {
                    255
                } else {
                    0
                }
            })
            .collect();
        let s = Subject::from_mask(
            &Mask {
                width: w,
                height: h,
                data,
            },
            128,
        )
        .unwrap();
        assert_eq!(s.bbox, Rect::new(20, 10, 20, 20));
        assert_eq!((s.focus.x, s.focus.y), (29, 19));
        assert!((s.coverage - 0.05).abs() < 1e-3);
    }

    #[test]
    fn empty_mask_has_no_subject() {
        assert!(Subject::from_mask(
            &Mask {
                width: 10,
                height: 10,
                data: vec![0; 100]
            },
            128
        )
        .is_none());
    }

    #[test]
    fn boost_produces_a_sharper_subject() {
        let img = photo(640, 480, 3);
        let subject = Subject::manual(Rect::new(250, 180, 120, 100), 640, 480).unwrap();
        let r = encode_with_subject(
            &img,
            &subject,
            &BoostOptions {
                effort: 3,
                ..Default::default()
            },
        )
        .unwrap();
        let boosted_score = r.boosted_subject_score.expect("boost attempted");
        assert!(
            boosted_score > r.single_subject_score,
            "{boosted_score} vs {}",
            r.single_subject_score
        );
        // The composited file decodes as one normal image.
        let d = jxl::decode(r.boosted.as_ref().unwrap()).unwrap();
        assert_eq!((d.frames.len(), d.probe.width), (1, 640));
    }
}
