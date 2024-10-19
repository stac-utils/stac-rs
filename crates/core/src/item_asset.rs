use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An item asset is an object that contains details about the datafiles that
/// will be included in member items.
///
/// Assets included at the Collection level do not imply that all assets are
/// available from all Items. However, it is recommended that the Asset
/// Definition is a complete set of all assets that may be available from any
/// member Items. So this should be the union of the available assets, not just
/// the intersection of the available assets.
///
/// Other custom fields, or fields from other extensions may also be included in the Asset object.
///
/// Any property that exists for a Collection-level asset object must also exist
/// in the corresponding assets object in each Item. If a collection's asset
/// object contains properties that are not explicitly stated in the Item's
/// asset object then that property does not apply to the item's asset. Item
/// asset objects at the Collection-level can describe any of the properties of
/// an asset, but those assets properties and values must also reside in the
/// item's asset object. To consolidate item-level asset object properties in an
/// API setting, consider storing the STAC Item objects without the larger
/// properties internally as 'invalid' STAC items, and merge in the desired
/// properties at serving time from the Collection-level.
///
/// At least two fields (e.g. title and type) are required to be provided, in
/// order for it to adequately describe Item assets. The two fields must not
/// necessarily be taken from the defined fields on this struct and may include
/// any custom field.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemAsset {
    /// The displayed title for clients and users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// A description of the Asset providing additional details, such as how it
    /// was processed or created.
    ///
    /// CommonMark 0.29 syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Media type of the asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The semantic roles of the asset, similar to the use of rel in links.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<String>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}
