//! Abstractions to safely read GDAL datasets from multiple
//! threads.

use super::{RasterUtilsGdalError, Result};
use crate::chunking::ChunkWindow;
use crate::geometry::RasterWindow;
use gdal::{
    raster::{GdalType, RasterBand},
    Dataset,
};
use ndarray::Array2;

use std::{num::NonZeroUsize, path::Path};

/// Abstracts reading chunks from raster.
pub trait ChunkReader {
    /// Emulate [`RasterBand::read_into_slice`].
    fn read_into_slice<T>(&self, out: &mut [T], raster_window: RasterWindow) -> Result<()>
    where
        T: GdalType + Copy;

    /// Helper to read into an ndarray.
    fn read_as_array<T>(&self, raster_window: RasterWindow) -> Result<Array2<T>>
    where
        T: GdalType + Copy,
    {
        let bufsize = raster_window.num_pixels();
        let mut buf = Vec::with_capacity(bufsize);

        // Safety: paradigm suggested in std docs
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#examples-18
        unsafe {
            buf.set_len(bufsize);
        }

        let array_shape = raster_window.shape();
        self.read_into_slice(&mut buf[..], raster_window)?;
        Array2::from_shape_vec(array_shape, buf).map_err(RasterUtilsGdalError::NdarrayShapeError)
    }

    /* /// Helper to read into slice from output of
    /// [`ChunkConfig`] iterator
    fn read_chunk_into_slice<T>(
        &self,
        out: &mut [T],
        chunk: ChunkWindow,
    ) -> Result<()>
    where
        T: GdalType + Copy,
    {
        self.read_into_slice(out, chunk.into())
    } */

    /// Helper to read ndarray from output of
    /// [`ChunkConfig`] iterator
    fn read_chunk<T>(&self, chunk: ChunkWindow) -> Result<Array2<T>>
    where
        T: GdalType + Copy,
    {
        self.read_as_array(chunk.into())
    }

    // TODO: read using gdal read_chunk faster?
}

impl<'a> ChunkReader for RasterBand<'a> {
    fn read_into_slice<T>(&self, out: &mut [T], raster_window: RasterWindow) -> Result<()>
    where
        T: GdalType + Copy,
    {
        let (off, size) = raster_window.into();
        self.read_into_slice(off.into(), size, size, out, None)
            .map_err(RasterUtilsGdalError::GdalError)
    }
}

#[derive(Clone, Copy)]
pub struct BandIndex(NonZeroUsize);

impl BandIndex {
    fn get(&self) -> usize {
        self.0.get()
    }
}

/// A [`ChunkReader`] that is [`Send`], but not [`Sync`].
///
/// Obtains a `RasterBand` handle for each read.
pub struct DatasetReader(pub Dataset, pub BandIndex);

impl ChunkReader for DatasetReader {
    fn read_into_slice<T>(&self, out: &mut [T], raster_window: RasterWindow) -> Result<()>
    where
        T: GdalType + Copy,
    {
        let band = self.0.rasterband(self.1.get())?;
        ChunkReader::read_into_slice(&band, out, raster_window)
    }
}

/// A [`ChunkReader`] that is [`Send`] + [`Sync`].
///
/// Opens the dataset for each read.
pub struct RasterPathReader<'a, P: AsRef<Path> + ?Sized>(pub &'a P, pub BandIndex);

impl<'a, P> ChunkReader for RasterPathReader<'a, P>
where
    P: AsRef<Path> + ?Sized,
{
    fn read_into_slice<T>(&self, out: &mut [T], raster_window: RasterWindow) -> Result<()>
    where
        T: GdalType + Copy,
    {
        DatasetReader(Dataset::open(self.0)?, self.1).read_into_slice(out, raster_window)
    }
}
