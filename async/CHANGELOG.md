# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.3] - 2024-09-05

### Changed

- Bump **stac** and **stac-api** versions

## [0.5.2] - 2024-08-12

### Added

- `impl Default for Client` ([#252](https://github.com/stac-utils/stac-rs/pull/252))

### Fixed

- Better send message handling in the api client ([#288](https://github.com/stac-utils/stac-rs/pull/288))

## [0.5.1] - 2024-04-29

### Changed

- Updated **stac** and **stac-api** dependencies

## [0.5.0] - 2024-04-07

### Added

- `ApiClient::with_client` ([#227](https://github.com/stac-utils/stac-rs/pull/227))

### Removed

- Downloading (use [stac-asset](https://github.com/stac-utils/stac-asset) instead) ([#194](https://github.com/stac-utils/stac-rs/pull/194))

## [0.4.1] - 2023-09-25

### Changed

- Update `stac-api` major version

## [0.4.0] - 2023-04-03

### Added

- `ApiClient` ([#130](https://github.com/stac-utils/stac-rs/pull/130))
- `Client::post` ([#130](https://github.com/stac-utils/stac-rs/pull/130))
- Item paging ([#133](https://github.com/stac-utils/stac-rs/pull/133))
- `Client::request` and `Client::request_from_link` ([#133](https://github.com/stac-utils/stac-rs/pull/133))
- Mocks for testing ([#133](https://github.com/stac-utils/stac-rs/pull/133))
- Downloading ([#142](https://github.com/stac-utils/stac-rs/pull/142), [#152](https://github.com/stac-utils/stac-rs/pull/152))

### Changed

- Refactored to modules ([#130](https://github.com/stac-utils/stac-rs/pull/130))
- `stac_async::read` now can return anything that deserializes and implements `Href` ([#135](https://github.com/stac-utils/stac-rs/pull/135))

### Fixed

- Reading Windows hrefs ([#142](https://github.com/stac-utils/stac-rs/pull/142))

## [0.3.0] - 2023-01-08

No changes.

## [0.2.0] - 2022-12-29

Initial release of **stac-async**.

[Unreleased]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.5.3...main
[0.5.3]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.5.2...stac-async-v0.5.3
[0.5.2]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.5.1...stac-async-v0.5.2
[0.5.1]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.5.0...stac-async-v0.5.1
[0.5.0]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.4.1...stac-async-v0.5.0
[0.4.1]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.4.0...stac-async-v0.4.1
[0.4.0]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.3.0...stac-async-v0.4.0
[0.3.0]: https://github.com/stac-utils/stac-rs/compare/stac-async-v0.2.0...stac-async-v0.3.0
[0.2.0]: https://github.com/stac-utils/stac-rs/releases/tag/stac-async-v0.2.0

<!-- markdownlint-disable-file MD024 -->
