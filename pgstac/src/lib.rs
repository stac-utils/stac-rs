//! Rust interface for [pgstac](https://github.com/stac-utils/pgstac)
//!
//! # Examples
//!
//! [Client] provides an interface to query a **pgstac** database.
//! It can be created from anything that implements [tokio_postgres::GenericClient].
//!
//! ```
//! use pgstac::Client;
//! use tokio_postgres::NoTls;
//!
//! # tokio_test::block_on(async {
//! let config = "postgresql://username:password@localhost:5432/postgis";
//! let (client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
//! let client = Client::new(&client);
//! # })
//! ```
//!
//! If you want to work in a transaction, you can do that too:
//!
//! ```no_run
//! use stac::Collection;
//! # use pgstac::Client;
//! # use tokio_postgres::NoTls;
//! # tokio_test::block_on(async {
//! # let config = "postgresql://username:password@localhost:5432/postgis";
//! let (mut client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
//! let transaction = client.transaction().await.unwrap();
//! let client = Client::new(&transaction);
//! client.add_collection(Collection::new("an-id", "a description")).await.unwrap();
//! transaction.commit().await.unwrap();
//! # })
//! ```
//!
//! # Features
//!
//! - `tls`: provide a function to create an unverified tls provider, which can be useful in some circumstances (see <https://github.com/stac-utils/stac-rs/issues/375>)

#![deny(missing_docs)]

mod client;
mod page;
#[cfg(feature = "tls")]
mod tls;

pub use {client::Client, page::Page};
#[cfg(feature = "tls")]
pub use {tls::make_unverified_tls, tokio_postgres_rustls::MakeRustlsConnect};

/// Crate-specific error enum.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A boxed error.
    ///
    /// Used to capture generic errors from [tokio_postgres::types::FromSql].
    #[error(transparent)]
    Boxed(#[from] Box<dyn std::error::Error + Sync + Send>),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [tokio_postgres::Error]
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),

    /// An unknown error.
    ///
    /// Used when [tokio_postgres::types::FromSql] doesn't have a source.
    #[error("unknown error")]
    Unknown,
}

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;
