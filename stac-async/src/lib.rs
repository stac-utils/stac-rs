//! Asynchronous I/O for [STAC](https://stacspec.org/), built on [stac-rs](https://github.com/gadomski/stac-rs)
//!
//! This is just a small library that provides an async version of [stac::read].
//! It also includes a thin wrapper around [reqwest::Client] for efficiently
//! getting multiple STAC items in the same session.
//!
//! # Examples
//!
//! ```
//! # tokio_test::block_on(async {
//! let value = stac_async::read("data/simple-item.json").await.unwrap();
//! # })
//! ```

#![deny(missing_docs, missing_debug_implementations, unused_extern_crates)]

mod api_client;
mod client;
mod error;
mod io;

pub use {
    api_client::ApiClient,
    client::Client,
    error::Error,
    io::{read, read_json, write_json_to_path},
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::Client;
    use stac::Href;

    #[tokio::test]
    async fn read_filesystem() {
        let value = super::read("data/simple-item.json").await.unwrap();
        assert!(value.href().unwrap().ends_with("data/simple-item.json"));
    }

    #[tokio::test]
    async fn read_network() {
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
        let value = super::read(href).await.unwrap();
        assert_eq!(value.href().unwrap(), href);
    }

    #[tokio::test]
    async fn client() {
        let client = Client::new();
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
        let value = client.get(href).await.unwrap().unwrap();
        assert_eq!(value.href().unwrap(), href);
    }
}
