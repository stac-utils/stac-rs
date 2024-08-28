# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Bbox` ([#303](https://github.com/stac-utils/stac-rs/pull/303))
- Functions to create collections from items ([#304](https://github.com/stac-utils/stac-rs/pull/304))

### Changed

- Use `DateTime<Utc>` instead of `String` for datetimes ([#297](https://github.com/stac-utils/stac-rs/pull/297), [#304](https://github.com/stac-utils/stac-rs/pull/304))
- Add `Href.clear_href` ([#299](https://github.com/stac-utils/stac-rs/pull/299))

### Removed

- `stac::read_json` from the public API ([#301](https://github.com/stac-utils/stac-rs/pull/301))
- `Bounds` ([#303](https://github.com/stac-utils/stac-rs/pull/303))

## [0.8.0] - 2024-08-12

### Added

- `Version` enum ([#293](https://github.com/stac-utils/stac-rs/pull/293))
- STAC v1.1.0-beta.1 ([#294](https://github.com/stac-utils/stac-rs/pull/294)):
  - `Asset::bands` vec and `Band` struct
  - `nodata`, `data_type`, `statistics`, and `unit` to `Asset`
- Migrations via a `Migrate` trait ([#294](https://github.com/stac-utils/stac-rs/pull/294))

### Changed

- `STAC_VERSION` is now a `Version`, not a string ([#293](https://github.com/stac-utils/stac-rs/pull/293))
- `version` fields on collection, catalog, and item are now public ([#293](https://github.com/stac-utils/stac-rs/pull/293))
- `DataType` and `Statistics` structures are now top-level instead of being in the `raster` extension ([#294](https://github.com/stac-utils/stac-rs/pull/294))

## [0.7.2] - 2024-07-24

### Fixed

- `Bounds::to_geometry` ([#279](https://github.com/stac-utils/stac-rs/pull/279))

## [0.7.1] - 2024-07-24

### Added

- `stac::datetime::Interval` ([#252](https://github.com/stac-utils/stac-rs/pull/252))
- `TryFrom<Feature>` and `TryInto<Feature>` for `Item` ([#255](https://github.com/stac-utils/stac-rs/pull/255))
- `FlatItem` ([#260](https://github.com/stac-utils/stac-rs/pull/260), [#263](https://github.com/stac-utils/stac-rs/pull/263), [#275](https://github.com/stac-utils/stac-rs/pull/275))
- Authentication extension ([#268](https://github.com/stac-utils/stac-rs/pull/268))
- Electro-Optical extension ([#271](https://github.com/stac-utils/stac-rs/pull/271))

### Fixed

- Removed unnecessary qualifications ([#251](https://github.com/stac-utils/stac-rs/pull/251))
- Axis ordering for WGS84 bounds+geometries ([#277](https://github.com/stac-utils/stac-rs/pull/277))

## [0.7.0] - 2024-04-29

### Added

- `Link::method` and `Link::body` ([#244](https://github.com/stac-utils/stac-rs/pull/244))

### Changed

- Use `geojson::Geometry` instead of a crate-specific `Geometry` type ([#245](https://github.com/stac-utils/stac-rs/pull/245))

### Removed

- `schemars` feature ([#245](https://github.com/stac-utils/stac-rs/pull/245))

## [0.6.0] - 2024-04-11

### Added

- The projection and raster extensions, the `Extension` trait, and the `Fields` trait ([#234](https://github.com/stac-utils/stac-rs/pull/234))
- `stac::item::Builder` ([#237](https://github.com/stac-utils/stac-rs/pull/237))
- The `gdal` feature ([#232](https://github.com/stac-utils/stac-rs/pull/232), [#240](https://github.com/stac-utils/stac-rs/pull/240))
- `Bounds` ([#232](https://github.com/stac-utils/stac-rs/pull/232))

### Changed

- The `extensions` attribute of catalogs, collections, and items is now non-optional ([#234](https://github.com/stac-utils/stac-rs/pull/234))
- The `roles` attribute of assets is now non-optional ([#237](https://github.com/stac-utils/stac-rs/pull/237))

## [0.5.3] - 2024-04-07

### Added

- `Geometry::point` ([#206](https://github.com/stac-utils/stac-rs/pull/206))
- `Item::intersects_datetime_str` ([#209](https://github.com/stac-utils/stac-rs/pull/209))

## [0.5.2] - 2023-10-18

### Added

- `Item::intersects` ([#202](https://github.com/stac-utils/stac-rs/pull/202))
- Common metadata fields ([#203](https://github.com/stac-utils/stac-rs/pull/203))

### Deprecated

- `Item::intersects_bbox` ([#204](https://github.com/stac-utils/stac-rs/pull/204))

## [0.5.1] - 2023-09-14

### Added

- `geo` feature ([#178](https://github.com/stac-utils/stac-rs/pull/178))
- `schemars` feature ([#177](https://github.com/stac-utils/stac-rs/pull/177))
- `intersects_bbox` and `intersects_datetimes` for `Item` ([#180](https://github.com/stac-utils/stac-rs/pull/180), [#182](https://github.com/stac-utils/stac-rs/pull/182))
- `geo` and `datetime` modules ([#182](https://github.com/stac-utils/stac-rs/pull/182))
- `(de)serialize_type` to the public interface ([#187](https://github.com/stac-utils/stac-rs/pull/187))

### Changed

- `Links::remove_relative_links` has the same vibe as `Links::remove_structural_links` ([#176](https://github.com/stac-utils/stac-rs/pull/176))
- Use our own `Geometry` structure ([#178](https://github.com/stac-utils/stac-rs/pull/178))

## [0.5.0] - 2023-06-27

### Added

- `Links::remove_structural_links`, and more rel types to `Link::is_structural` ([#170](https://github.com/stac-utils/stac-rs/pull/170))
- `Item::set_geometry` ([#172](https://github.com/stac-utils/stac-rs/pull/172))

### Removed

- `Link::set_query` ([#171](https://github.com/stac-utils/stac-rs/pull/171))
- `jsonschema` feature (it's now in its own crate, **stac-validate**) ([#172](https://github.com/stac-utils/stac-rs/pull/172))

## [0.4.0] - 2023-04-01

### Added

- `Deserialize` for `Value` ([#135](https://github.com/stac-utils/stac-rs/pull/135))
- `type` checks on (de)serialization ([#136](https://github.com/stac-utils/stac-rs/pull/136))
- `Assets` trait ([#137](https://github.com/stac-utils/stac-rs/pull/137))
- `Link::remove_relative_hrefs` ([#142](https://github.com/stac-utils/stac-rs/pull/142))
- `stac::href_to_url` ([#142](https://github.com/stac-utils/stac-rs/pull/142))
- `TryFrom<Map<String, Value>>` for all three object types ([#149](https://github.com/stac-utils/stac-rs/pull/149))
- `FromIterator` for `ItemCollection` ([#151](https://github.com/stac-utils/stac-rs/pull/151))

### Changed

- `stac::read` now can return anything that deserializes and implements `Href` ([#135](https://github.com/stac-utils/stac-rs/pull/135))
- `Collection::assets` is now non-optional ([#137](https://github.com/stac-utils/stac-rs/pull/137))
- `type` and `version` fields on all objects are now private ([#141](https://github.com/stac-utils/stac-rs/pull/141))

## [0.3.2] - 2023-02-19

### Added

- STAC API fields to `Link` ([#126](https://github.com/stac-utils/stac-rs/pull/126)])
- `TryFrom<Value>` (and `TryFrom<Item>` and friends) for a `serde_json::Map<String, serde_json::Value>` ([#126](https://github.com/stac-utils/stac-rs/pull/126), [#130](https://github.com/stac-utils/stac-rs/pull/130))

## [0.3.1] - 2023-01-13

### Added

- `Item::collection` setter in the builder pattern ([#117](https://github.com/stac-utils/stac-rs/pull/117))
- `Link::geojson` ([#119](https://github.com/stac-utils/stac-rs/pull/119))
- `Link::is_json` and `Link::is_geojson` ([#120](https://github.com/stac-utils/stac-rs/pull/120))
- `Link::set_query` ([#127](https://github.com/stac-utils/stac-rs/pull/127))

## [0.3.0] - 2023-01-08

### Changed

- Reorganized to a workspace ([#114](https://github.com/stac-utils/stac-rs/pull/114))
- `ItemCollection::links` is now public ([#115](https://github.com/stac-utils/stac-rs/pull/115))
- `Links::make_relative_links_absolute` takes the href as an argument, and `Links` does not require `Href` ([#116](https://github.com/stac-utils/stac-rs/pull/116))

## [0.2.0] - 2022-12-29

### Added

- `description` to `Catalog::new` and `Collection::new` ([#102](https://github.com/stac-utils/stac-rs/pull/102))
- jsonschema validation ([#105](https://github.com/stac-utils/stac-rs/pull/105), [#106](https://github.com/stac-utils/stac-rs/pull/106))
- `Extensions` trait ([#105](https://github.com/stac-utils/stac-rs/pull/105))
- `ItemCollection` ([#107](https://github.com/stac-utils/stac-rs/pull/107))
- `Value::type_name` ([#108](https://github.com/stac-utils/stac-rs/pull/108))
- `Links::make_relative_hrefs_absolute` ([#110](https://github.com/stac-utils/stac-rs/pull/110))

### Changed

- Signature of `Error::MissingHref` (no longer takes a String) ([#110](https://github.com/stac-utils/stac-rs/pull/110))
- `Links` now requires `Href` ([#110](https://github.com/stac-utils/stac-rs/pull/110))

### Fixed

- `Collections::new` sets a valid license ("proprietary") ([#104](https://github.com/stac-utils/stac-rs/pull/104))

### Removed

- `Error::TypeMismatch`, deprecated since v0.1.1 ([#111](https://github.com/stac-utils/stac-rs/pull/111))

## [0.1.2] - 2022-12-08

### Added

- `Links` trait ([#95](https://github.com/stac-utils/stac-rs/pull/95), [#96](https://github.com/stac-utils/stac-rs/pull/96), [#97](https://github.com/stac-utils/stac-rs/pull/97), [#100](https://github.com/stac-utils/stac-rs/pull/100))
- `Link::json` for making links with a JSON media type ([#100](https://github.com/stac-utils/stac-rs/pull/100))
- Builder methods for `Link` ([#100](https://github.com/stac-utils/stac-rs/pull/100))
- `Item::collection_link` ([#100](https://github.com/stac-utils/stac-rs/pull/100))
- `Link::collection` for making `rel="collection"` links ([#100](https://github.com/stac-utils/stac-rs/pull/100))
- `Link::self_` for making `rel="self"` links ([#101](https://github.com/stac-utils/stac-rs/pull/101))

## [0.1.1] - 2022-12-01

### Deprecated

- `Error::TypeMismatch` ([#93](https://github.com/stac-utils/stac-rs/pull/93))

## [0.1.0] - 2022-11-30

### Added

- `stac::read_json`

### Changed

- Module layout

### Removed

- `Item::set_collection` and `Item::collection_link`
- CI coverage
- `stac::read_from_url` and `stac::read_from_path`

### Fixed

- Documentation examples

## [0.0.5] - 2022-11-12

### Added

- `Href` trait
- `Value`

### Changed

- `Asset.type_` is now `Asset.r#type`
- `reqwest` is now an optional feature

### Removed

- `PathBufHref`
- `Href::into_string`
- `Stac`
- `Layout`
- `Reader`
- `Writer`
- `Object`
- `Href` struct
- benchmarks

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

[Unreleased]: https://github.com/stac-utils/stac-rs/compare/stac-v0.8.0...main
[0.8.0]: https://github.com/stac-utils/stac-rs/compare/stac-v0.7.2...stac-v0.8.0
[0.7.2]: https://github.com/stac-utils/stac-rs/compare/stac-v0.7.1...stac-v0.7.2
[0.7.1]: https://github.com/stac-utils/stac-rs/compare/stac-v0.7.0...stac-v0.7.1
[0.7.0]: https://github.com/stac-utils/stac-rs/compare/stac-v0.6.0...stac-v0.7.0
[0.6.0]: https://github.com/stac-utils/stac-rs/compare/stac-v0.5.3...stac-v0.6.0
[0.5.3]: https://github.com/stac-utils/stac-rs/compare/stac-v0.5.2...stac-v0.5.3
[0.5.2]: https://github.com/stac-utils/stac-rs/compare/stac-v0.5.1...stac-v0.5.2
[0.5.1]: https://github.com/stac-utils/stac-rs/compare/stac-v0.5.0...stac-v0.5.1
[0.5.0]: https://github.com/stac-utils/stac-rs/compare/stac-v0.4.0...stac-v0.5.0
[0.4.0]: https://github.com/stac-utils/stac-rs/compare/stac-v0.3.2...stac-v0.4.0
[0.3.2]: https://github.com/stac-utils/stac-rs/compare/stac-v0.3.1...stac-v0.3.2
[0.3.1]: https://github.com/stac-utils/stac-rs/compare/stac-v0.3.0...stac-v0.3.1
[0.3.0]: https://github.com/stac-utils/stac-rs/compare/v0.2.0...stac-v0.3.0
[0.2.0]: https://github.com/stac-utils/stac-rs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/stac-utils/stac-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/stac-utils/stac-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/stac-utils/stac-rs/compare/v0.0.5...v0.1.0
[0.0.5]: https://github.com/stac-utils/stac-rs/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/stac-utils/stac-rs/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/stac-utils/stac-rs/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/stac-utils/stac-rs/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/stac-utils/stac-rs/releases/tag/v0.0.1

<!-- markdownlint-disable-file MD024 -->
