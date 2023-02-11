# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `From<Vec<Collection>>` for `Collections` ([#124](https://github.com/gadomski/stac-rs/pull/124))
- `UrlBuilder` ([#129](https://github.com/gadomski/stac-rs/pull/129))
- New `LinkBuilder` methods, including some renames ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Links` for `Collections`, `ItemCollection` ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Conformance` structure ([#126](https://github.com/gadomski/stac-rs/pull/126))
- `Default` for `Search` ([#126](https://github.com/gadomski/stac-rs/pull/126))

### Changed

- `ItemCollection` now has a `items` attribute, instead of `features` ([#126](https://github.com/gadomski/stac-rs/pull/126))

### Removed

- `Link` was removed, STAC API link attributes were added to `stac::Link` ([#126](https://github.com/gadomski/stac-rs/pull/126))

## [0.1.0] - 2023-01-14

Initial release

[unreleased]: https://github.com/gadomski/stac-rs/compare/stac-api-v0.1.0...main
[0.1.0]: https://github.com/gadomski/stac-rs/releases/tag/stac-api-v0.1.0
