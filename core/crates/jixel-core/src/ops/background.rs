//! On-device background removal.
//!
//! Uses **U²-Netp** (Qin et al., 2020, Apache-2.0), the 4.6 MB "portable"
//! variant of U²-Net, run with `tract`, a pure-Rust inference engine. No
//! native AI runtime is needed and photos never leave the device, so it works
//! the same on a phone, a laptop or a server.
//!
//! The network predicts a 320×320 saliency mask. A **guided filter** then
//! re-aligns that coarse mask with the edges of the full-resolution photo,
//! which recovers much sharper outlines (hair, fingers) than plain upscaling.

use serde::{Deserialize, Serialize};

use crate::{Raster, Result};

/// A single-channel mask with values 0 (background) … 255 (subject).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Background removal settings.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BackgroundOptions {
    /// Align the mask to the photo's edges (recommended).
    pub refine_edges: bool,
    /// Make the cut-out fully opaque/transparent instead of soft-edged.
    pub hard_edges: bool,
    /// Replace the background with this colour (RGB) instead of making it
    /// transparent. Use e.g. white for e-commerce shots.
    pub fill: Option<[u8; 3]>,
}

impl Default for BackgroundOptions {
    fn default() -> Self {
        Self {
            refine_edges: true,
            hard_edges: false,
            fill: None,
        }
    }
}

/// Whether this build includes the model.
pub const fn is_available() -> bool {
    cfg!(feature = "background-removal")
}

/// Predicts the subject mask at the raster's displayed size.
pub fn subject_mask(raster: &Raster, refine_edges: bool) -> Result<Mask> {
    let mut r = raster.clone();
    r.apply_orientation()?;
    let rgba = r.to_rgba8()?;
    let coarse = model::predict(&rgba)?;
    let (w, h) = rgba.dimensions();
    let mut mask = upscale(&coarse, model::SIZE, model::SIZE, w, h);
    if refine_edges {
        let guide: Vec<f32> = rgba
            .pixels()
            .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
            .collect();
        let radius = (w.min(h) / 160).clamp(2, 24) as usize;
        mask = guided_filter(&guide, &mask, w as usize, h as usize, radius, 1e-3);
    }
    // Gentle contrast curve: removes the faint halo U²-Net leaves around
    // subjects while keeping soft edges soft.
    let data = mask
        .iter()
        .map(|&m| (((m - 0.1) / 0.8).clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();
    Ok(Mask {
        width: w,
        height: h,
        data,
    })
}

/// Removes (or replaces) the background. The result is RGBA unless `fill`
/// is set, in which case it is opaque RGB.
pub fn remove_background(raster: &Raster, opts: &BackgroundOptions) -> Result<Raster> {
    let mask = subject_mask(raster, opts.refine_edges)?;
    let mut r = raster.clone();
    r.apply_orientation()?;
    let mut rgba = r.to_rgba8()?;
    for (p, &m) in rgba.pixels_mut().zip(&mask.data) {
        let m = if opts.hard_edges {
            if m >= 128 {
                255
            } else {
                0
            }
        } else {
            m
        };
        p[3] = ((p[3] as u32 * m as u32 + 127) / 255) as u8;
    }
    let mut out = r.with_pixels(image::DynamicImage::ImageRgba8(rgba));
    if let Some(fill) = opts.fill {
        let img = out.to_rgba8()?;
        let rgb: Vec<u8> = img
            .pixels()
            .flat_map(|p| {
                // Composite the cut-out over the fill colour.
                let a = p[3] as u32;
                let blend =
                    |c: u8, f: u8| ((c as u32 * a + f as u32 * (255 - a) + 127) / 255) as u8;
                [
                    blend(p[0], fill[0]),
                    blend(p[1], fill[1]),
                    blend(p[2], fill[2]),
                ]
            })
            .collect();
        out = r.with_pixels(image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(img.width(), img.height(), rgb).expect("sizes match"),
        ));
    }
    Ok(out)
}

/// Bilinear upscaling of a float mask.
fn upscale(src: &[f32], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<f32> {
    let mut out = Vec::with_capacity(dw as usize * dh as usize);
    for y in 0..dh {
        let fy = ((y as f32 + 0.5) * sh as f32 / dh as f32 - 0.5).clamp(0.0, (sh - 1) as f32);
        let (y0, ty) = (fy.floor() as usize, fy.fract());
        let y1 = (y0 + 1).min(sh as usize - 1);
        for x in 0..dw {
            let fx = ((x as f32 + 0.5) * sw as f32 / dw as f32 - 0.5).clamp(0.0, (sw - 1) as f32);
            let (x0, tx) = (fx.floor() as usize, fx.fract());
            let x1 = (x0 + 1).min(sw as usize - 1);
            let at = |x: usize, y: usize| src[y * sw as usize + x];
            let top = at(x0, y0) * (1.0 - tx) + at(x1, y0) * tx;
            let bottom = at(x0, y1) * (1.0 - tx) + at(x1, y1) * tx;
            out.push(top * (1.0 - ty) + bottom * ty);
        }
    }
    out
}

/// Box filter (mean over a `(2r+1)²` window) using a summed-area table.
fn box_mean(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut sat = vec![0f64; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0f64;
        for x in 0..w {
            row += src[y * w + x] as f64;
            sat[(y + 1) * (w + 1) + x + 1] = sat[y * (w + 1) + x + 1] + row;
        }
    }
    let mut out = vec![0f32; w * h];
    for y in 0..h {
        let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(h));
        for x in 0..w {
            let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(w));
            let s = sat[y1 * (w + 1) + x1] - sat[y0 * (w + 1) + x1] - sat[y1 * (w + 1) + x0]
                + sat[y0 * (w + 1) + x0];
            out[y * w + x] = (s / ((y1 - y0) * (x1 - x0)) as f64) as f32;
        }
    }
    out
}

/// He et al.'s guided filter: makes `p` follow the edges of `guide`.
fn guided_filter(guide: &[f32], p: &[f32], w: usize, h: usize, r: usize, eps: f32) -> Vec<f32> {
    let mul = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).collect::<Vec<f32>>();
    let mean_i = box_mean(guide, w, h, r);
    let mean_p = box_mean(p, w, h, r);
    let corr_ip = box_mean(&mul(guide, p), w, h, r);
    let corr_ii = box_mean(&mul(guide, guide), w, h, r);
    let mut a = vec![0f32; w * h];
    let mut b = vec![0f32; w * h];
    for i in 0..w * h {
        let var = corr_ii[i] - mean_i[i] * mean_i[i];
        let cov = corr_ip[i] - mean_i[i] * mean_p[i];
        a[i] = cov / (var + eps);
        b[i] = mean_p[i] - a[i] * mean_i[i];
    }
    let mean_a = box_mean(&a, w, h, r);
    let mean_b = box_mean(&b, w, h, r);
    (0..w * h)
        .map(|i| (mean_a[i] * guide[i] + mean_b[i]).clamp(0.0, 1.0))
        .collect()
}

#[cfg(feature = "background-removal")]
mod model {
    use std::sync::OnceLock;

    use tract_onnx::prelude::*;

    use crate::{Error, Result};

    /// Network input/output resolution.
    pub const SIZE: u32 = 320;

    /// The model file, embedded at compile time (see `models/README.md`).
    static MODEL_BYTES: &[u8] = include_bytes!("../../../../../models/u2netp.onnx");

    type Model = std::sync::Arc<TypedRunnableModel>;

    /// Loads and optimises the network once, on first use (~100 ms).
    fn model() -> Result<&'static Model> {
        static MODEL: OnceLock<std::result::Result<Model, String>> = OnceLock::new();
        MODEL
            .get_or_init(|| {
                tract_onnx::onnx()
                    .model_for_read(&mut std::io::Cursor::new(MODEL_BYTES))
                    .and_then(|m| {
                        m.with_input_fact(0, f32::fact([1, 3, SIZE as usize, SIZE as usize]).into())
                    })
                    .and_then(|m| m.into_optimized())
                    .and_then(|m| m.into_runnable())
                    .map_err(|e| e.to_string())
            })
            .as_ref()
            .map_err(|e| Error::Decode(format!("could not load the background model: {e}")))
    }

    /// Runs U²-Netp and returns a SIZE×SIZE mask in 0..=1.
    pub fn predict(rgba: &image::RgbaImage) -> Result<Vec<f32>> {
        let small =
            image::imageops::resize(rgba, SIZE, SIZE, image::imageops::FilterType::Lanczos3);
        // Preprocessing used when U²-Net was trained: scale by the maximum
        // value, then normalise with the ImageNet mean and std.
        let max = small
            .pixels()
            .flat_map(|p| [p[0], p[1], p[2]])
            .max()
            .unwrap_or(255)
            .max(1) as f32;
        const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
        const STD: [f32; 3] = [0.229, 0.224, 0.225];
        let n = SIZE as usize;
        let input = tract_ndarray::Array4::from_shape_fn((1, 3, n, n), |(_, c, y, x)| {
            (small.get_pixel(x as u32, y as u32)[c] as f32 / max - MEAN[c]) / STD[c]
        });
        let outputs = model()?
            .run(tvec!(Tensor::from(input).into()))
            .map_err(|e| Error::Decode(format!("background model failed: {e}")))?;
        // The first output (d0) is the fused, most accurate prediction.
        let pred = outputs[0]
            .to_plain_array_view::<f32>()
            .map_err(|e| Error::Decode(e.to_string()))?;
        let values: Vec<f32> = pred.iter().copied().collect();
        let (lo, hi) = values
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        let range = (hi - lo).max(1e-6);
        Ok(values.iter().map(|v| (v - lo) / range).collect())
    }
}

#[cfg(not(feature = "background-removal"))]
mod model {
    use crate::{Error, Result};

    pub const SIZE: u32 = 320;

    pub fn predict(_: &image::RgbaImage) -> Result<Vec<f32>> {
        Err(Error::BackgroundRemovalUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_mean_of_constant_is_constant() {
        let out = box_mean(&[2.0; 30], 6, 5, 2);
        assert!(out.iter().all(|&v| (v - 2.0).abs() < 1e-6));
    }

    #[test]
    fn guided_filter_snaps_a_blurry_mask_to_an_edge() {
        // Guide: sharp vertical edge at x = 10. Mask: a soft ramp around it.
        let (w, h) = (20, 8);
        let guide: Vec<f32> = (0..w * h)
            .map(|i| if i % w < 10 { 0.0 } else { 1.0 })
            .collect();
        let ramp: Vec<f32> = (0..w * h)
            .map(|i| (i % w) as f32 / (w - 1) as f32)
            .collect();
        let out = guided_filter(&guide, &ramp, w, h, 3, 1e-4);
        assert!(
            out[4 * w + 8] < 0.4 && out[4 * w + 11] > 0.6,
            "{:?}",
            &out[4 * w..5 * w]
        );
    }

    #[test]
    #[cfg(feature = "background-removal")]
    fn model_finds_a_centered_object() {
        // A red disc on a light grey background.
        let (w, h) = (200u32, 160u32);
        let data = (0..w * h)
            .flat_map(|i| {
                let (x, y) = ((i % w) as f32 - 100.0, (i / w) as f32 - 80.0);
                if x * x + y * y < 45.0 * 45.0 {
                    [220, 30, 40]
                } else {
                    [235, 235, 235]
                }
            })
            .collect();
        let mask = subject_mask(&Raster::from_u8(w, h, 3, data), true).unwrap();
        let at = |x: u32, y: u32| mask.data[(y * w + x) as usize];
        assert!(at(100, 80) > 200, "centre is subject: {}", at(100, 80));
        assert!(at(5, 5) < 50, "corner is background: {}", at(5, 5));
    }
}
