//! Splitting images into tiles, and Deep Zoom pyramids.
//!
//! Tiles are cut from the decoded pixels, so each tile is pixel-exact when
//! stored losslessly. Use cases: zoomable product photos, maps and plans,
//! panoramas, and print or social-media grids ("carousel" splits).
//!
//! [`deep_zoom`] writes the Deep Zoom Image (DZI) layout used by viewers such
//! as OpenSeadragon: `<name>.dzi` plus `<name>_files/<level>/<col>_<row>.<ext>`.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::crop::crop;
use super::resize::resize_exact;
use super::Rect;
use crate::{Error, Raster, Result};

/// One tile and where it came from.
#[derive(Clone, Debug)]
pub struct Tile {
    pub col: u32,
    pub row: u32,
    pub rect: Rect,
    pub raster: Raster,
}

/// How to split.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Split {
    /// A fixed number of columns and rows; edge tiles absorb remainders.
    Grid { cols: u32, rows: u32 },
    /// Tiles of this size; the last column/row may be smaller.
    Size { width: u32, height: u32 },
}

/// The rectangles a split produces, row by row.
pub fn layout(width: u32, height: u32, split: Split) -> Result<Vec<(u32, u32, Rect)>> {
    let edges = |total: u32, n: u32| -> Vec<u32> {
        (0..=n)
            .map(|i| (total as u64 * i as u64 / n as u64) as u32)
            .collect()
    };
    let (xs, ys) = match split {
        Split::Grid { cols, rows } => {
            if cols == 0 || rows == 0 || cols > width || rows > height {
                return Err(Error::InvalidInput("invalid grid size".into()));
            }
            (edges(width, cols), edges(height, rows))
        }
        Split::Size {
            width: tw,
            height: th,
        } => {
            if tw == 0 || th == 0 {
                return Err(Error::InvalidInput("tile size must be positive".into()));
            }
            let steps = |total: u32, step: u32| -> Vec<u32> {
                let mut v: Vec<u32> = (0..total).step_by(step as usize).collect();
                v.push(total);
                v
            };
            (steps(width, tw), steps(height, th))
        }
    };
    let mut out = Vec::new();
    for (row, y) in ys.windows(2).enumerate() {
        for (col, x) in xs.windows(2).enumerate() {
            out.push((
                col as u32,
                row as u32,
                Rect::new(x[0], y[0], x[1] - x[0], y[1] - y[0]),
            ));
        }
    }
    Ok(out)
}

/// Splits `raster` into tiles (in parallel).
pub fn split(raster: &Raster, how: Split) -> Result<Vec<Tile>> {
    let mut r = raster.clone();
    r.apply_orientation()?;
    layout(r.width, r.height, how)?
        .into_par_iter()
        .map(|(col, row, rect)| {
            Ok(Tile {
                col,
                row,
                rect,
                raster: crop(&r, rect)?,
            })
        })
        .collect()
}

/// A generated Deep Zoom pyramid.
#[derive(Clone, Debug)]
pub struct DeepZoom {
    /// Contents of `<name>.dzi`.
    pub dzi: String,
    /// `(relative path, encoded tile)`, e.g. `("photo_files/12/3_4.jxl", …)`.
    pub files: Vec<(String, Vec<u8>)>,
    pub levels: u32,
}

/// Builds a Deep Zoom pyramid. `encode` turns each tile into file bytes and
/// `extension` is the matching file extension (e.g. `"jxl"`).
pub fn deep_zoom(
    raster: &Raster,
    name: &str,
    tile_size: u32,
    overlap: u32,
    extension: &str,
    encode: impl Fn(&Raster) -> Result<Vec<u8>> + Sync,
) -> Result<DeepZoom> {
    if tile_size < 16 {
        return Err(Error::InvalidInput("tile size must be at least 16".into()));
    }
    let mut full = raster.clone();
    full.apply_orientation()?;
    let (w, h) = (full.width, full.height);
    // Level `max` is full size; each level below halves it, down to 1×1.
    let max_level = (w.max(h) as f64).log2().ceil() as u32;
    let mut files = Vec::new();
    let mut level_image = full;
    for level in (0..=max_level).rev() {
        let scale = 1u64 << (max_level - level);
        let lw = (w as u64).div_ceil(scale) as u32;
        let lh = (h as u64).div_ceil(scale) as u32;
        if (lw, lh) != (level_image.width, level_image.height) {
            // Halve from the previous level: fast and sharp enough.
            level_image = resize_exact(&level_image, lw.max(1), lh.max(1))?;
        }
        let cols = lw.div_ceil(tile_size);
        let rows = lh.div_ceil(tile_size);
        let tiles: Vec<(String, Vec<u8>)> = (0..rows)
            .flat_map(|r| (0..cols).map(move |c| (c, r)))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(c, r)| {
                let x0 = (c * tile_size).saturating_sub(if c > 0 { overlap } else { 0 });
                let y0 = (r * tile_size).saturating_sub(if r > 0 { overlap } else { 0 });
                let x1 = ((c + 1) * tile_size + overlap).min(lw);
                let y1 = ((r + 1) * tile_size + overlap).min(lh);
                let tile = crop(&level_image, Rect::new(x0, y0, x1 - x0, y1 - y0))?;
                Ok((
                    format!("{name}_files/{level}/{c}_{r}.{extension}"),
                    encode(&tile)?,
                ))
            })
            .collect::<Result<_>>()?;
        files.extend(tiles);
    }
    let dzi = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <Image xmlns=\"http://schemas.microsoft.com/deepzoom/2008\" Format=\"{extension}\" Overlap=\"{overlap}\" TileSize=\"{tile_size}\">\n  \
         <Size Width=\"{w}\" Height=\"{h}\"/>\n</Image>\n"
    );
    Ok(DeepZoom {
        dzi,
        files,
        levels: max_level + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::photo;

    #[test]
    fn grid_covers_the_image_exactly() {
        let tiles = layout(101, 50, Split::Grid { cols: 3, rows: 2 }).unwrap();
        assert_eq!(tiles.len(), 6);
        let area: u64 = tiles.iter().map(|t| t.2.area()).sum();
        assert_eq!(area, 101 * 50);
    }

    #[test]
    fn size_split_has_smaller_edge_tiles() {
        let tiles = layout(
            100,
            30,
            Split::Size {
                width: 40,
                height: 40,
            },
        )
        .unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[2].2.width, 20);
    }

    #[test]
    fn tiles_reassemble_into_the_original() {
        let r = photo(50, 30, 3);
        let tiles = split(&r, Split::Grid { cols: 4, rows: 3 }).unwrap();
        let first = &tiles[0];
        assert_eq!(first.raster.samples, crop(&r, first.rect).unwrap().samples);
    }

    #[test]
    fn deep_zoom_has_one_level_per_halving() {
        let r = photo(300, 200, 3);
        let dz = deep_zoom(&r, "p", 128, 1, "png", |t| Ok(vec![t.width as u8])).unwrap();
        assert_eq!(dz.levels, 10); // 2^9 = 512 >= 300
        assert!(dz.files.iter().any(|(p, _)| p == "p_files/9/2_1.png"));
        assert!(dz.files.iter().any(|(p, _)| p == "p_files/0/0_0.png"));
        assert!(dz.dzi.contains("Width=\"300\""));
    }
}
