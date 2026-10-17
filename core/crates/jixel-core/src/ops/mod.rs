//! Image operations.
//!
//! * [`resize`]: high-quality downscaling (Lanczos3, alpha-aware).
//! * [`crop`]: pixel-exact or re-encoded crops.
//! * [`tiles`]: grid splitting and Deep Zoom (DZI) pyramids.
//! * [`background`]: on-device background removal (U²-Netp).
//! * [`subject`]: finds the main subject and spends more quality on it.

pub mod background;
pub mod crop;
pub mod resize;
pub mod subject;
pub mod tiles;

use serde::{Deserialize, Serialize};

/// An axis-aligned rectangle in image pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Grows the rectangle by `margin` on every side, staying inside
    /// `width × height`.
    pub fn expand(&self, margin: u32, width: u32, height: u32) -> Rect {
        let x = self.x.saturating_sub(margin);
        let y = self.y.saturating_sub(margin);
        Rect {
            x,
            y,
            width: (self.right() + margin).min(width) - x,
            height: (self.bottom() + margin).min(height) - y,
        }
    }

    /// Snaps the edges outwards to multiples of `step`, staying inside the
    /// image. JPEG XL works on 8×8 blocks, so aligned layers have clean edges.
    pub fn align_outwards(&self, step: u32, width: u32, height: u32) -> Rect {
        let x = self.x / step * step;
        let y = self.y / step * step;
        let r = self.right().div_ceil(step) * step;
        let b = self.bottom().div_ceil(step) * step;
        Rect {
            x,
            y,
            width: r.min(width) - x,
            height: b.min(height) - y,
        }
    }

    /// True if the rectangle lies within a `width × height` image and is not empty.
    pub fn fits_in(&self, width: u32, height: u32) -> bool {
        self.width > 0 && self.height > 0 && self.right() <= width && self.bottom() <= height
    }
}
