use crate::{version::Step, Result, Version};

/// Trait for objects that can migrate from one version to another.
pub trait Migrate {
    /// Migrates this object to another version.
    ///
    /// # Examples
    ///
    /// [Item](crate::Item) implements migrate.
    ///
    /// ```
    /// use stac::{Item, Migrate};
    ///
    /// let mut item: Item = stac::read("../spec-examples/v1.0.0/simple-item.json").unwrap();
    /// item.migrate("1.1.0-beta.1".parse().unwrap()).unwrap();
    /// assert_eq!(item.version.to_string(), "1.1.0-beta.1");
    /// ```
    fn migrate(&mut self, version: Version) -> Result<()> {
        let steps = self.version().steps_to(version);
        for step in steps {
            match step {
                Step::v1_0_0_to_v1_1_0 => self.migrate_v1_0_0_to_v1_1_0()?,
                Step::v1_1_0_to_v1_0_0 => self.migrate_v1_1_0_to_v1_0_0()?,
            }
        }
        *self.version_mut() = version;
        Ok(())
    }

    /// Returns this object's current version.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Migrate, Item};
    ///
    /// let item: Item = stac::read("../spec-examples/v1.0.0/simple-item.json").unwrap();
    /// assert_eq!(item.version().to_string(), "1.0.0");
    /// ```
    fn version(&self) -> Version;

    /// Returns a mutable reference to this object's current version.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Migrate, Item};
    ///
    /// let mut item: Item = stac::read("../spec-examples/v1.0.0/simple-item.json").unwrap();
    /// *item.version_mut() = "1.1.0-beta.1".parse().unwrap();
    /// ```
    fn version_mut(&mut self) -> &mut Version;

    /// Migrate from v1.0.0 to v1.1.0.
    ///
    /// Default implementation is a no-op.
    fn migrate_v1_0_0_to_v1_1_0(&mut self) -> Result<()> {
        Ok(())
    }

    /// Migrate from v1.1.0 to v1.0.0.
    ///
    /// Default implementation is a no-op.
    fn migrate_v1_1_0_to_v1_0_0(&mut self) -> Result<()> {
        Ok(())
    }
}
