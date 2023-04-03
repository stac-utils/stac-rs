# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2023-04-03

### Added

- `ApiClient` ([#130](https://github.com/gadomski/stac-rs/pull/130))
- `Client::post` ([#130](https://github.com/gadomski/stac-rs/pull/130))
- Item paging ([#133](https://github.com/gadomski/stac-rs/pull/133))
- `Client::request` and `Client::request_from_link` ([#133](https://github.com/gadomski/stac-rs/pull/133))
- Mocks for testing ([#133](https://github.com/gadomski/stac-rs/pull/133))
- Downloading ([#142](https://github.com/gadomski/stac-rs/pull/142), [#152](https://github.com/gadomski/stac-rs/pull/152))

### Changed

- Refactored to modules ([#130](https://github.com/gadomski/stac-rs/pull/130))
- `stac_async::read` now can return anything that deserializes and implements `Href` ([#135](https://github.com/gadomski/stac-rs/pull/135))

### Fixed

- Reading Windows hrefs ([#142](https://github.com/gadomski/stac-rs/pull/142))

## [0.3.0] - 2023-01-08

No changes.

## [0.2.0] - 2022-12-29

Initial release of **stac-async**.

[Unreleased]: https://github.com/gadomski/stac-rs/compare/stac-async-v0.4.0...main
[0.4.0]: https://github.com/gadomski/stac-rs/compare/stac-async-v0.3.0...stac-async-v0.4.0
[0.3.0]: https://github.com/gadomski/stac-rs/compare/stac-async-v0.2.0...stac-async-v0.3.0
[0.2.0]: https://github.com/gadomski/stac-rs/releases/tag/stac-async-v0.2.0
