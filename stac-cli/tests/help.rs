use assert_cmd::Command;

#[test]
fn help() {
    let mut command = Command::cargo_bin("stac").unwrap();
    command.arg("help").assert().success();
}
