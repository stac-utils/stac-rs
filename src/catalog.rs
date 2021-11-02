pub struct Catalog {
    pub id: String,
}

impl Catalog {
    /// Creates a new `Catalog` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Catalog;
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Catalog {
        Catalog { id: id.to_string() }
    }
}
