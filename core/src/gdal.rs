//! Functions that leverage [GDAL](https://gdal.org/).

use crate::{
    extensions::{
        projection::Centroid,
        raster::{Band, Raster, Statistics},
        Projection,
    },
    Asset, Bbox, Extensions, Item, Result,
};
use gdal::{
    spatial_ref::{CoordTransform, SpatialRef},
    Dataset,
};

/// Update an [Item] using GDAL.
///
/// Adds things like the [Raster] and [Projection] extensions.
///
/// # Examples
///
/// ```
/// use stac::{Asset, Item, Extensions, extensions::Raster};
/// let mut item = Item::new("an-id");
/// item.assets.insert("data".to_string(), Asset::new("assets/dataset.tif"));
/// stac::gdal::update_item(&mut item, false, true).unwrap();
/// assert!(item.has_extension::<Raster>());
/// ```
pub fn update_item(
    item: &mut Item,
    force_statistics: bool,
    is_approx_statistics_ok: bool,
) -> Result<()> {
    gdal::config::set_error_handler(|err, code, msg| log::warn!("{:?} ({}): {}", err, code, msg));
    let mut has_raster = false;
    let mut has_projection = false;
    let mut projections = Vec::new();
    let mut bbox = Bbox::default(); // TODO support 2D
    for asset in item.assets.values_mut() {
        update_asset(asset, force_statistics, is_approx_statistics_ok)?;
        if let Some(projection) = asset.extension::<Projection>()? {
            has_projection = true;
            if let Some(asset_bounds) = projection.wgs84_bounds()? {
                bbox.update(asset_bounds);
            }
            projections.push(projection);
        }
        if !has_raster && asset.has_extension::<Raster>() {
            has_raster = true;
        }
    }
    if bbox.is_valid() {
        item.geometry = Some(bbox.to_geometry());
        item.bbox = Some(bbox);
    }
    if has_projection {
        if !projections.is_empty()
            && projections
                .iter()
                .all(|projection| *projection == projections[0])
        {
            item.set_extension(projections[0].clone())?;
            for asset in item.assets.values_mut() {
                asset.remove_extension::<Projection>();
            }
        } else {
            item.add_extension::<Projection>();
        }
    }
    if has_raster {
        item.add_extension::<Raster>();
    }
    Ok(())
}

fn update_asset(
    asset: &mut Asset,
    force_statistics: bool,
    is_approx_statistics_ok: bool,
) -> Result<()> {
    let dataset = Dataset::open(&asset.href)?;
    let mut spatial_resolution = None;
    let mut projection = Projection::default();
    let (width, height) = dataset.raster_size();
    projection.shape = Some(vec![height, width]);
    if let Ok(geo_transform) = dataset.geo_transform() {
        projection.transform = Some(vec![
            geo_transform[1],
            geo_transform[2],
            geo_transform[0],
            geo_transform[4],
            geo_transform[5],
            geo_transform[3],
        ]);
        spatial_resolution = Some((geo_transform[1].abs() + geo_transform[5].abs()) / 2.0);
        let (width, height) = dataset.raster_size();
        let width = width as f64;
        let height = height as f64;
        let x0 = geo_transform[0];
        let x1 = geo_transform[0] + width * geo_transform[1];
        let x2 = geo_transform[0] + width * geo_transform[1] + height * geo_transform[2];
        let x3 = geo_transform[0] + height * geo_transform[2];
        let xmin = x0.min(x1).min(x2).min(x3);
        let xmax = x0.max(x1).max(x2).max(x3);
        let y0 = geo_transform[3];
        let y1 = geo_transform[3] + width * geo_transform[4];
        let y2 = geo_transform[3] + width * geo_transform[4] + height * geo_transform[5];
        let y3 = geo_transform[3] + height * geo_transform[5];
        let ymin = y0.min(y1).min(y2).min(y3);
        let ymax = y0.max(y1).max(y2).max(y3);
        projection.bbox = Some(vec![xmin, ymin, xmax, ymax]);
    }
    if let Ok(spatial_ref) = dataset.spatial_ref() {
        if spatial_ref
            .auth_name()
            .ok()
            .map(|auth_name| auth_name == "EPSG")
            .unwrap_or_default()
        {
            projection.epsg = spatial_ref.auth_code().ok();
        }
        // FIXME Don't know how to get WKT2 out of the gdal crate yet.
        if projection.epsg.is_none() {
            projection.projjson = spatial_ref
                .to_projjson()
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok());
        }
        if let Some(bbox) = projection.bbox.as_ref() {
            let mut x = [(bbox[0] + bbox[2]) / 2.];
            let mut y = [(bbox[1] + bbox[3]) / 2.];
            let coord_transform = CoordTransform::new(&spatial_ref, &SpatialRef::from_epsg(4326)?)?;
            coord_transform.transform_coords(&mut x, &mut y, &mut [])?;
            let round = |n: f64| (n * 10_000_000.).round() / 10_000_000.;
            projection.centroid = Some(Centroid {
                lat: round(x[0]),
                lon: round(y[0]),
            });
        }
    }
    let count = dataset.raster_count();
    let mut bands = Vec::with_capacity(count);
    for i in 1..=count {
        let band = dataset.rasterband(i)?;
        bands.push(Band {
            nodata: band.no_data_value(),
            data_type: Some(band.band_type().into()),
            spatial_resolution,
            scale: band.scale(),
            offset: band.offset(),
            unit: Some(band.unit()).filter(|s| !s.is_empty()),
            statistics: band
                .get_statistics(force_statistics, is_approx_statistics_ok)?
                .map(Statistics::from),
            ..Default::default()
        })
    }
    asset.set_extension(projection)?;
    asset.set_extension(Raster { bands })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        extensions::{projection::Centroid, raster::DataType, Projection, Raster},
        item::Builder,
        Extensions,
    };

    #[test]
    fn raster_data_type() {
        let mut item = Builder::new("an-id")
            .asset("data", "assets/dataset.tif")
            .into_item()
            .unwrap();
        super::update_item(&mut item, false, true).unwrap();
        assert!(item.has_extension::<Raster>());
        let raster: Raster = item
            .assets
            .get("data")
            .unwrap()
            .extension()
            .unwrap()
            .unwrap();
        assert_eq!(
            *raster.bands[0].data_type.as_ref().unwrap(),
            DataType::UInt16
        )
    }

    #[test]
    fn raster_spatial_resolution() {
        let mut item = Builder::new("an-id")
            .asset("data", "assets/dataset_geo.tif")
            .into_item()
            .unwrap();
        super::update_item(&mut item, false, true).unwrap();
        let raster: Raster = item
            .assets
            .get("data")
            .unwrap()
            .extension()
            .unwrap()
            .unwrap();
        assert_eq!(
            raster.bands[0].spatial_resolution.unwrap(),
            100.01126757344893
        );
    }

    #[test]
    fn projection() {
        let mut item = Builder::new("an-id")
            .asset("data", "assets/dataset_geo.tif")
            .into_item()
            .unwrap();
        super::update_item(&mut item, false, true).unwrap();
        let projection: Projection = item.extension().unwrap().unwrap();
        assert_eq!(projection.epsg.unwrap(), 32621);
        assert_eq!(
            projection.bbox.unwrap(),
            vec![373185.0, 8019284.949381611, 639014.9492102272, 8286015.0]
        );
        assert_eq!(projection.shape.unwrap(), vec![2667, 2658]);
        assert_eq!(
            projection.transform.unwrap(),
            vec![
                100.01126757344893,
                0.0,
                373185.0,
                0.0,
                -100.01126757344893,
                8286015.0,
            ]
        );
        assert_eq!(
            projection.centroid.unwrap(),
            Centroid {
                lon: -56.8079473,
                lat: 73.4675736,
            }
        )
    }
}
