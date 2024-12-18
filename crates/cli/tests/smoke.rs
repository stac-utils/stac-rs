use assert_cmd::Command;
use std::{fs::File, io::Read};

#[test]
fn help() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command.arg("help").assert().success();
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

#[test]
fn create_stac_item() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command
        .arg("item")
        .arg("assets/dataset_geo.tif")
        .assert()
        .success();
}
