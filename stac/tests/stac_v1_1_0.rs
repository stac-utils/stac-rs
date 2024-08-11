use assert_json_diff::{assert_json_matches, CompareMode, Config, NumericMode};
use serde_json::json;
use stac::Asset;

#[test]
fn migrate_bands_on_asset() {
    // https://github.com/radiantearth/stac-spec/blob/master/best-practices.md#band-migration
    let mut asset: Asset = serde_json::from_value(json!({
        "href": "example.tif",
        "eo:bands": [
          {
            "name": "r",
            "common_name": "red"
          },
          {
            "name": "g",
            "common_name": "green"
          },
          {
            "name": "b",
            "common_name": "blue"
          },
          {
            "name": "nir",
            "common_name": "nir"
          }
        ],
        "raster:bands": [
          {
            "data_type": "uint16",
            "spatial_resolution": 10,
            "sampling": "area"
          },
          {
            "data_type": "uint16",
            "spatial_resolution": 10,
            "sampling": "area"
          },
          {
            "data_type": "uint16",
            "spatial_resolution": 10,
            "sampling": "area"
          },
          {
            "data_type": "uint16",
            "spatial_resolution": 30,
            "sampling": "area"
          }
        ]
    }))
    .unwrap();
    asset.migrate_bands().unwrap();
    let config = Config::new(CompareMode::Strict).numeric_mode(NumericMode::AssumeFloat);
    assert_json_matches!(
        serde_json::to_value(asset).unwrap(),
        json! {{
          "href": "example.tif",
          "data_type": "uint16",
          "raster:sampling": "area",
          "raster:spatial_resolution": 10,
          "bands": [
            {
              "name": "r",
              "eo:common_name": "red",
            },
            {
              "name": "g",
              "eo:common_name": "green"
            },
            {
              "name": "b",
              "eo:common_name": "blue"
            },
            {
              "name": "nir",
              "eo:common_name": "nir",
              "raster:spatial_resolution": 30
            }
          ]
        }},
        config
    );
}
