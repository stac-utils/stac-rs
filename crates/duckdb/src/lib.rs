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
};
use stac_api::Search;
use std::fmt::Debug;
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

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),
}

/// A crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A client for making DuckDB requests for STAC objects.
#[derive(Debug)]
pub struct Client {
    connection: Connection,
}

/// A SQL query.
#[derive(Debug)]
pub struct Query {
    /// The SQL.
    pub sql: String,

    /// The parameters.
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
        Ok(Client { connection })
    }
    /// Searches this client, returning a [stac::ItemCollection].
    pub fn search(&self, href: &str, search: impl Into<Search>) -> Result<stac::ItemCollection> {
        let record_batches = self.search_to_arrow(href, search)?;
        let schema = record_batches[0].schema();
        let table = Table::try_new(record_batches, schema)?;
        let items = stac::geoarrow::from_table(table)?;
        Ok(items)
    }

    /// Searches this client, returning a vector of vectors of all matched record batches.
    pub fn search_to_arrow(
        &self,
        href: &str,
        search: impl Into<Search>,
    ) -> Result<Vec<RecordBatch>> {
        let query = self.query(search, href)?;
        let mut statement = self.connection.prepare(&query.sql)?;
        statement
            .query_arrow(duckdb::params_from_iter(query.params))?
            .map(to_geoarrow_record_batch)
            .collect::<Result<_>>()
    }

    /// Searches this client, returning a [stac_api::ItemCollection].
    ///
    /// Use this method if you want JSON that might not be valid STAC items,
    /// e.g. if you've excluded required fields from the response.
    pub fn search_to_json(
        &self,
        href: &str,
        search: impl Into<Search>,
    ) -> Result<stac_api::ItemCollection> {
        // TODO verify that we don't panic when zero items match
        let record_batches = self.search_to_arrow(href, search)?;
        let schema = record_batches[0].schema();
        let table = Table::try_new(record_batches, schema)?;
        let items = stac::geoarrow::json::from_table(table)?;
        let item_collection = stac_api::ItemCollection::new(items)?;
        Ok(item_collection)
    }

    fn query(&self, search: impl Into<Search>, href: &str) -> Result<Query> {
        // Ok(Sql {
        //     select: Some("ST_AsWKB(geometry)::BLOB geometry".to_string()),
        //     query: String::new(),
        //     params: Vec::new(),
        // })

        let mut statement = self.connection.prepare(&format!(
            "SELECT column_name FROM (DESCRIBE SELECT * from read_parquet('{}'))",
            href
        ))?;
        let mut columns = Vec::new();
        for row in statement.query_map([], |row| row.get::<_, String>(0))? {
            let column = row?;
            if column == "geometry" {
                columns.push("ST_AsWKB(geometry)::BLOB geometry".to_string());
            } else {
                columns.push(format!("\"{}\"", column));
            }
        }
        Ok(Query {
            sql: format!(
                "SELECT {} FROM read_parquet('{}')",
                columns.join(", "),
                href
            ),
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
    if let Some((index, _)) = record_batch.schema().column_with_name("geometry") {
        let geometry_column = record_batch.remove_column(index);
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
        record_batch = RecordBatch::try_new(schema.into(), columns)?;
    }
    Ok(record_batch)
}

#[cfg(test)]
mod tests {
    use super::Client;
    use rstest::{fixture, rstest};
    use stac::ValidateBlocking;
    use stac_api::Search;
    use std::sync::Mutex;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[fixture]
    fn client() -> Client {
        let _mutex = MUTEX.lock().unwrap();
        Client::new().unwrap()
    }

    #[rstest]
    fn search_all(client: Client) {
        let item_collection = client
            .search("data/100-sentinel-2-items.parquet", Search::default())
            .unwrap();
        assert_eq!(item_collection.items.len(), 100);
        item_collection.items[0].validate_blocking().unwrap();
    }
}
