use gdal::errors::GdalError;
use ndarray::ShapeError;

#[derive(thiserror::Error, Debug)]
pub enum RasterUtilsGdalError {
    #[error(transparent)]
    GdalError(#[from] GdalError),
    #[error(transparent)]
    NdarrayShapeError(#[from] ShapeError),
}

pub type Result<T> = std::result::Result<T, RasterUtilsGdalError>;
