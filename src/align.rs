//! Align a pair of rasters by their geo. transform.
//!
//! This module provides utilities to process a pair of
//! rasters with geographic alignment:
//!
//! - Given two raster bands `A` and `B` that don't
//! necessarily belong to the same raster, compute the
//! unique pixel `(k, l)` of `B` that contains the center of
//! the pixel `(i, j)` in `A`.
//!
//! - Extend the above functionality efficiently to work
//! with chunks of `A`.

use super::geometry::{as_f64, as_usize, Offset, PixelPixelTransform, Size};
use geo::{AffineTransform, Coord};

type ChunkTransform = PixelPixelTransform;

/// Calculate residue of an transform for a pair of offsets.
/// This is used to succinctly convert from array
/// coordinates of a chunk of one raster, to the array
/// coordinates of the corresponding chunk of another
/// raster.
///
/// # Arguments

/// - `transform` - [`PixelTransform`] between the pixel
/// coordinates of the two rasters. May be computed using [
/// `transform_between` ].
///
/// - `off_1` - starting coordinates of the chunk of the
/// first raster (a.k.a source chunk). Shift by `(0.5, 0.5)`
/// to map the center of the source pixel.
///
/// - `off_2` - starting coordinates of the corresponding
/// chunk of the second raster (a.k.a target chunk). The
/// extents of this is typically calculated using
/// [`transform_window`][crate::prelude::transform_window].
///
/// Returns a `PixelTransform` that transforms an array
/// index of the source chunk into array index of the target
/// chunk. Both indices are floating-point tuples,
/// representing interpolated position in the chunks.
///
/// # Derivation
///
/// Suppose `(x, y)` and `(X, Y)` represent the pixel
/// coordinates of the source and target rasters
/// respectively.   Then:
///
/// `(X, Y) = transform(x, y)`
///
/// `off_2 + (J, I) = transform(off_1 + (j, i))`
///
/// `(J, I) = transform(off_1) - off_2 + transform(j, i)`
pub fn chunk_transform(
    transform: &PixelPixelTransform,
    off_1: Offset,
    off_2: Offset,
) -> ChunkTransform {
    // zero out xoff and yoff and apply transform to off_1.
    // subtract off_2 from resulting coord.
    let residue = AffineTransform::translate(-1., -1.)
        .compose(transform)
        .apply(Coord::from(as_f64(off_1)))
        - Coord::from(as_f64(off_2));
    // summ resedue to xoff and yoff.
    AffineTransform::translate(residue.x, residue.y).compose(transform)
}

/// Converts a [`chunk_transform`] into a function that maps
/// input (integer) indices to indices on the output raster
/// if it falls within the given dimension (`dim`), and
/// otherwise `None`.
pub fn index_transformer(chunk_t: ChunkTransform, dim: Size) -> impl Fn(Size) -> Option<Size> {
    let (cols, rows) = dim;

    move |indexes| {
        // Transform indices
        let pt = chunk_t.apply(Coord::from(as_f64(indexes)));
        if pt.x < 0. || pt.y < 0. {
            return None;
        }

        let (j_2, i_2) = as_usize(pt.x_y());
        if j_2 >= cols || i_2 >= rows {
            None
        } else {
            Some((i_2, j_2))
        }
    }
}

/* #[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use gdal::Dataset;

    fn print_mat3x3(t: &AffineTransform) {
        eprint!("{:?}", t)
    }

    #[test]
    #[ignore]
    fn test_with_input() {
        use std::env::var;
        let path1 = var("RASTER1").expect("env: RASTER1 not found");
        let path2 = var("RASTER2").expect("env: RASTER2 not found");
        let ds1 = Dataset::open(Path::new(&path1)).unwrap();
        let ds2 = Dataset::open(Path::new(&path2)).unwrap();

        let t1 = transform_from_dataset(&ds1);
        let t2 = transform_from_dataset(&ds2);
        eprintln!("ds1 transform: ");
        print_mat3x3(&t1);
        eprintln!("ds2 transform: ");
        print_mat3x3(&t2);

        let tbet = transform_between(&ds1, &ds2).unwrap();
        eprintln!("transform between: ");
        print_mat3x3(&tbet);

        let tchunk = chunk_transform(&tbet, Vector2::new(0., 0.), Vector2::new(10., 0.));
        eprintln!("transform chunk: ");
        print_mat3x3(&tchunk);
    }
} */
