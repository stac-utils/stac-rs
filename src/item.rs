pub struct Item {
    pub id: String,
}

impl Item {
    /// Creates a new `Item` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Item;
    /// let item = Item::new("an-id");
    /// assert_eq!(item.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Item {
        Item { id: id.to_string() }
    }
}
