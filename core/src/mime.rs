//! [Media Types](https://en.wikipedia.org/wiki/Media_type) are a key element
//! that enables STAC to be a rich source of information for clients.
//!
//! The best practice is to use as specific of a media type as is possible (so
//! if a file is a GeoJSON then don't use a JSON media type), and to use
//! [registered](https://www.iana.org/assignments/media-types/media-types.xhtml)
//! IANA types as much as possible.  The following table lists types that
//! commonly show up in STAC assets.

/// GeoTIFF with standardized georeferencing metadata
#[deprecated(since = "0.9.0", note = "Prefer IMAGE_GEOTIFF")]
pub const GEOTIFF: &str = "image/tiff; application=geotiff";

/// GeoTIFF with standardized georeferencing metadata
pub const IMAGE_GEOTIFF: &str = "image/tiff; application=geotiff";

/// [Cloud Optimized GeoTIFF](https://www.cogeo.org/) (unofficial).
#[deprecated(since = "0.9.0", note = "Prefer IMAGE_GEOTIFF")]
pub const COG: &str = "image/tiff; application=geotiff; profile=cloud-optimized";

/// [Cloud Optimized GeoTIFF](https://www.cogeo.org/) (unofficial).
///
/// Once there is an [official media
/// type](http://osgeo-org.1560.x6.nabble.com/Media-type-tc5411498.html) it will
/// be added and the custom media type here will be deprecated.
pub const IMAGE_COG: &str = "image/tiff; application=geotiff; profile=cloud-optimized";

/// JPEG 2000
#[deprecated(since = "0.9.0", note = "Prefer IMAGE_JP2")]
pub const JP2: &str = "image/jp2";

/// JPEG 2000
pub const IMAGE_JP2: &str = "image/jp2";

/// Visual PNGs (e.g. thumbnails)
#[deprecated(since = "0.9.0", note = "Prefer ::mime::IMAGE_PNG")]
pub const PNG: &str = "image/png";

/// Visual JPEGs (e.g. thumbnails, oblique)
#[deprecated(since = "0.9.0", note = "Prefer ::mime::IMAGE_JPEG")]
pub const JPEG: &str = "image/jpeg";

/// XML metadata [RFC 7303](https://www.ietf.org/rfc/rfc7303.txt)
#[deprecated(since = "0.9.0", note = "Prefer ::mime::TEXT_JPEG")]
pub const XML: &str = "text/xml";

/// A JSON file (often metadata, or [labels](https://github.com/radiantearth/stac-spec/tree/master/extensions/label#labels-required))
#[deprecated(since = "0.9.0", note = "Prefer ::mime::APPLICATION_JSON")]
pub const JSON: &str = "application/json";

/// Plain text (often metadata)
#[deprecated(since = "0.9.0", note = "Prefer ::mime::TEXT_PLAIN")]
pub const TEXT: &str = "text/plain";

/// [GeoJSON](https://geojson.org/)
#[deprecated(since = "0.9.0", note = "Prefer APPLICATION_GEOJSON")]
pub const GEOJSON: &str = "application/geo+json";

/// [GeoJSON](https://geojson.org/)
pub const APPLICATION_GEOJSON: &str = "application/geo+json";

/// [GeoPackage](https://www.geopackage.org/)
#[deprecated(since = "0.9.0", note = "Prefer APPLICATION_GEOPACKAGE")]
pub const GEOPACKAGE: &str = "application/geopackage+sqlite3";

/// [GeoPackage](https://www.geopackage.org/)
pub const APPLICATION_GEOPACKAGE: &str = "application/geopackage+sqlite3";

/// Hierarchical Data Format version 5                           
#[deprecated(since = "0.9.0", note = "Prefer APPLICATION_HDF5")]
pub const HDF5: &str = "application/x-hdf5";

/// Hierarchical Data Format version 5                           
pub const APPLICATION_HDF5: &str = "application/x-hdf5";

/// Hierarchical Data Format versions 4 and earlier.
#[deprecated(since = "0.9.0", note = "Prefer APPLICATION_HDF")]
pub const HDF: &str = "application/x-hdf";

/// Hierarchical Data Format versions 4 and earlier.
pub const APPLICATION_HDF: &str = "application/x-hdf";

/// The OpenAPI 3.0 content type.
pub const APPLICATION_OPENAPI_3_0: &str = "application/vnd.oai.openapi+json;version=3.0";

/// [COPC](https://copc.io/) Cloud optimized point cloud
pub const APPLICATION_COPC: &str = "application/vnd.laszip+copc";

/// Apache [Geoparquet](https://geoparquet.org/)
pub const APPLICATION_PARQUET: &str = "application/vnd.apache.parquet";

/// [OGC 3D Tiles](https://www.ogc.org/standard/3dtiles/)       
pub const APPLICATION_3DTILES: &str = "application/3dtiles+json";

/// Protomaps [PMTiles](https://github.com/protomaps/PMTiles/blob/main/spec/v3/spec.md)
pub const APPLICATION_PMTILES: &str = "application/vnd.pmtiles";
