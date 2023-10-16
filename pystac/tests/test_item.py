from pystac_rs import Item


def test_init():
    item = Item("an-id")
    assert item.id == "an-id"
    assert item.version == "1.0.0"
    assert item.extensions is None
    assert item.geometry is None
    assert item.bbox is None
    assert item.properties["datetime"] is not None
    assert item.links.empty()
    assert item.collection is None


def test_setters():
    item = Item("an-id")
    item.id = "new-id"
    assert item.id == "new-id"
