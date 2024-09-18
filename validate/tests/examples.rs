use rstest::rstest;
use stac::Value;
use stac_validate::ValidateBlocking;
use std::path::PathBuf;

#[rstest]
fn v1_0_0(#[files("../spec-examples/v1.0.0/**/*.json")] path: PathBuf) {
    let value: Value = stac::read(path.to_str().unwrap()).unwrap();
    value.validate_blocking().unwrap();
}

#[rstest]
fn v1_1_0(#[files("../spec-examples/v1.1.0/**/*.json")] path: PathBuf) {
    let value: Value = stac::read(path.to_str().unwrap()).unwrap();
    value.validate_blocking().unwrap();
}
