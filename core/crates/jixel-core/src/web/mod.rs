//! Responsive web export: everything a website needs to serve JPEG XL today.
//!
//! [`export`] turns one image into:
//!
//! * a set of widths (480 … 2560 px, never larger than the source) in JPEG XL
//!   plus fallback formats for browsers without JPEG XL support;
//! * a [`Manifest`] (`manifest.json`) describing every file, for build tools;
//! * ready-to-paste snippets for plain HTML, React/Next.js, WordPress, nginx
//!   and Apache ([`snippets`]).
//!
//! Browsers pick the first `<source>` they support, so visitors with JPEG XL
//! (Safari 17+, Chrome 155+, Firefox 157+) get the smallest file and everyone
//! else still gets a working image.

pub mod snippets;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::codec::jxl::FocusPoint;
use crate::convert::{encode_raster, ConvertOptions};
use crate::metadata::MetadataPolicy;
use crate::ops::resize::{self, Fit};
use crate::{Error, Format, Raster, Result};

/// Widths that cover phones, tablets, laptops and 4K screens without too
/// many files.
pub const DEFAULT_WIDTHS: [u32; 6] = [480, 768, 1080, 1440, 1920, 2560];

/// Settings for [`export`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebExportOptions {
    /// File name stem: `hero` gives `hero-1080.jxl`.
    pub name: String,
    /// Widths to generate; values above the source width are dropped and the
    /// source width itself is added if nothing else fits.
    pub widths: Vec<u32>,
    /// 0–100, applied to every format. 85 is visually lossless for most
    /// photos in JPEG XL.
    pub quality: f32,
    /// Formats served to browsers without JPEG XL, best first. The last one
    /// is used for the `<img>` fallback, so it should work everywhere.
    pub fallbacks: Vec<Format>,
    pub effort: u8,
    /// Progressive JPEG XL: a preview appears after a few percent of the file.
    pub progressive: bool,
    /// Load this point first (subject-first loading). Displayed coordinates
    /// of the *source* image; scaled for every width.
    pub focus: Option<FocusPoint>,
    /// URL path where the files will live, e.g. `/images/`.
    pub base_url: String,
    /// `sizes` attribute: how wide the image is shown, e.g.
    /// `(min-width: 1024px) 50vw, 100vw`.
    pub sizes: String,
    pub alt: String,
    /// Above-the-fold image (LCP): load eagerly with high priority instead
    /// of lazily.
    pub hero: bool,
    /// Metadata is stripped by default: it is useless on the web and may
    /// reveal where a photo was taken.
    pub metadata: MetadataPolicy,
}

impl Default for WebExportOptions {
    fn default() -> Self {
        Self {
            name: "image".into(),
            widths: DEFAULT_WIDTHS.to_vec(),
            quality: 85.0,
            fallbacks: vec![Format::Avif, Format::Jpeg],
            effort: 7,
            progressive: true,
            focus: None,
            base_url: "/images/".into(),
            sizes: "100vw".into(),
            alt: String::new(),
            hero: false,
            metadata: MetadataPolicy::StripAll,
        }
    }
}

/// One generated file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebFile {
    /// File name relative to the export folder, e.g. `hero-1080.jxl`.
    pub path: String,
    pub format: Format,
    pub width: u32,
    pub height: u32,
    pub bytes: usize,
}

/// Machine-readable description of an export (`manifest.json`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    /// Size of the largest generated version (use for `width`/`height`
    /// attributes to avoid layout shift).
    pub width: u32,
    pub height: u32,
    pub base_url: String,
    pub sizes: String,
    pub alt: String,
    pub hero: bool,
    /// Formats in the order browsers should try them (JPEG XL first).
    pub formats: Vec<Format>,
    pub files: Vec<WebFile>,
    /// Total bytes per format, to show the saving at a glance.
    pub totals: Vec<(Format, usize)>,
}

impl Manifest {
    /// Files of one format, smallest width first.
    pub fn files_of(&self, format: Format) -> impl Iterator<Item = &WebFile> {
        self.files.iter().filter(move |f| f.format == format)
    }

    /// The file used by the `<img>` fallback: the last fallback format, at
    /// the width closest to 1080 px.
    pub fn fallback(&self) -> Option<&WebFile> {
        let format = *self.formats.last()?;
        self.files_of(format).min_by_key(|f| f.width.abs_diff(1080))
    }

    /// `srcset` value for one format.
    pub fn srcset(&self, format: Format) -> String {
        self.files_of(format)
            .map(|f| format!("{}{} {}w", self.base_url, f.path, f.width))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Saving of JPEG XL against a format, in percent (−40 = 40% smaller).
    pub fn jxl_saving_vs(&self, format: Format) -> Option<f64> {
        let total = |f: Format| self.totals.iter().find(|t| t.0 == f).map(|t| t.1 as f64);
        Some((total(Format::Jxl)? / total(format)? - 1.0) * 100.0)
    }
}

/// The result of [`export`]: files to write plus their description.
#[derive(Clone, Debug)]
pub struct WebExport {
    pub manifest: Manifest,
    /// `(path, bytes)` for every image file.
    pub files: Vec<(String, Vec<u8>)>,
    /// `(path, text)` for `manifest.json` and every snippet.
    pub text_files: Vec<(String, String)>,
}

/// Creates the responsive image set for `raster`.
pub fn export(raster: &Raster, opts: &WebExportOptions) -> Result<WebExport> {
    if opts.name.is_empty() || opts.name.contains(['/', '\\', '"', '<', '>']) {
        return Err(Error::InvalidInput(
            "the export name must be a plain file name".into(),
        ));
    }
    let mut source = raster.clone();
    source.apply_orientation()?;
    source.drop_unused_alpha()?;

    let mut widths: Vec<u32> = opts
        .widths
        .iter()
        .copied()
        .filter(|&w| w > 0 && w <= source.width)
        .collect();
    if widths.is_empty() {
        // Smaller than every requested width: never upscale, export as is.
        widths.push(source.width);
    }
    widths.sort_unstable();
    widths.dedup();

    // JPEG XL first, then the fallbacks. Transparent images cannot use JPEG.
    let mut formats = vec![Format::Jxl];
    for &f in &opts.fallbacks {
        let f = if f == Format::Jpeg && source.has_alpha() {
            Format::Png
        } else {
            f
        };
        if !formats.contains(&f) && f.can_encode() {
            formats.push(f);
        }
    }

    let resized: Vec<Raster> = widths
        .par_iter()
        .map(|&w| resize::fit(&source, Fit::width(w)))
        .collect::<Result<_>>()?;
    let jobs: Vec<(usize, Format)> = (0..resized.len())
        .flat_map(|i| formats.iter().map(move |&f| (i, f)))
        .collect();
    let mut encoded: Vec<(WebFile, Vec<u8>)> = jobs
        .par_iter()
        .map(|&(i, format)| {
            let image = &resized[i];
            let scale = image.width as f64 / source.width as f64;
            let focus = opts.focus.map(|p| FocusPoint {
                x: ((p.x as f64 * scale) as u32).min(image.width - 1),
                y: ((p.y as f64 * scale) as u32).min(image.height - 1),
            });
            let convert = ConvertOptions {
                quality: opts.quality.min(99.0),
                effort: opts.effort,
                progressive: opts.progressive,
                focus,
                metadata: opts.metadata,
                ..ConvertOptions::to(format)
            };
            let bytes = encode_raster(image, &convert)?;
            let file = WebFile {
                path: format!("{}-{}.{}", opts.name, image.width, format.extension()),
                format,
                width: image.width,
                height: image.height,
                bytes: bytes.len(),
            };
            Ok((file, bytes))
        })
        .collect::<Result<_>>()?;
    encoded.sort_by_key(|(f, _)| (formats.iter().position(|&x| x == f.format), f.width));

    let largest = resized.last().expect("at least one width");
    let totals = formats
        .iter()
        .map(|&f| {
            (
                f,
                encoded
                    .iter()
                    .filter(|e| e.0.format == f)
                    .map(|e| e.1.len())
                    .sum(),
            )
        })
        .collect();
    let manifest = Manifest {
        name: opts.name.clone(),
        width: largest.width,
        height: largest.height,
        base_url: opts.base_url.clone(),
        sizes: opts.sizes.clone(),
        alt: opts.alt.clone(),
        hero: opts.hero,
        formats,
        files: encoded.iter().map(|e| e.0.clone()).collect(),
        totals,
    };
    let mut text_files = vec![(
        "manifest.json".to_string(),
        serde_json::to_string_pretty(&manifest).map_err(|e| Error::Encode(e.to_string()))?,
    )];
    text_files.extend(snippets::all(&manifest));
    let files = encoded.into_iter().map(|(f, b)| (f.path, b)).collect();
    Ok(WebExport {
        manifest,
        files,
        text_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::photo;

    #[test]
    fn widths_are_capped_and_formats_ordered() {
        let r = photo(1200, 800, 3);
        let opts = WebExportOptions {
            effort: 3,
            name: "hero".into(),
            ..Default::default()
        };
        let out = export(&r, &opts).unwrap();
        let m = &out.manifest;
        assert_eq!(m.formats, vec![Format::Jxl, Format::Avif, Format::Jpeg]);
        assert_eq!(
            m.files_of(Format::Jxl).map(|f| f.width).collect::<Vec<_>>(),
            vec![480, 768, 1080]
        );
        assert_eq!(out.files.len(), 9);
        assert!(m
            .srcset(Format::Jxl)
            .starts_with("/images/hero-480.jxl 480w, "));
        assert_eq!(m.fallback().unwrap().path, "hero-1080.jpg");
        assert!(
            m.jxl_saving_vs(Format::Jpeg).unwrap() < 0.0,
            "JPEG XL is smaller than JPEG"
        );
        assert!(out
            .text_files
            .iter()
            .any(|(p, _)| p == "snippets/picture.html"));
    }

    #[test]
    fn transparent_images_fall_back_to_png() {
        let out = export(
            &photo(300, 200, 4),
            &WebExportOptions {
                effort: 3,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            out.manifest.formats,
            vec![Format::Jxl, Format::Avif, Format::Png]
        );
        // Smaller than every default width: exported at its own size.
        assert_eq!(out.manifest.width, 300);
    }

    #[test]
    fn rejects_paths_as_names() {
        let opts = WebExportOptions {
            name: "../evil".into(),
            ..Default::default()
        };
        assert!(export(&photo(10, 10, 3), &opts).is_err());
    }
}
