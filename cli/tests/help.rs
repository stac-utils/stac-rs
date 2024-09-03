use assert_cmd::Command;

#[test]
fn help() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command.arg("help").assert().success();
}
