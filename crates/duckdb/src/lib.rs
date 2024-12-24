//! Use [duckdb](https://duckdb.org/) with [STAC](https://stacspec.org).

#![warn(unused_crate_dependencies)]

use arrow::{
    array::{GenericByteArray, RecordBatch},
    datatypes::{GenericBinaryType, SchemaBuilder},
};
use duckdb::{types::Value, Connection};
use geoarrow::{
    array::{CoordType, WKBArray},
    datatypes::NativeType,
    table::Table,
    ArrayBase,
};
use libduckdb_sys as _;
use stac_api::Search;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};
use thiserror::Error;

/// A crate-specific error enum.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// [arrow::error::ArrowError]
    #[error(transparent)]
    Arrow(#[from] arrow::error::ArrowError),

    /// [duckdb::Error]
    #[error(transparent)]
    DuckDB(#[from] duckdb::Error),

    /// [geoarrow::error::GeoArrowError]
    #[error(transparent)]
    GeoArrow(#[from] geoarrow::error::GeoArrowError),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [parquet::errors::ParquetError]
    #[error(transparent)]
    Parquet(#[from] parquet::errors::ParquetError),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// Utf 8 error
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

/// A crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A client for making DuckDB requests for STAC objects.
#[derive(Debug)]
pub struct Client {
    connection: Connection,
    collections: HashMap<String, Vec<String>>,
}

/// A SQL query.
#[derive(Debug)]
pub struct Sql {
    /// The select.
    pub select: Option<String>,

    /// The query.
    pub query: String,

    /// The query parameters.
    pub params: Vec<Value>,
}

impl Client {
    /// Creates a new client with no data sources.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_duckdb::Client;
    ///
    /// let client = Client::new().unwrap();
    /// ```
    pub fn new() -> Result<Client> {
        let connection = Connection::open_in_memory()?;
        connection.execute("INSTALL spatial", [])?;
        connection.execute("LOAD spatial", [])?;
        Ok(Client {
            connection,
            collections: HashMap::new(),
        })
    }

    /// Adds a [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) href to this client.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_duckdb::Client;
    ///
    /// let mut client = Client::new().unwrap();
    /// client.add_href("data/100-sentinel-2-items.parquet").unwrap();
    /// ```
    pub fn add_href(&mut self, href: impl ToString) -> Result<()> {
        let href = href.to_string();
        let mut statement = self
            .connection
            .prepare(&format!("SELECT collection FROM read_parquet('{}')", href))?;
        let collections = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<HashSet<_>, duckdb::Error>>()?;
        for collection in collections {
            let entry = self.collections.entry(collection).or_default();
            entry.push(href.clone());
        }
        Ok(())
    }

    /// Creates a new client from a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_duckdb::Client;
    ///
    /// let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
    /// ```
    pub fn from_href(href: impl ToString) -> Result<Client> {
        let mut client = Client::new()?;
        client.add_href(href)?;
        Ok(client)
    }

    /// Searches this client, returning a [stac::ItemCollection].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_duckdb::Client;
    /// use stac_api::Search;
    ///
    /// let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
    /// let item_collection = client.search(Search::default()).unwrap();
    /// assert_eq!(item_collection.items.len(), 100);
    /// ```
    pub fn search(&self, search: impl Into<Search>) -> Result<stac::ItemCollection> {
        let mut items = Vec::new();
        for record_batches in self
            .search_to_arrow(search)?
            .into_iter()
            .filter(|r| !r.is_empty())
        {
            let schema = record_batches[0].schema();
            let table = Table::try_new(record_batches, schema)?;
            items.extend(stac::geoarrow::from_table(table)?.items);
        }
        Ok(items.into())
    }

    /// Searches this client, returning a vector of vectors of all matched record batches.
    ///
    /// # Examples
    ///
    /// Each inner grouping of record batches comes from the same source table:
    ///
    /// ```
    /// use stac_duckdb::Client;
    /// use stac_api::Search;
    ///
    /// let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
    /// for record_batches in client.search_to_arrow(Search::default()).unwrap() {
    ///     // Schema can be different between groups of record batches
    ///     for record_batch in record_batches {
    ///         // Each record batch in this scope will have the same schema
    ///     }
    /// }
    /// ```
    pub fn search_to_arrow(&self, search: impl Into<Search>) -> Result<Vec<Vec<RecordBatch>>> {
        let mut record_batches = Vec::new();
        let search = search.into();
        let collections = search.collections.clone();
        let sql = Sql::new(search)?;
        for collection in collections.unwrap_or_else(|| self.collections.keys().cloned().collect())
        {
            if let Some(hrefs) = self.collections.get(&collection) {
                for href in hrefs {
                    let statement = format!(
                        "SELECT {} FROM read_parquet('{}')",
                        sql.select.as_deref().unwrap_or("*"),
                        href
                    );
                    let mut statement = self.connection.prepare(&statement)?;
                    record_batches.push(
                        statement
                            .query_arrow(duckdb::params_from_iter(&sql.params))?
                            .map(to_geoarrow_record_batch)
                            .collect::<Result<_>>()?,
                    );
                }
            }
        }
        Ok(record_batches)
    }

    /// Searches this client, returning a [stac_api::ItemCollection].
    ///
    /// Use this method if you want JSON that might not be valid STAC items,
    /// e.g. if you've excluded required fields from the response.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_duckdb::Client;
    /// use stac_api::Search;
    ///
    /// let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
    /// let item_collection = client.search_to_json(Search::default()).unwrap();
    /// assert_eq!(item_collection.items.len(), 100);
    /// ```
    pub fn search_to_json(&self, search: impl Into<Search>) -> Result<stac_api::ItemCollection> {
        let mut items = Vec::new();
        for record_batches in self
            .search_to_arrow(search)?
            .into_iter()
            .filter(|r| !r.is_empty())
        {
            let schema = record_batches[0].schema();
            let table = Table::try_new(record_batches, schema)?;
            items.extend(stac::geoarrow::json::from_table(table)?);
        }
        Ok(items.into())
    }
}

impl Sql {
    fn new(search: Search) -> Result<Sql> {
        Ok(Sql {
            select: Some("ST_AsWKB(geometry)::BLOB geometry".to_string()),
            query: String::new(),
            params: Vec::new(),
        })
    }
}

/// Return this crate's version.
///
/// # Examples
///
/// ```
/// println!("{}", stac_duckdb::version());
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn to_geoarrow_record_batch(mut record_batch: RecordBatch) -> Result<RecordBatch> {
    let geometry_column = record_batch.remove_column(0);
    let binary_array: GenericByteArray<GenericBinaryType<i32>> =
        arrow::array::downcast_array(&geometry_column);
    let wkb_array = WKBArray::new(binary_array, Default::default());
    let geometry_array = geoarrow::io::wkb::from_wkb(
        &wkb_array,
        NativeType::Geometry(CoordType::Interleaved),
        false,
    )?;
    let mut columns = record_batch.columns().to_vec();
    let mut schema_builder = SchemaBuilder::from(&*record_batch.schema());
    schema_builder.push(geometry_array.extension_field());
    let schema = schema_builder.finish();
    columns.push(geometry_array.to_array_ref());
    let record_batch = RecordBatch::try_new(schema.into(), columns)?;
    Ok(record_batch)
}

#[cfg(test)]
mod tests {
    use super::Client;
    use rstest::{fixture, rstest};
    use stac_api::Search;
    use std::sync::Mutex;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[fixture]
    fn client() -> Client {
        let _mutex = MUTEX.lock().unwrap();
        Client::from_href("data/100-sentinel-2-items.parquet").unwrap()
    }

    #[rstest]
    fn search_all(client: Client) {
        let item_collection = client.search(Search::default()).unwrap();
        assert_eq!(item_collection.items.len(), 100);
    }
}
