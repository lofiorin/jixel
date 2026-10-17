//! File formats Jixel knows about, and what each one can do.

use serde::{Deserialize, Serialize};

/// An image file format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Jxl,
    Jpeg,
    Png,
    Webp,
    Avif,
    Gif,
    Tiff,
    Bmp,
    Qoi,
    Ico,
    Tga,
    Pnm,
    /// Recognised (so we can give a clear message) but not decodable yet.
    Heic,
}

impl Format {
    /// Every format, in the order the UI lists them.
    pub const ALL: [Format; 13] = [
        Format::Jxl,
        Format::Avif,
        Format::Webp,
        Format::Jpeg,
        Format::Png,
        Format::Gif,
        Format::Tiff,
        Format::Bmp,
        Format::Qoi,
        Format::Ico,
        Format::Tga,
        Format::Pnm,
        Format::Heic,
    ];

    /// Detects the format from the first bytes of a file ("magic numbers").
    /// Always prefer this over the file extension, which can lie.
    pub fn detect(bytes: &[u8]) -> Option<Format> {
        let b = bytes;
        let has = |at: usize, sig: &[u8]| b.get(at..at + sig.len()) == Some(sig);
        Some(
            if has(0, &[0xFF, 0x0A]) || has(0, b"\0\0\0\x0CJXL \r\n\x87\n") {
                Format::Jxl
            } else if has(0, &[0xFF, 0xD8, 0xFF]) {
                Format::Jpeg
            } else if has(0, b"\x89PNG\r\n\x1a\n") {
                Format::Png
            } else if has(0, b"RIFF") && has(8, b"WEBP") {
                Format::Webp
            } else if has(4, b"ftyp") {
                match b.get(8..12)? {
                    b"avif" | b"avis" => Format::Avif,
                    b"heic" | b"heix" | b"hevc" | b"heim" | b"heis" | b"mif1" | b"msf1" => {
                        Format::Heic
                    }
                    _ => return None,
                }
            } else if has(0, b"GIF87a") || has(0, b"GIF89a") {
                Format::Gif
            } else if has(0, b"II*\0") || has(0, b"MM\0*") {
                Format::Tiff
            } else if has(0, b"BM") {
                Format::Bmp
            } else if has(0, b"qoif") {
                Format::Qoi
            } else if has(0, &[0, 0, 1, 0]) {
                Format::Ico
            } else if b.first() == Some(&b'P') && matches!(b.get(1), Some(b'1'..=b'7')) {
                Format::Pnm
            } else {
                return None;
            },
        )
    }

    /// Format for a file extension (case-insensitive, with or without a dot).
    pub fn from_extension(ext: &str) -> Option<Format> {
        Some(
            match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
                "jxl" => Format::Jxl,
                "jpg" | "jpeg" | "jpe" | "jfif" => Format::Jpeg,
                "png" | "apng" => Format::Png,
                "webp" => Format::Webp,
                "avif" => Format::Avif,
                "gif" => Format::Gif,
                "tif" | "tiff" => Format::Tiff,
                "bmp" => Format::Bmp,
                "qoi" => Format::Qoi,
                "ico" => Format::Ico,
                "tga" => Format::Tga,
                "pnm" | "pbm" | "pgm" | "ppm" | "pam" => Format::Pnm,
                "heic" | "heif" => Format::Heic,
                _ => return None,
            },
        )
    }

    /// Preferred file extension, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Format::Jxl => "jxl",
            Format::Jpeg => "jpg",
            Format::Png => "png",
            Format::Webp => "webp",
            Format::Avif => "avif",
            Format::Gif => "gif",
            Format::Tiff => "tiff",
            Format::Bmp => "bmp",
            Format::Qoi => "qoi",
            Format::Ico => "ico",
            Format::Tga => "tga",
            Format::Pnm => "ppm",
            Format::Heic => "heic",
        }
    }

    /// MIME type, as used in HTML `type=` attributes and HTTP headers.
    pub fn mime(self) -> &'static str {
        match self {
            Format::Jxl => "image/jxl",
            Format::Jpeg => "image/jpeg",
            Format::Png => "image/png",
            Format::Webp => "image/webp",
            Format::Avif => "image/avif",
            Format::Gif => "image/gif",
            Format::Tiff => "image/tiff",
            Format::Bmp => "image/bmp",
            Format::Qoi => "image/qoi",
            Format::Ico => "image/x-icon",
            Format::Tga => "image/x-tga",
            Format::Pnm => "image/x-portable-anymap",
            Format::Heic => "image/heic",
        }
    }

    /// Human-readable name.
    pub fn label(self) -> &'static str {
        match self {
            Format::Jxl => "JPEG XL",
            Format::Jpeg => "JPEG",
            Format::Png => "PNG",
            Format::Webp => "WebP",
            Format::Avif => "AVIF",
            Format::Gif => "GIF",
            Format::Tiff => "TIFF",
            Format::Bmp => "BMP",
            Format::Qoi => "QOI",
            Format::Ico => "ICO",
            Format::Tga => "TGA",
            Format::Pnm => "PNM",
            Format::Heic => "HEIC",
        }
    }

    pub fn can_decode(self) -> bool {
        !matches!(self, Format::Avif | Format::Heic)
    }

    pub fn can_encode(self) -> bool {
        !matches!(self, Format::Heic)
    }

    pub fn supports_alpha(self) -> bool {
        !matches!(
            self,
            Format::Jpeg | Format::Bmp | Format::Pnm | Format::Heic
        )
    }

    pub fn supports_animation(self) -> bool {
        matches!(self, Format::Jxl | Format::Gif | Format::Webp | Format::Png)
    }

    /// Whether the format has an adjustable lossy quality.
    pub fn is_lossy_capable(self) -> bool {
        matches!(
            self,
            Format::Jxl | Format::Jpeg | Format::Webp | Format::Avif
        )
    }

    /// Whether the format can store an image without any loss.
    pub fn is_lossless_capable(self) -> bool {
        !matches!(self, Format::Jpeg | Format::Heic)
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_magic_numbers() {
        assert_eq!(Format::detect(&[0xFF, 0x0A, 0]), Some(Format::Jxl));
        assert_eq!(
            Format::detect(b"\0\0\0\x0CJXL \r\n\x87\n...."),
            Some(Format::Jxl)
        );
        assert_eq!(
            Format::detect(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(Format::Jpeg)
        );
        assert_eq!(Format::detect(b"RIFF\0\0\0\0WEBPVP8 "), Some(Format::Webp));
        assert_eq!(Format::detect(b"\0\0\0\x1cftypavif"), Some(Format::Avif));
        assert_eq!(Format::detect(b"\0\0\0\x18ftypheic"), Some(Format::Heic));
        assert_eq!(Format::detect(b"nope"), None);
    }

    #[test]
    fn extensions_round_trip() {
        for f in Format::ALL {
            assert_eq!(Format::from_extension(f.extension()), Some(f));
        }
    }
}
