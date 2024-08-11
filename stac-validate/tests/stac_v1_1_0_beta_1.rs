use stac::Item;
use stac_validate::Validator;

#[test]
fn item() {
    let mut item = Item::new("an-id");
    item.version = "1.1.0-beta.1".parse().unwrap();
    let mut validator = Validator::for_version("1.1.0-beta.1".parse().unwrap()).unwrap();
    validator.validate(item).unwrap();
}
