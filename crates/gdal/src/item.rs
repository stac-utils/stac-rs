use gdal::{
    raster::{GdalDataType, StatisticsAll},
    Dataset,
};
use stac::{Asset, Bbox, DataType, Item, Statistics};
use stac_extensions::{raster::Band, Extensions, Projection, Raster};

use crate::{projection::ProjectionCalculations, Result};

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
/// stac_gdal::update_item(&mut item, false, true).unwrap();
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
    let mut bounds = item.bbox.unwrap_or(Bbox::TwoDimensional([
        f64::MAX,
        f64::MAX,
        f64::MIN,
        f64::MIN,
    ]));
    for asset in item.assets.values_mut() {
        update_asset(asset, force_statistics, is_approx_statistics_ok)?;
        let projection = asset.extension::<Projection>()?;
        if !projection.is_empty() {
            has_projection = true;
            if let Some(asset_bounds) = projection.wgs84_bounds()? {
                bounds.update(asset_bounds);
            }
            projections.push(projection);
        }
        if !has_raster && asset.has_extension::<Raster>() {
            has_raster = true;
        }
    }
    if bounds.is_valid() {
        item.set_geometry(geojson::Geometry::new(bounds.to_geometry().value))?;
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

/// Add /vsicurl/ to any http url.
///
/// It speeds up things a lot.
// TODO: Check other virtual filesystems (cloud storages?)
fn virtual_path(path: &str) -> String {
    if path.starts_with("http") {
        "/vsicurl/".to_owned() + path
    } else {
        path.to_owned()
    }
}

fn update_asset(
    asset: &mut Asset,
    force_statistics: bool,
    is_approx_statistics_ok: bool,
) -> Result<()> {
    let dataset = Dataset::open(virtual_path(&asset.href))?;
    let sparef = dataset.spatial_ref()?;
    let _ = sparef.to_wkt();

    let mut spatial_resolution = None;
    let mut projection = Projection::default();
    if let Ok(geo_transform) = dataset.geo_transform() {
        spatial_resolution = Some((geo_transform[1].abs() + geo_transform[5].abs()) / 2.0);
        let (width, height) = dataset.raster_size();
        // Yes, height comes first, see https://github.com/stac-extensions/projection/tree/f17b5707439e4d6aa5102a9018e5e52984d0d744?tab=readme-ov-file#projshape
        projection.shape = Some(vec![height, width]);
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
        // Yes, the order is strange, see https://github.com/stac-extensions/projection/tree/f17b5707439e4d6aa5102a9018e5e52984d0d744?tab=readme-ov-file#projtransform
        projection.transform = Some(vec![
            geo_transform[1],
            geo_transform[2],
            geo_transform[0],
            geo_transform[4],
            geo_transform[5],
            geo_transform[3],
            0.,
            0.,
            1.,
        ]);
    }
    if let Ok(spatial_ref) = dataset.spatial_ref() {
        if spatial_ref
            .auth_name()
            .ok()
            .map(|auth_name| auth_name == "EPSG")
            .unwrap_or_default()
        {
            projection.code = spatial_ref
                .auth_code()
                .ok()
                .map(|code| format!("EPSG:{}", code));
        }
        // FIXME There is no way to get WKT2 from gdal crate yet.
        // to_wkt() uses OSRExportToWkt, and we need OSRExportToWktEx with FORMAT=WKT2_2018 in options.
        if projection.code.is_none() {
            projection.projjson = spatial_ref
                .to_projjson()
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok());
        }
    }
    let count = dataset.raster_count();
    let mut bands = Vec::with_capacity(count);
    for i in 1..=count {
        let band = dataset.rasterband(i)?;
        bands.push(Band {
            nodata: band.no_data_value(),
            data_type: Some(gdal_type_to_stac(band.band_type())),
            spatial_resolution,
            scale: band.scale(),
            offset: band.offset(),
            unit: Some(band.unit()).filter(|s| !s.is_empty()),
            statistics: band
                .get_statistics(force_statistics, is_approx_statistics_ok)?
                .map(gdal_statistics_to_stac),
            // TODO: Check if we can read/calculate three values below
            histogram: None,
            sampling: None,
            bits_per_sample: None,
        })
    }
    asset.set_extension(projection)?;
    asset.set_extension(Raster { bands })?;
    Ok(())
}

fn gdal_type_to_stac(value: GdalDataType) -> stac::DataType {
    match value {
        GdalDataType::Unknown => DataType::Other,
        GdalDataType::UInt8 => DataType::UInt8,
        GdalDataType::Int8 => DataType::Int8,
        GdalDataType::UInt16 => DataType::UInt16,
        GdalDataType::Int16 => DataType::Int16,
        GdalDataType::UInt32 => DataType::UInt32,
        GdalDataType::Int32 => DataType::Int32,
        GdalDataType::UInt64 => DataType::UInt64,
        GdalDataType::Int64 => DataType::Int64,
        GdalDataType::Float32 => DataType::Float32,
        GdalDataType::Float64 => DataType::Float64,
    }
}

fn gdal_statistics_to_stac(value: StatisticsAll) -> Statistics {
    Statistics {
        mean: Some(value.mean),
        minimum: Some(value.min),
        maximum: Some(value.max),
        stddev: Some(value.std_dev),
        // TODO: Check if we can get/calculate value below
        valid_percent: None,
    }
}
