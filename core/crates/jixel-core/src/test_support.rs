//! Shared helpers for unit tests.

use crate::Raster;

/// A smooth, photo-like test image with some texture.
pub(crate) fn photo(w: u32, h: u32, channels: u8) -> Raster {
    let mut data = Vec::with_capacity((w * h * channels as u32) as usize);
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 / w as f32;
            let fy = y as f32 / h as f32;
            let tex = ((x * 7 + y * 13) % 17) as f32 / 17.0 * 20.0;
            let px = [
                (fx * 200.0 + tex) as u8,
                (fy * 180.0 + tex) as u8,
                ((1.0 - fx) * 150.0 + 50.0) as u8,
                if channels == 4 {
                    (fy * 255.0) as u8
                } else {
                    255
                },
            ];
            data.extend_from_slice(&px[..channels as usize]);
        }
    }
    Raster::from_u8(w, h, channels, data)
}
