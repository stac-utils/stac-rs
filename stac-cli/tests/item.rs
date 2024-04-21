use assert_cmd::Command;
use stac::{Item, ItemCollection};

#[test]
fn item() {
    let mut command = Command::cargo_bin("stac").unwrap();
    command.arg("item").arg("an-id").assert().success();
}

#[test]
fn item_collection() {
    let mut command = Command::cargo_bin("stac").unwrap();
    let item_a = serde_json::to_string(&Item::new("item-a")).unwrap();
    let output = command
        .arg("item")
        .arg("item-b")
        .arg("-c")
        .write_stdin(item_a)
        .unwrap();
    let item_collection: ItemCollection = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(item_collection.items.len(), 2);
}
