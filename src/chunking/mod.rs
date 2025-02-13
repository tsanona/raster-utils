//! Process rasters in memory-efficient chunks.
//!
//! It is often inefficient to load a large rasters
//! completely into memory while processing it. This module
//! provides iterators to load data in smaller chunks.
//!
//! # Raster Memory Layout
//!
//! Large rasters are typically sub-divided internally into
//! rectangular blocks of a specific size. For instance,
//! each band of a GDAL raster may be configured with a
//! _block size_ and the total dimension is split into
//! consecutive blocks of the specified size.
//!
//! The individual blocks support _random access_ while data
//! within a block may require reading the entire block (eg.
//! if the blocks are compressed). While the GDAL API
//! supports reading an arbitary window of data, the
//! underlying driver implements this by reading all the
//! necessary blocks and copying the necessary data into the
//! buffer. Thus, it is more efficient to read along block
//! boundaries.
//!
//! # Memory Efficient Iteration
//!
//! In order to process with a small memory footprint, the
//! algorithm must satisfy a **locality constraint**: to
//! process data at pixel `(x, y)`, it is sufficient to
//! access a small window (say `5x5`) centered around the
//! pixel. In particular, the chunks supported by this
//! module have the following properties:
//!
//! - **Full Width.** Each chunk spans the full width of the
//! raster. This simplifies the iteration logic, and is
//! currently the only supported mode.
//!
//! - **Fixed Padding.** Each chunk may additionally use a
//! fixed number of rows above and below it.

pub mod builder;
mod iters;
#[cfg(feature = "use-rayon")]
mod par_iters;

pub use super::{RasterUtilsError, Result};

/// Config for creating chunks within a raster.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkConfig {
    /// Width of raster to be chunked.
    width: usize,
    /// Height of raster to be chunked.
    height: usize,
    /// Size of chunks.
    ///
    /// For rasters with multiple bands
    /// this should be set to the
    /// least common multiple between
    /// all "natural" block sizes.
    block_size: usize,
    /// Minimum number of rows required in
    /// each chunk of data.
    /// Does not include the padding.
    /// This value should be a multiple of
    /// `block_size` for efficiency.    
    data_height: usize,
    /// Number of additional rows required on
    /// either size of the data.
    padding: usize,
    /// Start of processing range.
    ///
    /// Should be larger or equal to `padding`.
    start: usize,
    /// End of processing range.
    end: usize,
}

impl ChunkConfig {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }
    pub fn data_height(&self) -> usize {
        self.data_height
    }
    pub fn padding(&self) -> usize {
        self.padding
    }

    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
}

/// The type of item produced by the iterations. Consists
/// of:
///
/// 0. reference to the underlying `ChunkConfig`
/// 1. the start index of this chunk
/// 2. the number of rows (incl. padding) for this chunk
pub type ChunkWindow<'a> = (&'a ChunkConfig, usize, usize);

#[inline]
/// Find smallest multiple of m that is higher then num.
fn next_multiple(num: usize, m: usize) -> usize {
    num.div_ceil(m) * m
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use crate::chunking::builder::ChunkConfigBuilder;

    use super::*;

    fn debug_cfg(cfg: ChunkConfig) {
        eprintln!("{:?}", cfg);
        for (_, ls, size) in &cfg {
            eprintln!("{} -> {}", ls, ls + size);
        }
    }

    fn check_cfg(cfg: ChunkConfig, output: Vec<(usize, usize)>) {
        assert_eq!(
            cfg.into_iter().map(|(_, a, b)| (a, b)).collect::<Vec<_>>(),
            output
        );
    }

    #[test]
    #[ignore]
    fn test_with_input() {
        use std::env::var;
        let cfg = var("CHUNK_CONFIG").expect("env: CHUNK_CONFIG not found");
        let nums: Vec<usize> = cfg
            .trim()
            .split(' ')
            .map(str::parse)
            .collect::<std::result::Result<Vec<_>, _>>()
            .expect("couldn't parse CHUNK_CONFIG as [usize; 6]");

        debug_cfg(
            ChunkConfigBuilder::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(nums[0]).unwrap(),
            )
            .add_block_size(NonZeroUsize::new(nums[1]).unwrap())
            .with_data_height(NonZeroUsize::new(nums[2]).unwrap())
            .with_padding(nums[3])
            .with_start(nums[4])
            .with_end(nums[5])
            .build(),
        );
    }

    #[test]
    fn test_simple() {
        check_cfg(
            ChunkConfigBuilder::new(
                NonZeroUsize::new(32).unwrap(),
                NonZeroUsize::new(20).unwrap(),
            )
            .add_block_size(NonZeroUsize::new(2).unwrap())
            .with_padding(7)
            .with_end(10)
            .build(),
            vec![(0, 16), (2, 15)],
        )
    }
}
