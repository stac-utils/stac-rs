use assert_cmd::Command;

#[test]
fn item() {
    let mut command = Command::cargo_bin("stacrs").unwrap();
    command.arg("item").arg("an-id").assert().success();
}
