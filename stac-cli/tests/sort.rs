use assert_cmd::Command;
use std::{fs::File, io::Read};

#[test]
fn sort_stdin() {
    let mut command = Command::cargo_bin("stac").unwrap();
    let mut item = String::new();
    File::open("data/simple-item.json")
        .unwrap()
        .read_to_string(&mut item)
        .unwrap();
    command.arg("sort").write_stdin(item).assert().success();
}
