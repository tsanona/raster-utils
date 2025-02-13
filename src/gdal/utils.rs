use gdal::GeoTransform;
use geo::AffineTransform;

// TODO: Add other gdal utils from original crate

/// Converts raw GDAL [GeoTransform] information
/// into Geo [AffineTransform].
pub fn geo_affine_from(geo_transform: &GeoTransform) -> AffineTransform {
    AffineTransform::new(
        geo_transform[1],
        geo_transform[2],
        geo_transform[0],
        geo_transform[4],
        geo_transform[5],
        geo_transform[3],
    )
}


#[cfg(test)]
mod tests {
    use super::geo_affine_from;
    use gdal::Dataset;
    use geo::{AffineOps, Point};
    use std::path::Path;

    #[test]
    #[ignore]
    fn test_with_input() {
        use std::env::var;
        let path = var("RASTER").expect("env: RASTER not found");
        let transform = geo_affine_from(
            &Dataset::open(Path::new(&path))
                .unwrap()
                .geo_transform()
                .unwrap(),
        );

        eprint!("[");
        for param in [
            transform.a(),
            transform.b(),
            transform.xoff(),
            transform.d(),
            transform.e(),
            transform.yoff(),
        ] {
            eprint!("{:15.3}", param)
        }
        eprintln!("]");

        let t_pt = Point::new(0.0, 0.0).affine_transform(&transform);
        eprintln!("(0, 0) -> ({:15.3},{:15.3})", t_pt.x(), t_pt.y());
    }
}
