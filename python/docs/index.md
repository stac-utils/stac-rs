# stacrs

A small, no-dependency Python library and command-line interface (CLI) for working with [STAC](https://stacspec.org), using [Rust](https://github.com/stac-utils/stac-rs) under the hood.

## Installation

```shell
pip install stacrs
```

## Usage

### API

```python
import stacrs

stacrs.validate_href("https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json")
```

See [the API documentation](./api.md) for more.

### CLI

```shell
$ stacrs --help
Command line interface for stac-rs

Usage: stacrs [OPTIONS] <COMMAND>

Commands:
  item       Creates a STAC Item
  migrate    Migrates a STAC value from one version to another
  search     Searches a STAC API
  serve      Serves a STAC API
  sort       Sorts the fields of STAC object
  translate  Translates STAC values between formats
  validate   Validates a STAC object or API endpoint using json-schema validation
  help       Print this message or the help of the given subcommand(s)

Options:
  -c, --compact
          Use a compact representation of the output, if possible

  -i, --input-format <INPUT_FORMAT>
          The input format. If not provided, the format will be detected from the input file extension when possible

          Possible values:
          - parquet: stac-geoparquet
          - json:    JSON (the default)

  -o, --output-format <OUTPUT_FORMAT>
          The output format. If not provided, the format will be detected from the output file extension when possible

          Possible values:
          - parquet: stac-geoparquet
          - json:    JSON (the default)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
