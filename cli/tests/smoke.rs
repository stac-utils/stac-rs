use assert_cmd::Command;
use stac::ItemCollection;
use std::{fs::File, io::Read};

#[test]
fn help() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command.arg("help").assert().success();
}

#[test]
fn item() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command.arg("item").arg("an-id").assert().success();
}

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

#[test]
fn migrate() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command
        .arg("migrate")
        .arg("../spec-examples/v1.0.0/simple-item.json")
        .arg("--version")
        .arg("1.1.0")
        .assert()
        .success();
}

#[test]
fn translate() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command
        .arg("translate")
        .arg("examples/simple-item.json")
        .arg("-o")
        .arg("ndjson")
        .assert()
        .success();
}

#[test]
fn validate() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command
        .arg("validate")
        .arg("examples/simple-item.json")
        .assert()
        .success();
}

#[test]
fn validate_stdin() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    let mut item = String::new();
    File::open("examples/simple-item.json")
        .unwrap()
        .read_to_string(&mut item)
        .unwrap();
    command.arg("validate").write_stdin(item).assert().success();
}
