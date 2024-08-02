use crate::Result;
use serde::Serialize;

/// Prints output.
#[derive(Debug)]
pub struct Printer {
    compact: bool,
}

impl Printer {
    /// Creates a new printer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Printer;
    ///
    /// let printer = Printer::new(true);
    /// ```
    pub fn new(compact: bool) -> Printer {
        Printer { compact }
    }

    /// Prints some output in compact format, if possible.
    pub fn println_compact<S: Serialize>(&self, s: S) -> Result<()> {
        Ok(println!("{}", serde_json::to_string(&s)?))
    }

    /// Prints some serializable output.
    pub fn println<S: Serialize>(&self, s: S) -> Result<()> {
        let output = if self.compact {
            serde_json::to_string(&s)?
        } else {
            serde_json::to_string_pretty(&s)?
        };
        Ok(println!("{}", output))
    }
}
