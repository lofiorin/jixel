//! Objective quality measurement and format comparisons.
//!
//! Scores use [SSIMULACRA2](https://github.com/cloudinary/ssimulacra2), the
//! metric the JPEG XL team uses to tune libjxl. Rough scale:
//!
//! | Score | Meaning |
//! |---|---|
//! | 90+ | visually lossless, even when flipping between the two |
//! | 80  | very high quality: differences only on close inspection |
//! | 70  | high quality: typical of good web images |
//! | 50  | medium: visible artefacts |
//! | < 30 | low |

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use ssimulacra2::{compute_frame_ssimulacra2, LinearRgb};

use crate::convert::{decode_any, encode_raster, ConvertOptions};
use crate::metadata::MetadataPolicy;
use crate::ops::resize::{self, Fit};
use crate::{Error, Format, Raster, Result};

/// SSIMULACRA2 score of `distorted` against `reference` (same size).
///
/// Transparent images are scored over a white and over a black background,
/// and the lower score is returned: an artefact hidden by one background
/// still counts.
pub fn ssimulacra2(reference: &Raster, distorted: &Raster) -> Result<f64> {
    let (a, b) = (reference.to_rgba8()?, distorted.to_rgba8()?);
    if a.dimensions() != b.dimensions() {
        return Err(Error::InvalidInput(
            "images to compare must have the same size".into(),
        ));
    }
    let (w, h) = a.dimensions();
    if w < 8 || h < 8 {
        return Err(Error::InvalidInput(
            "images must be at least 8×8 to be scored".into(),
        ));
    }
    let has_alpha = reference.has_alpha() || distorted.has_alpha();
    let backgrounds: &[f32] = if has_alpha { &[1.0, 0.0] } else { &[1.0] };
    let mut worst = f64::INFINITY;
    for &bg in backgrounds {
        let score = compute_frame_ssimulacra2(linear(&a, bg)?, linear(&b, bg)?)
            .map_err(|e| Error::InvalidInput(format!("quality metric failed: {e}")))?;
        worst = worst.min(score);
    }
    Ok(worst)
}

/// sRGB 8-bit RGBA → linear RGB, composited over a grey level `bg` (0–1).
fn linear(img: &image::RgbaImage, bg: f32) -> Result<LinearRgb> {
    let lut: Vec<f32> = (0..256)
        .map(|v| {
            let c = v as f32 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    let data = img
        .pixels()
        .map(|p| {
            let a = p[3] as f32 / 255.0;
            [0, 1, 2].map(|i| lut[p[i] as usize] * a + bg * (1.0 - a))
        })
        .collect();
    LinearRgb::new(data, img.width() as usize, img.height() as usize)
        .map_err(|e| Error::InvalidInput(format!("quality metric failed: {e}")))
}

/// What the formats are asked to do in a [`battle`].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum BattleGoal {
    /// Same quality setting for every encoder. Simple, but settings are not
    /// comparable across formats.
    SameSetting { quality: f32 },
    /// Each format searches for the smallest file reaching this
    /// SSIMULACRA2 score. The fair comparison: same visual quality, who is
    /// smallest?
    SameScore { score: f64 },
}

impl Default for BattleGoal {
    fn default() -> Self {
        BattleGoal::SameScore { score: 80.0 }
    }
}

/// One contender's result.
#[derive(Clone, Debug, Serialize)]
pub struct BattleEntry {
    pub format: Format,
    /// Quality setting used (0–100).
    pub quality: f32,
    pub bytes: usize,
    /// `None` when the result cannot be decoded here (AVIF).
    pub score: Option<f64>,
    /// Bits per pixel.
    pub bpp: f64,
    /// Size relative to the JPEG entry, in percent (−40 = 40% smaller).
    pub vs_jpeg_percent: Option<f64>,
    /// The encoded file, so the app can show it.
    #[serde(skip)]
    pub file: Vec<u8>,
    pub note: Option<String>,
}

/// Result of [`battle`].
#[derive(Clone, Debug, Serialize)]
pub struct Battle {
    pub goal: BattleGoal,
    /// Size the image was scored at (large images are downscaled to ~2 MP
    /// to keep the comparison quick).
    pub width: u32,
    pub height: u32,
    pub entries: Vec<BattleEntry>,
}

/// The formats that take part in a battle.
pub const CONTENDERS: [Format; 4] = [Format::Jxl, Format::Avif, Format::Webp, Format::Jpeg];

/// Encodes `raster` in every contender format and measures size and quality.
pub fn battle(raster: &Raster, goal: BattleGoal, effort: u8) -> Result<Battle> {
    let mut image = raster.clone();
    image.apply_orientation()?;
    let pixels = image.pixel_count() as f64;
    const MAX_PIXELS: f64 = 2_000_000.0;
    if pixels > MAX_PIXELS {
        let s = (MAX_PIXELS / pixels).sqrt();
        image = resize::fit(&image, Fit::width((image.width as f64 * s) as u32))?;
    }
    image.drop_unused_alpha()?;
    let pixels = image.pixel_count() as f64;

    let mut entries: Vec<BattleEntry> = CONTENDERS
        .par_iter()
        .map(|&format| -> Result<BattleEntry> {
            let run = |quality: f32| -> Result<(Vec<u8>, Option<f64>)> {
                let opts = ConvertOptions {
                    quality,
                    effort,
                    metadata: MetadataPolicy::StripAll,
                    ..ConvertOptions::to(format)
                };
                let file = encode_raster(&image, &opts)?;
                let score = if format.can_decode() {
                    let back = decode_any(&file)?.raster;
                    Some(ssimulacra2(&image, &back)?)
                } else {
                    None
                };
                Ok((file, score))
            };
            let (quality, file, score, note) = match goal {
                BattleGoal::SameSetting { quality } => {
                    let (f, s) = run(quality)?;
                    (quality, f, s, None)
                }
                BattleGoal::SameScore { score: target } if format.can_decode() => {
                    let (q, f, s) = search_quality(target, &run)?;
                    let note = (s.unwrap_or(0.0) < target)
                        .then(|| "could not reach the target score".to_string());
                    (q, f, s, note)
                }
                BattleGoal::SameScore { .. } => {
                    // Without an AVIF decoder we cannot score AVIF; show it
                    // at a setting that typically lands near the target.
                    let (f, s) = run(80.0)?;
                    (
                        80.0,
                        f,
                        s,
                        Some(
                            "not scored: AVIF decoding is not available, shown at quality 80"
                                .into(),
                        ),
                    )
                }
            };
            Ok(BattleEntry {
                format,
                quality,
                bytes: file.len(),
                score,
                bpp: file.len() as f64 * 8.0 / pixels,
                vs_jpeg_percent: None,
                file,
                note,
            })
        })
        .collect::<Result<_>>()?;

    if let Some(jpeg) = entries
        .iter()
        .find(|e| e.format == Format::Jpeg)
        .map(|e| e.bytes as f64)
    {
        for e in &mut entries {
            e.vs_jpeg_percent = Some((e.bytes as f64 / jpeg - 1.0) * 100.0);
        }
    }
    entries.sort_by_key(|e| e.bytes);
    Ok(Battle {
        goal,
        width: image.width,
        height: image.height,
        entries,
    })
}

/// Binary search for the lowest quality setting whose score reaches
/// `target`. Returns the best attempt if none does.
fn search_quality(
    target: f64,
    run: &dyn Fn(f32) -> Result<(Vec<u8>, Option<f64>)>,
) -> Result<(f32, Vec<u8>, Option<f64>)> {
    let (mut lo, mut hi) = (5.0f32, 99.0f32);
    let mut best: Option<(f32, Vec<u8>, Option<f64>)> = None;
    for _ in 0..7 {
        let mid = ((lo + hi) / 2.0).round();
        let (file, score) = run(mid)?;
        if score.unwrap_or(0.0) >= target {
            hi = mid;
            best = Some((mid, file, score));
        } else {
            lo = mid;
        }
        if hi - lo <= 1.0 {
            break;
        }
    }
    match best {
        Some(b) => Ok(b),
        None => {
            let (file, score) = run(99.0)?;
            Ok((99.0, file, score))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::photo;

    #[test]
    fn identical_images_score_100() {
        let r = photo(64, 64, 3);
        assert!(ssimulacra2(&r, &r).unwrap() > 99.0);
    }

    #[test]
    fn lower_quality_scores_lower() {
        let r = photo(128, 128, 3);
        let score_at = |q| {
            let f = encode_raster(
                &r,
                &ConvertOptions {
                    quality: q,
                    effort: 3,
                    ..ConvertOptions::to(Format::Jxl)
                },
            )
            .unwrap();
            ssimulacra2(&r, &decode_any(&f).unwrap().raster).unwrap()
        };
        assert!(score_at(95.0) > score_at(30.0));
    }

    #[test]
    fn battle_has_every_contender_sorted_by_size() {
        let r = photo(128, 96, 3);
        let b = battle(&r, BattleGoal::SameScore { score: 70.0 }, 3).unwrap();
        assert_eq!(b.entries.len(), CONTENDERS.len());
        assert!(b.entries.windows(2).all(|w| w[0].bytes <= w[1].bytes));
        let jxl = b.entries.iter().find(|e| e.format == Format::Jxl).unwrap();
        assert!(jxl.score.unwrap() >= 70.0);
    }
}
