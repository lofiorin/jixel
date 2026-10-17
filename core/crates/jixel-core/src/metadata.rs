//! EXIF / XMP metadata carried alongside an image, and privacy tools.
//!
//! EXIF is stored as raw TIFF data (starting with `II*\0` or `MM\0*`), which
//! is exactly what every container format embeds. We only parse the few tags
//! we need to edit in place (orientation and the GPS block), so unknown
//! maker notes survive untouched.

use serde::{Deserialize, Serialize};

/// What to do with metadata when writing a file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataPolicy {
    /// Keep EXIF and XMP as they are.
    Keep,
    /// Keep everything except location (GPS) data. The default: safe to
    /// publish, while keeping camera, copyright and date information.
    #[default]
    StripGps,
    /// Remove all EXIF and XMP. Smallest files; best for the web.
    StripAll,
}

/// Raw metadata blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    /// TIFF-structured EXIF data, without the `Exif\0\0` APP1 prefix.
    pub exif: Option<Vec<u8>>,
    /// XMP packet (UTF-8 XML).
    pub xmp: Option<Vec<u8>>,
}

const TAG_ORIENTATION: u16 = 0x0112;
const TAG_GPS_IFD: u16 = 0x8825;

impl Metadata {
    pub fn is_empty(&self) -> bool {
        self.exif.is_none() && self.xmp.is_none()
    }

    /// Builds metadata from EXIF as found in files. Strips the `Exif\0\0`
    /// prefix JPEG uses, and the 4-byte offset prefix JPEG XL uses.
    pub fn from_exif_any(raw: &[u8]) -> Option<Vec<u8>> {
        let mut data = raw;
        if data.starts_with(b"Exif\0\0") {
            data = &data[6..];
        }
        if data.len() > 4 && !is_tiff(data) && is_tiff(&data[4..]) {
            data = &data[4..];
        }
        is_tiff(data).then(|| data.to_vec())
    }

    /// Returns a copy with `policy` applied.
    pub fn with_policy(&self, policy: MetadataPolicy) -> Metadata {
        match policy {
            MetadataPolicy::Keep => self.clone(),
            MetadataPolicy::StripAll => Metadata::default(),
            MetadataPolicy::StripGps => {
                let mut m = self.clone();
                if let Some(exif) = m.exif.as_mut() {
                    strip_gps(exif);
                }
                if let Some(xmp) = m.xmp.as_mut() {
                    blank_xmp_gps(xmp);
                }
                m
            }
        }
    }

    /// EXIF orientation (1–8), if present.
    pub fn exif_orientation(&self) -> Option<u8> {
        let exif = self.exif.as_ref()?;
        let tiff = Tiff::new(exif)?;
        let entry = tiff.find_ifd0_entry(TAG_ORIENTATION)?;
        let value = tiff.u16_at(entry + 8)?;
        (1..=8).contains(&value).then_some(value as u8)
    }

    /// Sets the EXIF orientation tag to 1 ("normal"), if present.
    pub fn reset_exif_orientation(&mut self) {
        let Some(exif) = self.exif.as_mut() else {
            return;
        };
        let Some(tiff) = Tiff::new(exif) else { return };
        let Some(entry) = tiff.find_ifd0_entry(TAG_ORIENTATION) else {
            return;
        };
        let big = tiff.big_endian;
        let bytes = if big {
            1u16.to_be_bytes()
        } else {
            1u16.to_le_bytes()
        };
        exif[entry + 8..entry + 10].copy_from_slice(&bytes);
    }
}

fn is_tiff(d: &[u8]) -> bool {
    d.starts_with(b"II*\0") || d.starts_with(b"MM\0*")
}

/// Bounds-checked reader over TIFF bytes.
struct Tiff<'a> {
    data: &'a [u8],
    big_endian: bool,
}

impl<'a> Tiff<'a> {
    fn new(data: &'a [u8]) -> Option<Self> {
        is_tiff(data).then(|| Tiff {
            data,
            big_endian: data[0] == b'M',
        })
    }

    fn u16_at(&self, at: usize) -> Option<u16> {
        let b: [u8; 2] = self.data.get(at..at + 2)?.try_into().ok()?;
        Some(if self.big_endian {
            u16::from_be_bytes(b)
        } else {
            u16::from_le_bytes(b)
        })
    }

    fn u32_at(&self, at: usize) -> Option<u32> {
        let b: [u8; 4] = self.data.get(at..at + 4)?.try_into().ok()?;
        Some(if self.big_endian {
            u32::from_be_bytes(b)
        } else {
            u32::from_le_bytes(b)
        })
    }

    /// Byte offset of the 12-byte IFD0 entry with `tag`.
    fn find_ifd0_entry(&self, tag: u16) -> Option<usize> {
        let ifd = self.u32_at(4)? as usize;
        self.find_entry(ifd, tag)
    }

    fn find_entry(&self, ifd: usize, tag: u16) -> Option<usize> {
        let count = self.u16_at(ifd)? as usize;
        (0..count)
            .map(|i| ifd + 2 + i * 12)
            .find(|&e| self.u16_at(e) == Some(tag))
    }
}

/// Size in bytes of one value of a TIFF field type.
fn type_size(t: u16) -> usize {
    match t {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => 1,
    }
}

/// Zeroes every GPS value *and* the GPS directory itself, then leaves an
/// empty GPS directory behind. Zeroing (rather than just unlinking) matters:
/// unlinked bytes would still contain the coordinates.
fn strip_gps(exif: &mut [u8]) {
    let Some(tiff) = Tiff::new(exif) else { return };
    let Some(entry) = tiff.find_ifd0_entry(TAG_GPS_IFD) else {
        return;
    };
    let Some(gps_ifd) = tiff.u32_at(entry + 8).map(|v| v as usize) else {
        return;
    };
    let Some(count) = tiff.u16_at(gps_ifd).map(|v| v as usize) else {
        return;
    };

    let mut ranges = Vec::new();
    for i in 0..count {
        let e = gps_ifd + 2 + i * 12;
        let (Some(ty), Some(n)) = (tiff.u16_at(e + 2), tiff.u32_at(e + 4)) else {
            break;
        };
        let size = type_size(ty).saturating_mul(n as usize);
        if size > 4 {
            if let Some(off) = tiff.u32_at(e + 8) {
                ranges.push(off as usize..(off as usize).saturating_add(size));
            }
        }
        ranges.push(e..e + 12);
    }
    let len = exif.len();
    for r in ranges {
        let r = r.start.min(len)..r.end.min(len);
        exif[r].fill(0);
    }
    // An IFD with zero entries is valid TIFF.
    if gps_ifd + 2 <= len {
        exif[gps_ifd..gps_ifd + 2].fill(0);
    }
}

/// Overwrites `exif:GPS*` attributes and elements of an XMP packet with
/// spaces. Blanking (instead of removing) keeps the length identical, so the
/// packet can be edited in place inside a JPEG without rewriting segment
/// lengths, which also keeps JPEG XL's bit-exact JPEG reconstruction valid.
pub(crate) fn blank_xmp_gps(xmp: &mut [u8]) {
    let needle = b"exif:GPS";
    let mut i = 0;
    while let Some(pos) = find(&xmp[i..], needle).map(|p| p + i) {
        let element = pos > 0 && xmp[pos - 1] == b'<';
        let (from, to) = if element {
            // <exif:GPSLatitude>...</exif:GPSLatitude>  or  <exif:GPSx ... />
            let name_end = xmp[pos..]
                .iter()
                .position(|&c| c == b'>' || c == b'/' || c.is_ascii_whitespace())
                .map_or(xmp.len(), |e| pos + e);
            let mut close = b"</".to_vec();
            close.extend_from_slice(&xmp[pos..name_end]);
            close.push(b'>');
            let end = find(&xmp[name_end..], &close)
                .map(|e| name_end + e + close.len())
                .or_else(|| find(&xmp[name_end..], b"/>").map(|e| name_end + e + 2))
                .unwrap_or(xmp.len());
            (pos - 1, end)
        } else {
            // exif:GPSLatitude="..."
            let q1 = xmp[pos..].iter().position(|&c| c == b'"').map(|q| pos + q);
            let end = q1
                .and_then(|q| {
                    xmp[q + 1..]
                        .iter()
                        .position(|&c| c == b'"')
                        .map(|e| q + 1 + e + 1)
                })
                .unwrap_or(xmp.len());
            (pos, end)
        };
        xmp[from..to].fill(b' ');
        i = to;
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Zeroes GPS data inside a JPEG file's EXIF and XMP segments, in place.
/// Segment lengths never change, so the result is still a valid JPEG.
pub(crate) fn strip_gps_in_jpeg(jpeg: &mut [u8]) {
    let mut at = 2; // after SOI
    while at + 4 <= jpeg.len() && jpeg[at] == 0xFF {
        let marker = jpeg[at + 1];
        if marker == 0xDA || marker == 0xD9 {
            break; // start of scan / end of image: no more metadata
        }
        let len = u16::from_be_bytes([jpeg[at + 2], jpeg[at + 3]]) as usize;
        let (start, end) = (at + 4, (at + 2 + len).min(jpeg.len()));
        if marker == 0xE1 && start <= end {
            let seg = &mut jpeg[start..end];
            if seg.starts_with(b"Exif\0\0") {
                strip_gps(&mut seg[6..]);
            } else if seg.starts_with(b"http://ns.adobe.com/xap/1.0/") {
                blank_xmp_gps(seg);
            }
        }
        at += 2 + len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Little-endian TIFF with IFD0 = [orientation=6, GPS IFD pointer], and a
    /// GPS IFD with one RATIONAL[3] latitude stored out of line.
    fn sample_exif() -> Vec<u8> {
        let mut d = b"II*\0".to_vec();
        d.extend(8u32.to_le_bytes()); // IFD0 at 8
        d.extend(2u16.to_le_bytes()); // 2 entries
        d.extend(TAG_ORIENTATION.to_le_bytes());
        d.extend(3u16.to_le_bytes());
        d.extend(1u32.to_le_bytes());
        d.extend([6, 0, 0, 0]);
        d.extend(TAG_GPS_IFD.to_le_bytes());
        d.extend(4u16.to_le_bytes());
        d.extend(1u32.to_le_bytes());
        d.extend(38u32.to_le_bytes()); // GPS IFD at 38
        d.extend(0u32.to_le_bytes()); // next IFD
        assert_eq!(d.len(), 38);
        d.extend(1u16.to_le_bytes()); // 1 entry
        d.extend(2u16.to_le_bytes()); // GPSLatitude
        d.extend(5u16.to_le_bytes()); // RATIONAL
        d.extend(3u32.to_le_bytes());
        d.extend(56u32.to_le_bytes()); // data at 56
        d.extend(0u32.to_le_bytes());
        assert_eq!(d.len(), 56);
        d.extend([0xAB; 24]);
        d
    }

    #[test]
    fn reads_and_resets_orientation() {
        let mut m = Metadata {
            exif: Some(sample_exif()),
            xmp: None,
        };
        assert_eq!(m.exif_orientation(), Some(6));
        m.reset_exif_orientation();
        assert_eq!(m.exif_orientation(), Some(1));
    }

    #[test]
    fn strip_gps_erases_coordinates() {
        let m = Metadata {
            exif: Some(sample_exif()),
            xmp: None,
        }
        .with_policy(MetadataPolicy::StripGps);
        let exif = m.exif.unwrap();
        assert!(
            !exif.windows(2).any(|w| w == [0xAB, 0xAB]),
            "GPS bytes must be zeroed"
        );
        assert_eq!(&exif[38..40], &[0, 0], "GPS IFD left empty");
        assert_eq!(
            Metadata {
                exif: Some(exif),
                xmp: None
            }
            .exif_orientation(),
            Some(6)
        );
    }

    #[test]
    fn blank_xmp_gps_removes_attributes_and_elements() {
        let mut xmp = br#"<x a="1" exif:GPSLatitude="45,1N" b="2"><exif:GPSLongitude>9,2E</exif:GPSLongitude></x>"#.to_vec();
        let len = xmp.len();
        blank_xmp_gps(&mut xmp);
        let out = String::from_utf8(xmp).unwrap();
        assert_eq!(out.len(), len, "length must not change");
        assert!(
            !out.contains("GPS") && !out.contains("45,1N") && !out.contains("9,2E"),
            "{out}"
        );
        assert!(out.contains(r#"a="1""#) && out.contains(r#"b="2""#));
    }

    #[test]
    fn exif_prefixes_are_normalised() {
        let tiff = sample_exif();
        let mut jpeg = b"Exif\0\0".to_vec();
        jpeg.extend(&tiff);
        let mut jxl = vec![0, 0, 0, 0];
        jxl.extend(&tiff);
        assert_eq!(Metadata::from_exif_any(&jpeg), Some(tiff.clone()));
        assert_eq!(Metadata::from_exif_any(&jxl), Some(tiff));
    }
}
