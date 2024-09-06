use assert_cmd::Command;
use stac::ItemCollection;

#[test]
fn items() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    let output = command
        .arg("items")
        .arg("../core/assets/dataset_geo.tif")
        .arg("../core/assets/dataset.tif")
        .unwrap();
    let item_collection: ItemCollection = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(item_collection.items.len(), 2);
}
