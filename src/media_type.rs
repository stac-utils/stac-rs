//! [Media Types](https://en.wikipedia.org/wiki/Media_type) are a key element
//! that enables STAC to be a rich source of information for clients.
//!
//! The best practice is to use as specific of a media type as is possible (so
//! if a file is a GeoJSON then don't use a JSON media type), and to use
//! [registered](https://www.iana.org/assignments/media-types/media-types.xhtml)
//! IANA types as much as possible.  The following table lists types that
//! commonly show up in STAC assets.

/// GeoTIFF with standardized georeferencing metadata
pub const GEOTIFF: &str = "image/tiff; application=geotiff";

/// [Cloud Optimized GeoTIFF](https://www.cogeo.org/) (unofficial).
///
/// Once there is an [official media
/// type](http://osgeo-org.1560.x6.nabble.com/Media-type-tc5411498.html) it will
/// be added and the custom media type here will be deprecated.
pub const COG: &str = "image/tiff; application=geotiff; profile=cloud-optimized";

/// JPEG 2000
pub const JP2: &str = "image/jp2";

/// Visual PNGs (e.g. thumbnails)
pub const PNG: &str = "image/png";

/// Visual JPEGs (e.g. thumbnails, oblique)
pub const JPEG: &str = "image/jpeg";

/// XML metadata [RFC 7303](https://www.ietf.org/rfc/rfc7303.txt)
pub const XML: &str = "text/xml";

/// A JSON file (often metadata, or [labels](https://github.com/radiantearth/stac-spec/tree/master/extensions/label#labels-required))
pub const JSON: &str = "application/json";

/// Plain text (often metadata)
pub const TEXT: &str = "text/plain";

/// [GeoJSON](https://geojson.org/)
pub const GEOJSON: &str = "application/geo+json";

/// [GeoPackage](https://www.geopackage.org/)
pub const GEOPACKAGE: &str = "application/geopackage+sqlite3";

/// Hierarchical Data Format version 5                           
pub const HDF5: &str = "application/x-hdf5";

/// Hierarchical Data Format versions 4 and earlier.
pub const HDF: &str = "application/x-hdf";
