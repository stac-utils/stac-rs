//! Use [duckdb](https://duckdb.org/) with [STAC](https://stacspec.org).

#![warn(unused_crate_dependencies)]

use arrow::{
    array::{AsArray, GenericByteArray, RecordBatch},
    datatypes::{GenericBinaryType, SchemaBuilder},
};
use chrono::DateTime;
use duckdb::{types::Value, Connection};
use geo::BoundingRect;
use geoarrow::{
    array::{CoordType, WKBArray},
    datatypes::NativeType,
    table::Table,
};
use geojson::Geometry;
use stac::{Collection, SpatialExtent, TemporalExtent};
use stac_api::{Direction, Search};
use std::fmt::Debug;
use thiserror::Error;

const DEFAULT_COLLECTION_DESCRIPTION: &str =
    "Auto-generated collection from stac-geoparquet extents";

/// Searches a stac-geoparquet file.
pub fn search(
    href: &str,
    mut search: Search,
    max_items: Option<usize>,
) -> Result<stac_api::ItemCollection> {
    if let Some(max_items) = max_items {
        search.limit = Some(max_items.try_into()?);
    } else {
        search.limit = None;
    };
    let client = Client::new()?;
    client.search_to_json(href, search)
}

/// A crate-specific error enum.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// [arrow::error::ArrowError]
    #[error(transparent)]
    Arrow(#[from] arrow::error::ArrowError),

    /// [chrono::format::ParseError]
    #[error(transparent)]
    ChronoParse(#[from] chrono::format::ParseError),

    /// [duckdb::Error]
    #[error(transparent)]
    DuckDB(#[from] duckdb::Error),

    /// [geoarrow::error::GeoArrowError]
    #[error(transparent)]
    GeoArrow(#[from] geoarrow::error::GeoArrowError),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [geojson::Error]
    #[error(transparent)]
    GeoJSON(#[from] Box<geojson::Error>),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),
}

/// A crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A client for making DuckDB requests for STAC objects.
#[derive(Debug)]
pub struct Client {
    connection: Connection,
    config: Config,
}

/// Configuration for a client.
#[derive(Debug)]
pub struct Config {
    /// Whether to enable the s3 credential chain, which allows s3:// url access.
    ///
    /// True by default.
    pub use_s3_credential_chain: bool,

    /// Whether to enable hive partitioning.
    ///
    /// False by default.
    pub use_hive_partitioning: bool,
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
        Client::with_config(Config::default())
    }

    /// Creates a new client with the provided configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_duckdb::{Client, Config};
    ///
    /// let config = Config {
    ///     use_s3_credential_chain: true,
    ///     use_hive_partitioning: true,
    /// };
    /// let client = Client::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> Result<Client> {
        let connection = Connection::open_in_memory()?;
        connection.execute("INSTALL spatial", [])?;
        connection.execute("LOAD spatial", [])?;
        connection.execute("INSTALL icu", [])?;
        connection.execute("LOAD icu", [])?;
        if config.use_s3_credential_chain {
            connection.execute("CREATE SECRET (TYPE S3, PROVIDER CREDENTIAL_CHAIN)", [])?;
        }
        Ok(Client { connection, config })
    }

    /// Returns one or more [stac::Collection] from the items in the stac-geoparquet file.
    pub fn collections(&self, href: &str) -> Result<Vec<Collection>> {
        let start_datetime= if self.connection.prepare(&format!(
            "SELECT column_name FROM (DESCRIBE SELECT * from {}) where column_name = 'start_datetime'",
            self.read_parquet_str(href)
        ))?.query([])?.next()?.is_some() {
            "strftime(min(coalesce(start_datetime, datetime)), '%xT%X%z')"
        } else {
            "strftime(min(datetime), '%xT%X%z')"
        };
        let end_datetime = if self
            .connection
            .prepare(&format!(
            "SELECT column_name FROM (DESCRIBE SELECT * from {}) where column_name = 'end_datetime'",
            self.read_parquet_str(href)
        ))?
            .query([])?
            .next()?
            .is_some()
        {
            "strftime(max(coalesce(end_datetime, datetime)), '%xT%X%z')"
        } else {
            "strftime(max(datetime), '%xT%X%z')"
        };
        let mut statement = self.connection.prepare(&format!(
            "SELECT DISTINCT collection FROM {}",
            self.read_parquet_str(href)
        ))?;
        let mut collections = Vec::new();
        for row in statement.query_map([], |row| row.get::<_, String>(0))? {
            let collection_id = row?;
            let mut statement = self.connection.prepare(&
                format!("SELECT ST_AsGeoJSON(ST_Extent_Agg(geometry)), {}, {} FROM {} WHERE collection = $1", start_datetime, end_datetime,
                self.read_parquet_str(href)
            ))?;
            let row = statement.query_row([&collection_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            let mut collection = Collection::new(collection_id, DEFAULT_COLLECTION_DESCRIPTION);
            let geometry: geo::Geometry = Geometry::from_json_value(serde_json::from_str(&row.0)?)
                .map_err(Box::new)?
                .try_into()
                .map_err(Box::new)?;
            if let Some(bbox) = geometry.bounding_rect() {
                collection.extent.spatial = SpatialExtent {
                    bbox: vec![bbox.into()],
                };
            }
            collection.extent.temporal = TemporalExtent {
                interval: vec![[
                    Some(DateTime::parse_from_str(&row.1, "%FT%T%#z")?.into()),
                    Some(DateTime::parse_from_str(&row.2, "%FT%T%#z")?.into()),
                ]],
            };
            collections.push(collection);
        }
        Ok(collections)
    }

    /// Searches this client, returning a [stac::ItemCollection].
    pub fn search(&self, href: &str, search: impl Into<Search>) -> Result<stac::ItemCollection> {
        let record_batches = self.search_to_arrow(href, search)?;
        if record_batches.is_empty() {
            return Ok(Vec::new().into());
        }
        let schema = record_batches[0].schema();
        let table = Table::try_new(record_batches, schema)?;
        let items = stac::geoarrow::from_table(table)?;
        Ok(items)
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
        let record_batches = self.search_to_arrow(href, search)?;
        if record_batches.is_empty() {
            return Ok(Vec::new().into());
        }
        let schema = record_batches[0].schema();
        let table = Table::try_new(record_batches, schema)?;
        let items = stac::geoarrow::json::from_table(table)?;
        let item_collection = stac_api::ItemCollection::new(items)?;
        Ok(item_collection)
    }

    /// Searches this client, returning a vector of all matched record batches.
    pub fn search_to_arrow(
        &self,
        href: &str,
        search: impl Into<Search>,
    ) -> Result<Vec<RecordBatch>> {
        let query = self.query(search, href)?;
        let mut statement = self.connection.prepare(&query.sql)?;
        log::debug!("DuckDB SQL: {}", query.sql);
        statement
            .query_arrow(duckdb::params_from_iter(query.params))?
            .map(to_geoarrow_record_batch)
            .collect::<Result<_>>()
    }

    fn query(&self, search: impl Into<Search>, href: &str) -> Result<Query> {
        let mut search: Search = search.into();
        // Get suffix information early so we can take ownership of other parts of search as we go along.
        let limit = search.items.limit.take();
        let offset = search
            .items
            .additional_fields
            .get("offset")
            .and_then(|v| v.as_i64());
        let sortby = std::mem::take(&mut search.items.sortby);
        let fields = std::mem::take(&mut search.items.fields);

        let mut statement = self.connection.prepare(&format!(
            "SELECT column_name FROM (DESCRIBE SELECT * from {})",
            self.read_parquet_str(href)
        ))?;
        let mut columns = Vec::new();
        // Can we use SQL magic to make our query not depend on which columns are present?
        let mut has_start_datetime = false;
        let mut has_end_datetime: bool = false;
        for row in statement.query_map([], |row| row.get::<_, String>(0))? {
            let column = row?;
            if column == "start_datetime" {
                has_start_datetime = true;
            }
            if column == "end_datetime" {
                has_end_datetime = true;
            }

            if let Some(fields) = fields.as_ref() {
                if fields.exclude.contains(&column)
                    || !(fields.include.is_empty() || fields.include.contains(&column))
                {
                    continue;
                }
            }

            if column == "geometry" {
                columns.push("ST_AsWKB(geometry) geometry".to_string());
            } else {
                columns.push(format!("\"{}\"", column));
            }
        }

        let mut wheres = Vec::new();
        let mut params = Vec::new();
        if !search.ids.is_empty() {
            wheres.push(format!(
                "id IN ({})",
                (0..search.ids.len())
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            params.extend(search.ids.into_iter().map(Value::Text));
        }
        if let Some(intersects) = search.intersects {
            wheres.push("ST_Intersects(geometry, ST_GeomFromGeoJSON(?))".to_string());
            params.push(Value::Text(intersects.to_string()));
        }
        if !search.collections.is_empty() {
            wheres.push(format!(
                "collection IN ({})",
                (0..search.collections.len())
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            params.extend(search.collections.into_iter().map(Value::Text));
        }
        if let Some(bbox) = search.items.bbox {
            wheres.push("ST_Intersects(geometry, ST_GeomFromGeoJSON(?))".to_string());
            params.push(Value::Text(bbox.to_geometry().to_string()));
        }
        if let Some(datetime) = search.items.datetime {
            let interval = stac::datetime::parse(&datetime)?;
            if let Some(start) = interval.0 {
                wheres.push(format!(
                    "?::TIMESTAMPTZ <= {}",
                    if has_start_datetime {
                        "start_datetime"
                    } else {
                        "datetime"
                    }
                ));
                params.push(Value::Text(start.to_rfc3339()));
            }
            if let Some(end) = interval.1 {
                wheres.push(format!(
                    "?::TIMESTAMPTZ >= {}", // Inclusive, https://github.com/radiantearth/stac-spec/pull/1280
                    if has_end_datetime {
                        "end_datetime"
                    } else {
                        "datetime"
                    }
                ));
                params.push(Value::Text(end.to_rfc3339()));
            }
        }
        if search.items.filter.is_some() {
            todo!("Implement the filter extension");
        }
        if search.items.query.is_some() {
            todo!("Implement the query extension");
        }

        let mut suffix = String::new();
        if !wheres.is_empty() {
            suffix.push_str(&format!(" WHERE {}", wheres.join(" AND ")));
        }
        if !sortby.is_empty() {
            let mut order_by = Vec::with_capacity(sortby.len());
            for sortby in sortby {
                order_by.push(format!(
                    "{} {}",
                    sortby.field,
                    match sortby.direction {
                        Direction::Ascending => "ASC",
                        Direction::Descending => "DESC",
                    }
                ));
            }
            suffix.push_str(&format!(" ORDER BY {}", order_by.join(", ")));
        }
        if let Some(limit) = limit {
            suffix.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = offset {
            suffix.push_str(&format!(" OFFSET {}", offset));
        }
        Ok(Query {
            sql: format!(
                "SELECT {} FROM {}{}",
                columns.join(","),
                self.read_parquet_str(href),
                suffix,
            ),
            params,
        })
    }

    fn read_parquet_str(&self, href: &str) -> String {
        if self.config.use_hive_partitioning {
            format!(
                "read_parquet('{}', filename=true, hive_partitioning=1)",
                href
            )
        } else {
            format!("read_parquet('{}', filename=true)", href)
        }
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
            geometry_column.as_binary::<i32>().clone();
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

impl Default for Config {
    fn default() -> Self {
        Config {
            use_hive_partitioning: false,
            use_s3_credential_chain: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use geo::Geometry;
    use rstest::{fixture, rstest};
    use stac::{Bbox, Validate};
    use stac_api::{Search, Sortby};
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
        item_collection.items[0].validate().unwrap();
    }

    #[rstest]
    fn search_ids(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().ids(vec![
                    "S2A_MSIL2A_20240326T174951_R141_T13TDE_20240329T224429".to_string(),
                ]),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 1);
        assert_eq!(
            item_collection.items[0].id,
            "S2A_MSIL2A_20240326T174951_R141_T13TDE_20240329T224429"
        );
    }

    #[rstest]
    fn search_intersects(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().intersects(&Geometry::Point(geo::point! { x: -106., y: 40.5 })),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 50);
    }

    #[rstest]
    fn search_collections(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().collections(vec!["sentinel-2-l2a".to_string()]),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 100);

        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().collections(vec!["foobar".to_string()]),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 0);
    }

    #[rstest]
    fn search_bbox(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().bbox(Bbox::new(-106.1, 40.5, -106.0, 40.6)),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 50);
    }

    #[rstest]
    fn search_datetime(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().datetime("2024-12-02T00:00:00Z/.."),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 1);
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().datetime("../2024-12-02T00:00:00Z"),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 99);
    }

    #[rstest]
    fn search_limit(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default().limit(42),
            )
            .unwrap();
        assert_eq!(item_collection.items.len(), 42);
    }

    #[rstest]
    fn search_offset(client: Client) {
        let mut search = Search::default().limit(1);
        search
            .items
            .additional_fields
            .insert("offset".to_string(), 1.into());
        let item_collection = client
            .search("data/100-sentinel-2-items.parquet", search)
            .unwrap();
        assert_eq!(
            item_collection.items[0].id,
            "S2A_MSIL2A_20241201T175721_R141_T13TDE_20241201T213150"
        );
    }

    #[rstest]
    fn search_sortby(client: Client) {
        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default()
                    .sortby(vec![Sortby::asc("datetime")])
                    .limit(1),
            )
            .unwrap();
        assert_eq!(
            item_collection.items[0].id,
            "S2A_MSIL2A_20240326T174951_R141_T13TDE_20240329T224429"
        );

        let item_collection = client
            .search(
                "data/100-sentinel-2-items.parquet",
                Search::default()
                    .sortby(vec![Sortby::desc("datetime")])
                    .limit(1),
            )
            .unwrap();
        assert_eq!(
            item_collection.items[0].id,
            "S2B_MSIL2A_20241203T174629_R098_T13TDE_20241203T211406"
        );
    }

    #[rstest]
    fn search_fields(client: Client) {
        let item_collection = client
            .search_to_json(
                "data/100-sentinel-2-items.parquet",
                Search::default().fields("+id".parse().unwrap()).limit(1),
            )
            .unwrap();
        assert_eq!(item_collection.items[0].len(), 1);
    }

    #[rstest]
    fn collections(client: Client) {
        let collections = client
            .collections("data/100-sentinel-2-items.parquet")
            .unwrap();
        assert_eq!(collections.len(), 1);
    }
}
