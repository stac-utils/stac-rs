use rstest::rstest;
use stac::{Validate, Value};
use std::path::PathBuf;

#[rstest]
fn v1_0_0(#[files("../../spec-examples/v1.0.0/**/*.json")] path: PathBuf) {
    let value: Value = stac::read(path.to_str().unwrap()).unwrap();
    value.validate().unwrap();
}

#[rstest]
fn v1_1_0(#[files("../../spec-examples/v1.1.0/**/*.json")] path: PathBuf) {
    let value: Value = stac::read(path.to_str().unwrap()).unwrap();
    value.validate().unwrap();
}

#[test]
#[cfg(feature = "object-store-aws")]
fn read_from_s3() {
    tokio_test::block_on(async {
        stac::io::get_opts::<stac::Catalog, _, _, _>(
            "s3://nz-elevation/catalog.json",
            [("skip_signature", "true"), ("region", "ap-southeast-2")],
        )
        .await
        .unwrap();
    });
}
