//! Extensions describe how STAC can use extensions that extend the
//! functionality of the core spec or add fields for specific domains.
//!
//! Extensions can be published anywhere, although the preferred location for
//! public extensions is in the GitHub
//! [stac-extensions](https://github.com/stac-extensions/) organization.
//! This crate currently supports only a few extensions, though we plan to add more as we find the time.
//! See <https://stac-extensions.github.io/> for the latest table of community extensions.
//! This table below lists all [stable](https://github.com/radiantearth/stac-spec/tree/master/extensions#extension-maturity) extensions, as well as any other extensions that are supported by **stac-rs**:
//!
//! | Extension | Maturity | **stac-rs** supported version |
//! | -- | -- | -- |
//! | [Authentication](https://github.com/stac-extensions/authentication) | Proposal | v1.1.0 |
//! | [Electro-Optical](https://github.com/stac-extensions/eo) | Stable | v1.1.0 |
//! | [File Info](https://github.com/stac-extensions/file) | Stable | n/a |
//! | [Landsat](https://github.com/stac-extensions/landsat) | Stable | n/a |
//! | [Projection](https://github.com/stac-extensions/projection) | Stable | v1.1.0 |
//! | [Raster](https://github.com/stac-extensions/raster) | Candidate | v1.1.0 |
//! | [Scientific Citation](https://github.com/stac-extensions/scientific) | Stable | n/a |
//! | [View Geometry](https://github.com/stac-extensions/view) | Stable | n/a |
//!
//! ## Usage
//!
//! [Item](crate::Item), [Collection](crate::Collection),
//! [Catalog](crate::Catalog), and [Asset](crate::Asset) all implement the
//! [Extensions] trait, which provides methods to get, set, and remove extension information:
//!
//! ```
//! use stac::{Item, Extensions, extensions::{Projection, projection::Centroid}};
//! let mut item: Item = stac::read("data/extensions-collection/proj-example/proj-example.json").unwrap();
//! assert!(item.has_extension::<Projection>());
//!
//! // Get extension information
//! let mut projection: Projection = item.extension().unwrap().unwrap();
//! println!("epsg: {}", projection.epsg.unwrap());
//!
//! // Set extension information
//! projection.centroid = Some(Centroid { lat: 34.595302, lon: -101.344483 });
//! item.set_extension(projection).unwrap();
//!
//! // Remove an extension
//! item.remove_extension::<Projection>();
//! assert!(!item.has_extension::<Projection>());
//! ```

pub mod authentication;
pub mod electro_optical;
pub mod projection;
pub mod raster;

use crate::{Fields, Result};
use serde::{de::DeserializeOwned, Serialize};
pub use {projection::Projection, raster::Raster};

/// A trait implemented by extensions.
///
/// So far, all extensions are assumed to live in under
/// <https://stac-extensions.github.io> domain.
pub trait Extension: Serialize + DeserializeOwned {
    /// The schema URI.
    const IDENTIFIER: &'static str;

    /// The fiend name prefix.
    const PREFIX: &'static str;

    /// Returns everything from the identifier up until the version.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::extensions::{Raster, Extension};
    /// assert_eq!(Raster::identifier_prefix(), "https://stac-extensions.github.io/raster/");
    /// ```
    fn identifier_prefix() -> &'static str {
        assert!(Self::IDENTIFIER.starts_with("https://stac-extensions.github.io/"));
        let index = Self::IDENTIFIER["https://stac-extensions.github.io/".len()..]
            .find('/')
            .expect("all identifiers should have a first path segment");
        &Self::IDENTIFIER[0.."https://stac-extensions.github.io/".len() + index + 1]
    }
}

/// A trait for objects that may have STAC extensions.
pub trait Extensions: Fields {
    /// Returns a reference to this object's extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Extensions, Item};
    /// let item = Item::new("an-id");
    /// assert!(item.extensions().is_empty());
    /// ```
    fn extensions(&self) -> &Vec<String>;

    /// Returns a mutable reference to this object's extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Extensions, Item};
    /// let mut item = Item::new("an-id");
    /// item.extensions_mut().push("https://stac-extensions.github.io/raster/v1.1.0/schema.json".to_string());
    /// ```
    fn extensions_mut(&mut self) -> &mut Vec<String>;

    /// Returns true if this object has the given extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, extensions::{Projection, Extensions}};
    /// let mut item = Item::new("an-id");
    /// assert!(!item.has_extension::<Projection>());
    /// let projection = Projection { epsg: Some(4326), ..Default::default() };
    /// item.set_extension(projection).unwrap();
    /// assert!(item.has_extension::<Projection>());
    /// ```
    fn has_extension<E: Extension>(&self) -> bool {
        self.extensions()
            .iter()
            .any(|extension| extension.starts_with(E::identifier_prefix()))
    }

    /// Gets an extension's data.
    ///
    /// Returns `Ok(None)` if the object doesn't have the given extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, extensions::{Projection, Extensions}};
    /// let item: Item = stac::read("data/extensions-collection/proj-example/proj-example.json").unwrap();
    /// let projection: Projection = item.extension().unwrap().unwrap();
    /// assert_eq!(projection.epsg.unwrap(), 32614);
    /// ```
    fn extension<E: Extension>(&self) -> Result<Option<E>> {
        if self.has_extension::<E>() {
            self.fields_with_prefix(E::PREFIX).map(|v| Some(v))
        } else {
            Ok(None)
        }
    }

    /// Adds an extension's identifier to this object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, extensions::{Projection, Extensions}};
    /// let mut item = Item::new("an-id");
    /// item.add_extension::<Projection>();
    /// ```
    fn add_extension<E: Extension>(&mut self) {
        self.extensions_mut().push(E::IDENTIFIER.to_string());
        self.extensions_mut().dedup();
    }

    /// Sets an extension's data and adds its schema to this object's `extensions`.
    ///
    /// This will remove any previous versions of this extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, extensions::{Projection, Extensions}};
    /// let mut item = Item::new("an-id");
    /// let projection = Projection { epsg: Some(4326), ..Default::default() };
    /// item.set_extension(projection).unwrap();
    /// ```
    fn set_extension<E: Extension>(&mut self, extension: E) -> Result<()> {
        self.remove_extension::<E>();
        self.extensions_mut().push(E::IDENTIFIER.to_string());
        self.extensions_mut().dedup();
        self.set_fields_with_prefix(E::PREFIX, extension)
    }

    /// Removes this extension and all of its fields from this object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, extensions::{Projection, Extensions}};
    /// let mut item: Item = stac::read("data/extensions-collection/proj-example/proj-example.json").unwrap();
    /// assert!(item.has_extension::<Projection>());
    /// item.remove_extension::<Projection>();
    /// assert!(!item.has_extension::<Projection>());
    /// ```
    fn remove_extension<E: Extension>(&mut self) {
        // TODO how do we handle removing from assets when this is done on an item?
        self.remove_fields_with_prefix(E::PREFIX);
        self.extensions_mut()
            .retain(|extension| !extension.starts_with(E::identifier_prefix()))
    }
}

#[cfg(test)]
mod tests {
    use super::Extensions;
    use crate::{
        extensions::{
            raster::{Band, Raster},
            Projection,
        },
        Asset, Extension, Item,
    };
    use serde_json::json;

    #[test]
    fn identifer_prefix() {
        assert_eq!(
            Raster::identifier_prefix(),
            "https://stac-extensions.github.io/raster/"
        );
        assert_eq!(
            Projection::identifier_prefix(),
            "https://stac-extensions.github.io/projection/"
        );
    }

    #[test]
    fn set_extension_on_asset() {
        let mut asset = Asset::new("a/href.tif");
        assert!(!asset.has_extension::<Raster>());
        let mut band = Band::default();
        band.unit = Some("parsecs".to_string());
        let raster = Raster { bands: vec![band] };
        asset.set_extension(raster).unwrap();
        assert!(asset.has_extension::<Raster>());
        let mut item = Item::new("an-id");
        let _ = item.assets.insert("data".to_string(), asset);

        // TODO how do we let items know about what their assets are doing?
        // Maybe we don't?
        // assert!(item.has_extension::<Raster>());
        // let item = serde_json::to_value(item).unwrap();
        // assert_eq!(
        //     item.as_object()
        //         .unwrap()
        //         .get("stac_extensions")
        //         .unwrap()
        //         .as_array()
        //         .unwrap(),
        //     &vec!["https://stac-extensions.github.io/raster/v1.1.0/schema.json"]
        // );
    }

    #[test]
    fn remove_extension() {
        let mut item = Item::new("an-id");
        item.extensions
            .push("https://stac-extensions.github.io/projection/v1.1.0/schema.json".to_string());
        let _ = item
            .properties
            .additional_fields
            .insert("proj:epsg".to_string(), json!(4326));
        assert!(item.has_extension::<Projection>());
        item.remove_extension::<Projection>();
        assert!(!item.has_extension::<Projection>());
        assert!(item.extensions.is_empty());
        assert!(item.properties.additional_fields.is_empty());
    }
}
