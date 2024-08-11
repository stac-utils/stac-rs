use crate::{
    extensions::{ElectroOptical, Raster},
    Band, DataType, Extension, Extensions, Fields, Result, Statistics,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/// An Asset is an object that contains a URI to data associated with the [Item](crate::Item) that can be downloaded or streamed.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Asset {
    /// URI to the asset object.
    ///
    /// Relative and absolute URIs are both allowed.
    pub href: String,

    /// The displayed title for clients and users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// A description of the Asset providing additional details, such as how it was processed or created.
    ///
    /// [CommonMark 0.29](http://commonmark.org/) syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// [Media type](crate::media_type) of the asset.
    ///
    /// See the [common media types](https://github.com/radiantearth/stac-spec/blob/master/best-practices.md#common-media-types-in-stac) in the best practice doc for commonly used asset types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The semantic roles of the asset, similar to the use of rel in [Links](crate::Link).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub roles: Vec<String>,

    /// Creation date and time of the corresponding data, in UTC.
    ///
    /// This identifies the creation time of the data.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Date and time the metadata was updated last, in UTC.
    ///
    /// This identifies the updated time of the data.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,

    /// The bands array is used to describe the available bands in an Asset.
    ///
    /// This fields describes the general construct of a band or layer, which
    /// doesn't necessarily need to be a spectral band.  By adding fields from
    /// extensions you can indicate that a band, for example, is
    ///
    /// - a spectral band (EO extension),
    /// - a band with classification results (classification extension),
    /// - a band with quality information such as cloud cover probabilities,
    ///
    /// etc.
    ///
    /// This property is the successor of the eo:bands and raster:bands fields,
    /// which has been present in previous versions of these extensions.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub bands: Vec<Band>,

    /// Pixel values used to identify pixels that are nodata in the band either
    /// by the pixel value as a number or nan, inf or -inf (all strings).
    ///
    /// The extension specifies that this can be a number or a string, but we
    /// just use a f64 with a custom (de)serializer.
    ///
    /// TODO write custom (de)serializer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodata: Option<f64>,

    /// The data type of the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// Statistics of all the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Statistics>,

    /// Unit denomination of the pixel value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Additional fields on the asset.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,

    #[serde(skip)]
    extensions: Vec<String>,
}

/// Trait implemented by anything that has assets.
///
/// As of STAC v1.0.0, this is [Collection](crate::Collection) and [Item](crate::Item).
pub trait Assets {
    /// Returns a reference to this object's assets.
    ///
    /// # Examples
    ///
    /// [Item](crate::Item) has assets:
    ///
    /// ```
    /// use stac::{Item, Assets};
    /// let item: Item = stac::read("data/simple-item.json").unwrap();
    /// assert!(!item.assets().is_empty());
    /// ```
    fn assets(&self) -> &HashMap<String, Asset>;

    /// Returns a mut reference to this object's assets.
    ///
    /// # Examples
    ///
    /// [Item](crate::Item) has assets:
    ///
    /// ```
    /// use stac::{Item, Asset, Assets};
    /// let mut item: Item = stac::read("data/simple-item.json").unwrap();
    /// item.assets_mut().insert("foo".to_string(), Asset::new("./asset.tif"));
    /// ```
    fn assets_mut(&mut self) -> &mut HashMap<String, Asset>;
}

impl Asset {
    /// Creates a new asset with the provided href.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Asset;
    /// let asset = Asset::new("an-href");
    /// assert_eq!(asset.href, "an-href");
    /// ```
    pub fn new(href: impl ToString) -> Asset {
        Asset {
            href: href.to_string(),
            title: None,
            description: None,
            r#type: None,
            roles: Vec::new(),
            created: None,
            updated: None,
            bands: Vec::new(),
            nodata: None,
            data_type: None,
            statistics: None,
            unit: None,
            additional_fields: Map::new(),
            extensions: Vec::new(),
        }
    }

    /// Adds a role to this asset, returning the modified asset.
    ///
    /// Useful for builder patterns.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Asset;
    /// let asset = Asset::new("asset/dataset.tif").role("data");
    /// assert_eq!(asset.roles, vec!["data"]);
    /// ```
    pub fn role(mut self, role: impl ToString) -> Asset {
        self.roles.push(role.to_string());
        self.roles.dedup();
        self
    }

    /// Migrate eo and raster bands (STAC v1.0.0) to bands (STAC v1.1.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Asset, Extensions, extensions::{ElectroOptical, electro_optical::Band}};
    ///
    /// let mut asset = Asset::new("a-href");
    /// # #[allow(deprecated)]
    /// asset.set_extension(ElectroOptical {
    ///     bands: vec![Band {
    ///         name: "foo".to_string().into(),
    ///         ..Default::default()
    ///     },
    ///     Band {
    ///         name: "bar".to_string().into(),
    ///         ..Default::default()
    ///     }
    ///     ],
    ///     ..Default::default()
    /// }).unwrap();
    /// asset.migrate_bands();
    /// assert_eq!(asset.bands.len(), 2);
    /// assert_eq!(asset.bands[0].name.as_ref().unwrap(), "foo");
    /// assert_eq!(asset.bands[1].name.as_ref().unwrap(), "bar");
    /// ```
    #[allow(deprecated)]
    pub fn migrate_bands(&mut self) -> Result<()> {
        // Electro-Optical de-banding
        let mut electro_optical = self.extension::<ElectroOptical>()?;
        let mut keep_electro_optical = false;
        if self.bands.len() < electro_optical.bands.len() {
            self.bands
                .resize_with(electro_optical.bands.len(), Default::default);
        }
        for (electro_optical_band, band) in
            electro_optical.bands.iter_mut().zip(self.bands.iter_mut())
        {
            if let Some(name) = electro_optical_band.name.take() {
                band.name = Some(name);
            }
            if let Some(common_name) = electro_optical_band.common_name.take() {
                keep_electro_optical = true;
                let _ = band
                    .additional_fields
                    .insert("eo:common_name".into(), common_name.into());
            }
            if let Some(description) = electro_optical_band.description.take() {
                band.description = Some(description);
            }
            if let Some(center_wavelength) = electro_optical_band.center_wavelength.take() {
                keep_electro_optical = true;
                let _ = band
                    .additional_fields
                    .insert("eo:center_wavelength".into(), center_wavelength.into());
            }
            if let Some(full_width_half_max) = electro_optical_band.full_width_half_max.take() {
                keep_electro_optical = true;
                let _ = band
                    .additional_fields
                    .insert("eo:full_width_half_max".into(), full_width_half_max.into());
            }
            if let Some(solar_illumination) = electro_optical_band.solar_illumination.take() {
                keep_electro_optical = true;
                let _ = band
                    .additional_fields
                    .insert("eo:solar_illumination".into(), solar_illumination.into());
            }
        }
        let _ = self.additional_fields.remove("eo:bands");
        if !keep_electro_optical && electro_optical.is_empty() {
            self.remove_extension::<ElectroOptical>();
        }

        // Raster de-banding
        let mut raster = self.extension::<Raster>()?;
        let mut keep_raster = false;
        if self.bands.len() < raster.bands.len() {
            self.bands.resize_with(raster.bands.len(), Default::default);
        }
        for (raster_band, band) in raster.bands.iter_mut().zip(self.bands.iter_mut()) {
            if let Some(nodata) = raster_band.nodata.take() {
                band.nodata = Some(nodata);
            }
            if let Some(sampling) = raster_band.sampling.take() {
                keep_raster = true;
                let _ = band
                    .additional_fields
                    .insert("raster:sampling".into(), serde_json::to_value(sampling)?);
            }
            if let Some(data_type) = raster_band.data_type.take() {
                band.data_type = Some(data_type);
            }
            if let Some(bits_per_sample) = raster_band.bits_per_sample.take() {
                keep_raster = true;
                let _ = band
                    .additional_fields
                    .insert("raster:bits_per_sample".into(), bits_per_sample.into());
            }
            if let Some(spatial_resolution) = raster_band.spatial_resolution.take() {
                keep_raster = true;
                let _ = band.additional_fields.insert(
                    "raster:spatial_resolution".into(),
                    spatial_resolution.into(),
                );
            }
            if let Some(statistics) = raster_band.statistics.take() {
                band.statistics = Some(statistics);
            }
            if let Some(unit) = raster_band.unit.take() {
                band.unit = Some(unit);
            }
            if let Some(scale) = raster_band.scale.take() {
                keep_raster = true;
                let _ = band
                    .additional_fields
                    .insert("raster:scale".into(), scale.into());
            }
            if let Some(offset) = raster_band.offset.take() {
                keep_raster = true;
                let _ = band
                    .additional_fields
                    .insert("raster:offset".into(), offset.into());
            }
            if let Some(histogram) = raster_band.histogram.take() {
                keep_raster = true;
                let _ = band
                    .additional_fields
                    .insert("raster:histogram".into(), serde_json::to_value(histogram)?);
            }
        }
        let _ = self.additional_fields.remove("raster:bands");
        if !keep_raster && raster.is_empty() {
            self.remove_extension::<Raster>();
        }

        // Attribute de-duplication
        // TODO should the go onto `Fields`?
        let mut bands = serde_json::to_value(&self.bands)?;
        let mut keys = HashSet::new();
        for band in bands
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_object().unwrap())
        {
            keys.extend(band.keys().cloned());
        }
        for key in keys {
            let mut counts: HashMap<String, u16> = HashMap::new();
            let mut values: HashMap<String, Value> = HashMap::new();
            for band in bands
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b.as_object().unwrap())
            {
                if let Some(value) = band.get(&key) {
                    let s = serde_json::to_string(value)?;
                    if !values.contains_key(&s) {
                        let _ = values.insert(s.clone(), value.clone());
                    }
                    *counts.entry(s).or_default() += 1;
                }
            }
            if let Some((max_s, max_count)) = counts.into_iter().max_by(|a, b| a.1.cmp(&b.1)) {
                if max_count > 1 {
                    let value = values.remove(&max_s).unwrap();
                    for band in bands
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .map(|b| b.as_object_mut().unwrap())
                    {
                        if band.get(&key).map(|v| v == &value).unwrap_or_default() {
                            let _ = band.remove(&key);
                        }
                    }
                    let _ = self.additional_fields.insert(key, value);
                }
            }
        }
        self.bands = serde_json::from_value(bands)?;
        // I don't like doing this, but its the only way that I can think of to
        // get stuff like `data_type` up to the top level (instead of in
        // additional_properties).
        let _ = std::mem::replace(
            self,
            serde_json::from_value(serde_json::to_value(self.clone())?)?,
        );

        Ok(())
    }
}

impl Fields for Asset {
    fn fields(&self) -> &Map<String, Value> {
        &self.additional_fields
    }
    fn fields_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.additional_fields
    }
}

impl Extensions for Asset {
    fn extensions(&self) -> &Vec<String> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut Vec<String> {
        &mut self.extensions
    }
}

impl From<String> for Asset {
    fn from(value: String) -> Self {
        Asset::new(value)
    }
}

impl<'a> From<&'a str> for Asset {
    fn from(value: &'a str) -> Self {
        Asset::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Asset;

    #[test]
    fn new() {
        let asset = Asset::new("an-href");
        assert_eq!(asset.href, "an-href");
        assert!(asset.title.is_none());
        assert!(asset.description.is_none());
        assert!(asset.r#type.is_none());
        assert!(asset.roles.is_empty());
    }

    #[test]
    fn skip_serializing() {
        let asset = Asset::new("an-href");
        let value = serde_json::to_value(asset).unwrap();
        assert!(value.get("title").is_none());
        assert!(value.get("description").is_none());
        assert!(value.get("type").is_none());
        assert!(value.get("roles").is_none());
    }
}
