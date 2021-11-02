pub struct Collection {
    pub id: String,
}

impl Collection {
    /// Creates a new `Collection` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Collection {
        Collection { id: id.to_string() }
    }
}
