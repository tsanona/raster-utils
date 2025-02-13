//! Geometry manipulation utilities

use std::usize;

use geo::{AffineOps, AffineTransform, Coord, Rect};

use super::chunking::ChunkWindow;

/// Represents size (x, y) of a raster or a window in pixels.
pub type Size = (usize, usize);

/// Represents offset (x, y) in pixels, within a raster.
pub type Offset = (usize, usize);

pub fn as_usize(tuple: (f64, f64)) -> (usize, usize) {
    (tuple.0.floor() as usize, tuple.1.floor() as usize)
}

pub fn as_f64(tuple: (usize, usize)) -> (f64, f64) {
    (tuple.0 as f64, tuple.1 as f64)
}

/// Same as [Offset](Offset) but for Gdal.
pub type GdalOffset = (isize, isize);

/* /// Represents transform from pixel coordinates to "world" coordinates.
pub type PixelWorldTransform = AffineTransform; */

/// Represents transform from a pixel coordinate to another pixel coordinate.
pub type PixelPixelTransform = AffineTransform;

///A block of contiguous data in a raster.
pub struct RasterWindow(Rect<f64>);

impl RasterWindow {
    /// Number of pixels within window.
    pub fn num_pixels(&self) -> usize {
        use geo::Area;
        self.0.unsigned_area() as usize
    }

    /// Window offset.
    pub fn offset(&self) -> Offset {
        as_usize(self.0.min().x_y())
    }

    /// Window size (x, y)
    pub fn size(&self) -> Size {
        as_usize((self.0.max() - self.0.min()).x_y())
    }

    /// Window shape (row, column)
    pub fn shape(&self) -> (usize, usize) {
        let (x, y) = self.size();
        (y, x)
    }

    /// Emulate [`Geo::affine_transform`].
    pub fn affine_transform(&self, transform: &AffineTransform) -> Self {
        Self(self.0.affine_transform(transform))
    }
}

impl From<(Offset, Size)> for RasterWindow {
    fn from(value: (Offset, Size)) -> Self {
        let min = Coord::from(as_f64(value.0));
        let max = min + Coord::from(as_f64(value.1));
        Self(Rect::new(min, max))
    }
}

impl From<RasterWindow> for (GdalOffset, Size) {
    fn from(value: RasterWindow) -> Self {
        let (x, y) = value.offset();
        ((x as isize, y as isize), value.size())
    }
}

impl<'a> From<ChunkWindow<'a>> for RasterWindow {
    fn from(value: ChunkWindow<'a>) -> Self {
        let (cfg, start, end) = value;
        ((0 as usize, start), (cfg.width(), end - start)).into()
    }
}
