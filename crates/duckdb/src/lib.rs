//! Use [duckdb](https://duckdb.org/) with [STAC](https://stacspec.org).

#![deny(unused_crate_dependencies)]

use arrow::array::RecordBatch;
use duckdb::{types::Value, Connection};
use geoarrow::table::Table;
use libduckdb_sys as _;
use stac_api::{Direction, Search};
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

    /// Unimplemented feature.
    #[error("unimplemented: {0}")]
    Unimplemented(&'static str),
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
                    let mut statement = format!(
                        "SELECT {} FROM read_parquet('{}')",
                        sql.select.as_deref().unwrap_or("*"),
                        href
                    );
                    if !sql.is_empty() {
                        statement.push(' ');
                        statement.push_str(&sql.query);
                    }
                    let mut statement = self.connection.prepare(&statement)?;
                    record_batches.push(
                        statement
                            .query_arrow(duckdb::params_from_iter(&sql.params))?
                            .collect(),
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
        let mut wheres = Vec::new();
        let mut params = Vec::new();
        if let Some(ids) = search
            .ids
            .and_then(|ids| if ids.is_empty() { None } else { Some(ids) })
        {
            wheres.push(format!("id IN ({})", repeat_vars(ids.len())));
            params.extend(ids.into_iter().map(Value::from));
        }
        if let Some(collections) =
            search
                .collections
                .and_then(|c| if c.is_empty() { None } else { Some(c) })
        {
            wheres.push(format!(
                "collection IN ({})",
                repeat_vars(collections.len())
            ));
            params.extend(collections.into_iter().map(Value::from));
        }
        if let Some(intersects) = search.intersects {
            wheres
                .push("ST_Intersects(ST_GeomFromWKB(geometry), ST_GeomFromGeoJSON(?))".to_string());
            params.push(Value::from(intersects.to_string()));
        }
        if let Some(bbox) = search.items.bbox {
            wheres
                .push("ST_Intersects(ST_GeomFromWKB(geometry), ST_GeomFromGeoJSON(?))".to_string());
            params.push(Value::from(bbox.to_geometry().to_string()));
        }
        if let Some(datetime) = search.items.datetime {
            // TODO support start and end datetimes
            let (start, end) = stac::datetime::parse(&datetime)?;
            if let Some(start) = start {
                wheres.push("datetime >= make_timestamp(?)".to_string());
                params.push(Value::from(start.timestamp_micros()));
            }
            if let Some(end) = end {
                wheres.push("datetime <= make_timestamp(?)".to_string());
                params.push(Value::from(end.timestamp_micros()));
            }
        }
        let mut query = String::new();
        if !wheres.is_empty() {
            query.push_str("WHERE ");
            query.push_str(&wheres.join(" AND "));
        }
        let mut select = None;
        if let Some(fields) = search.items.fields {
            // TODO protect against injection
            if !fields.include.is_empty() {
                select = Some(fields.include.join(","));
            }
            // TODO implement
            if !fields.exclude.is_empty() {
                return Err(Error::Unimplemented("fields.exclude"));
            }
        }
        if let Some(sortby) = search.items.sortby {
            query.push_str(" ORDER BY ");
            let sortby: Vec<_> = sortby
                .into_iter()
                .map(|sortby| {
                    format!(
                        "{} {}",
                        sortby.field,
                        match sortby.direction {
                            Direction::Ascending => "ASC",
                            Direction::Descending => "DESC",
                        }
                    )
                })
                .collect();
            query.push_str(&sortby.join(","));
        }
        if let Some(limit) = search.items.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }
        if search.items.filter.is_some() {
            return Err(Error::Unimplemented("filter"));
        }
        if search.items.filter_crs.is_some() {
            return Err(Error::Unimplemented("filter_crs"));
        }
        if search.items.query.is_some() {
            return Err(Error::Unimplemented("query"));
        }
        Ok(Sql {
            select,
            query,
            params,
        })
    }

    fn is_empty(&self) -> bool {
        self.query.is_empty()
    }
}

fn repeat_vars(count: usize) -> String {
    assert_ne!(count, 0);
    let mut s = "?,".repeat(count);
    s.pop();
    s
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

#[cfg(test)]
mod tests {
    use super::Client;
    use duckdb_test::duckdb_test;
    use stac_api::{Direction, Fields, Items, Search, Sortby};
    use std::sync::Mutex;

    // This is an absolutely heinous way to ensure that only one test is hitting
    // the DB at a time -- the MUTEX is used in the duckdb-test crate as part of
    // the code generated by `duckdb_test`.
    //
    // There's got to be a better way.
    static MUTEX: Mutex<()> = Mutex::new(());

    #[duckdb_test]
    fn search_all() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let item_collection = client.search(Search::default()).unwrap();
        assert_eq!(item_collection.items.len(), 100);
    }

    #[duckdb_test]
    fn search_ids() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let search = Search {
            ids: Some(vec![
                "S2B_MSIL2A_20240828T174909_R141_T13SEB_20240828T214916".to_string(),
                "S2B_MSIL2A_20240828T174909_R141_T13SDD_20240828T214916".to_string(),
            ]),
            ..Default::default()
        };
        let item_collection = client.search(search).unwrap();
        assert_eq!(item_collection.items.len(), 2);
    }

    #[duckdb_test]
    fn search_collections() {
        let mut client = Client::new().unwrap();
        client
            .add_href("data/100-sentinel-2-items.parquet")
            .unwrap();
        client.add_href("data/100-landsat-items.parquet").unwrap();
        let search = Search {
            collections: Some(vec!["sentinel-2-l2a".to_string()]),
            ..Default::default()
        };
        let item_collection = client.search(search).unwrap();
        assert_eq!(item_collection.items.len(), 100);
    }

    #[duckdb_test]
    fn search_intersects() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let search = Search {
            intersects: Some((&geo::point!(x: -105., y: 41.)).into()),
            ..Default::default()
        };
        let item_collection = client.search(search).unwrap();
        assert_eq!(item_collection.items.len(), 2);
    }

    #[duckdb_test]
    fn search_limit() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let items = Items {
            limit: Some(10),
            ..Default::default()
        };
        let item_collection = client.search(items).unwrap();
        assert_eq!(item_collection.items.len(), 10);
    }

    #[duckdb_test]
    fn search_bbox() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let items = Items {
            bbox: Some(vec![-105., 41., -104., 42.].try_into().unwrap()),
            ..Default::default()
        };
        let item_collection = client.search(items).unwrap();
        assert_eq!(item_collection.items.len(), 4);
    }

    #[duckdb_test]
    fn search_datetime() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let items = Items {
            datetime: Some("2024-08-29T00:00:00Z/2024-09-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        let item_collection = client.search(items).unwrap();
        assert_eq!(item_collection.items.len(), 53);
    }

    #[duckdb_test]
    fn search_fields() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let items = Items {
            fields: Some(Fields {
                include: vec!["id".to_string()],
                exclude: Vec::new(),
            }),
            ..Default::default()
        };
        let item_collection = client.search_to_json(items).unwrap();
        assert_eq!(item_collection.items.len(), 100);
        assert_eq!(item_collection.items[0].keys().len(), 1);
    }

    #[duckdb_test]
    fn search_sortby() {
        let client = Client::from_href("data/100-sentinel-2-items.parquet").unwrap();
        let items = Items {
            sortby: Some(vec![Sortby {
                field: "datetime".to_string(),
                direction: Direction::Ascending,
            }]),
            ..Default::default()
        };
        let item_collection = client.search(items).unwrap();
        for (a, b) in item_collection
            .items
            .iter()
            .zip(item_collection.items.iter().skip(1))
        {
            assert!(a.properties.datetime <= b.properties.datetime);
        }
    }
}
