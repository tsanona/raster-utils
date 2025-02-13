use num::Integer;
use std::num::NonZeroUsize;

use super::{next_multiple, ChunkConfig};

/// Builder for [ChunkConfig].
pub struct ChunkConfigBuilder(ChunkConfig);
impl ChunkConfigBuilder {
    /// Create a [ChunkConfigBuilder] with given raster dimmentions.
    pub fn new(width: NonZeroUsize, height: NonZeroUsize) -> Self {
        let height = height.get();
        let default_config = ChunkConfig {
            width: width.get(),
            height,

            block_size: 1,
            data_height: 1,
            padding: 0,

            start: 0,
            end: height,
        };

        Self(default_config)
    }

    /// Accumulate `block_size` onto builder.
    ///
    /// Compute least common multiple with existing value and replace it.
    pub fn add_block_size(mut self, block_size: NonZeroUsize) -> Self {
        let block_size = block_size.get();
        if self.0.block_size != block_size {
            self.0.block_size = self.0.block_size.lcm(&block_size);
            self.adjust_data_height();
        }
        self
    }

    /// Set `data_height` for the chunking.
    pub fn with_data_height(mut self, data_height: NonZeroUsize) -> Self {
        self.0.data_height = data_height.get();
        self.adjust_data_height();
        self
    }

    /// Ensure `data_height` is a multiple of block size.
    #[inline]
    fn adjust_data_height(&mut self) {
        self.0.data_height = next_multiple(self.0.data_height, self.0.block_size);
    }

    /// Set `data_height` based on number of data pixels expected in each chunk.
    pub fn with_data_size(self, data_size: NonZeroUsize) -> Self {
        // data_height is zero iff data_size + width = 1
        // but data_size and width are both NonZeroUsize.
        let data_height = unsafe {
            NonZeroUsize::new_unchecked((data_size.get() + self.0.width - 1) / self.0.width)
        };
        self.with_data_height(data_height)
    }

    /// Set `padding` required for each chunk.
    pub fn with_padding(mut self, padding: usize) -> Self {
        self.0.padding = padding;
        self.adjust_start();
        self
    }

    /// Set `start` index of the iteration range.
    pub fn with_start(mut self, start: usize) -> Self {
        self.0.start = start;
        self.adjust_start();
        self
    }

    /// Ensure `start` is always greater than padding.
    #[inline]
    fn adjust_start(&mut self) {
        self.0.start = self.0.start.max(self.0.padding);
    }

    /// Set `end` index of the iteration range.
    pub fn with_end(mut self, end: usize) -> Self {
        self.0.end = end.min(self.0.height);
        self
    }

    /// Build [ChunkConfig]
    pub fn build(self) -> ChunkConfig {
        self.0
    }
}
