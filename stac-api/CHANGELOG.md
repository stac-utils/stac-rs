# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2023-04-03

### Added

- `From<Vec<Collection>>` for `Collections` ([#124](https://github.com/gadomski/stac-rs/pull/124))
- `UrlBuilder` ([#129](https://github.com/gadomski/stac-rs/pull/129), [#130](https://github.com/gadomski/stac-rs/pull/130))
- New `LinkBuilder` methods, including some renames ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Links` for `Collections`, `ItemCollection` ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Conformance` structure ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Default` for `Search` ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Clone` for `Search` and its sub structs ([#130](https://github.com/gadomski/stac-rs/pull/130))
- `Display` for `Fields` and `Sortby` ([#133](https://github.com/gadomski/stac-rs/pull/133))
- `Filter` as an externally-tagged enum ([#133](https://github.com/gadomski/stac-rs/pull/133))
- `Items` and `GetItems` for paging items ([#133](https://github.com/gadomski/stac-rs/pull/133))

### Changed

- `ItemCollection` now has a `items` attribute, instead of `features` ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Item` is now just a type alias ([#130](https://github.com/gadomski/stac-rs/pull/130))
- All `Search` fields are now optional ([#130](https://github.com/gadomski/stac-rs/pull/130))

### Removed

- `Link` was removed, STAC API link attributes were added to `stac::Link` ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Sortby::from_query_param` ([#133](https://github.com/gadomski/stac-rs/pull/133))

## [0.1.0] - 2023-01-14

Initial release

[unreleased]: https://github.com/gadomski/stac-rs/compare/stac-api-v0.2.0...main
[0.2.0]: https://github.com/gadomski/stac-rs/compare/stac-api-v0.1.0...stac-api-v0.2.0
[0.1.0]: https://github.com/gadomski/stac-rs/releases/tag/stac-api-v0.1.0
