//! Library to efficiently process GDAL rasters.

pub mod align;
pub mod chunking;
pub mod geometry;

//#[cfg(feature = "gdal")]
pub mod gdal;

#[derive(thiserror::Error, std::fmt::Debug)]
pub enum RasterUtilsError {
    //#[cfg(feature = "gdal")]
    #[error(transparent)]
    Gdal(gdal::error::RasterUtilsGdalError),
    #[error("Encountered an object with zero dimention")]
    ZeroDimention,
}

/// The `Result` type returned by this crate.
pub type Result<T> = std::result::Result<T, RasterUtilsError>;
