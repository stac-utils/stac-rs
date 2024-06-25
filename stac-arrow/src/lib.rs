mod error;
mod json;
#[cfg(feature = "parquet")]
mod parquet;
mod table;

#[cfg(feature = "parquet")]
pub use parquet::read_parquet;
pub use {error::Error, table::table_to_items};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;
