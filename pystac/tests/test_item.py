from pystac_rs import Item


def test_init():
    item = Item("an-id")
    assert item.id == "an-id"
    item.id = "new-id"
    assert item.id == "new-id"
