# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2024-09-19

### Added

- STAC v1.1.0 schemas ([#399](https://github.com/stac-utils/stac-rs/pull/399))

### Changed

- Moved to async-first, with a blocking interface ([#397](https://github.com/stac-utils/stac-rs/pull/397))

## [0.2.2] - 2024-09-06

### Added

- json-schema draft 07 and item resolver ([#358](https://github.com/stac-utils/stac-rs/pull/358))

## [0.2.1] - 2024-09-05

### Changed

- Bump **stac** version

## [0.2.0] - 2024-08-12

### Added

- `impl Default for Validator` ([#252](https://github.com/stac-utils/stac-rs/pull/252))
- Support for validating versions other than v1.0.0 ([#293](https://github.com/stac-utils/stac-rs/pull/293))

### Changed

- `ValidateCore::validate_core_json` now takes a mutable reference to the validator ([#293](https://github.com/stac-utils/stac-rs/pull/293))

### Removed

- `ValidateCore::validate_core` ([#293](https://github.com/stac-utils/stac-rs/pull/293))

## [0.1.2] - 2024-04-29

### Changed

- Updated **stac** version

## [0.1.1] - 2023-10-09

### Added

- Validation for `serde_json::Value` ([#190](https://github.com/stac-utils/stac-rs/pull/190))

## [0.1.0] - 2023-06-27

Initial release.

[Unreleased]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.3.0...main
[0.3.0]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.2.2..stac-validate-v0.3.0
[0.2.2]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.2.1..stac-validate-v0.2.2
[0.2.1]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.2.0..stac-validate-v0.2.1
[0.2.0]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.1.2..stac-validate-v0.2.0
[0.1.2]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.1.1..stac-validate-v0.1.2
[0.1.1]: https://github.com/stac-utils/stac-rs/compare/stac-validate-v0.1.0..stac-validate-v0.1.1
[0.1.0]: https://github.com/stac-utils/stac-rs/releases/tag/stac-validate-v0.1.0

<!-- markdownlint-disable-file MD024 -->
