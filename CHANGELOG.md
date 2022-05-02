# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `Asset.type_` is now `Asset.r#type`
- Rename `Read::read_struct` to `Read::read_object`
- `Read::read_json` now takes a reference to a `PathBufHref`
- `reqwests` is now an optional feature

## [0.0.4] - 2022-03-09

### Added

- Top-level convenience functions for reading all three object types directly to structures
- `Read::read_struct`
- `Error::TypeMismatch`
- Links to parent and root in `Stac` when adding a new object
- `Stac::href`
- `Href::file_name`
- `Stac::collections`
- Options to customize the `Walk` strategy
- `Stac::set_href`
- Coverage
- Crate-specific `Result`
- `Href::directory`
- `impl From<Href> for String`
- `Object::parent_link` and `Object::child_links`
- `Stac::add_link` and `Stac::children`
- `stac::layout`
- Pull request template
- Docs

### Changed

- Made `Handle`s innards private
- Generalized `Stac::find_child` to `Stac::find`
- Made `PathBufHref::new` public
- Cannot remove the root of a `Stac`
- `Href::make_relative` returns an absolute href if it can't be made relative
- Benchmark plots now have white backgrounds
- Reqwest test is ignored by default to speed up unit tests
- Use `impl` in function arguments instead of generic types
- The default walk iterator's Item is a `Result<Handle>`
- Set a walk's visit function as its own operation, rather than during the constructor

### Fixed

- Relative href generation

## [0.0.3] - 2022-02-22

### Added

- Doctesting for README.md
- `Href::rebase`
- `Object` and `HrefObject`
- Architecture diagram
- `Stac.add_child`
- Benchmarks
- `Walk`
- `Stac::remove`

### Changed

- Simplified `Render`'s href creation
- CI workflows
- `Stac::add_object` is now `add`
- `Stac::add_child_handle` is now `connect`
- `Stac::object` is now `get`

### Removed

- `stac::render`

## [0.0.2] - 2022-02-14

### Added

- More information to the README

### Removed

- Custom docs build

## [0.0.1] - 2022-02-14

Initial release.

[Unreleased]: https://github.com/gadomski/stac-rs/compare/v0.0.4...HEAD
[0.0.4]: https://github.com/gadomski/stac-rs/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/gadomski/stac-rs/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/gadomski/stac-rs/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/gadomski/stac-rs/releases/tag/v0.0.1
