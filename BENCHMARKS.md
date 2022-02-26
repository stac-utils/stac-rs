# Benchmarks

We use [criterion](https://docs.rs/criterion/latest/criterion/) for benchmarking.
Per the [user guide's recommendation](https://bheisler.github.io/criterion.rs/book/faq.html#how-should-i-run-criterionrs-benchmarks-in-a-ci-pipeline), we do not run benchmarks via Github Actions.
This repository contains [scripts/bench](./scripts/bench) to run benchmarks and copy their output into [benches/reports](./benches/reports).
To run the benchmarks, you'll need [cargo-criterion](https://github.com/bheisler/cargo-criterion):

```shell
cargo install cargo-criterion
```

Then, run the script:

```shell
scripts/bench
```

## Results

These results are from benchmarking runs on @gadomski's laptop.
YMMV.

### Layout

These benchmarks test how long it takes to lay out a `Stac`, i.e. set each object's href and links.

![layout items lines](./benches/reports/layout-items/lines.svg)

#### 1 item

![layout items 1](./benches/reports/layout-items/1/pdf.svg)

#### 10 item

![layout items 10](./benches/reports/layout-items/10/pdf.svg)

#### 100 item

![layout items 100](./benches/reports/layout-items/100/pdf.svg)

#### 1000 item

![layout items 1000](./benches/reports/layout-items/1000/pdf.svg)

#### 10000 item

![layout items 10000](./benches/reports/layout-items/10000/pdf.svg)

### Read

These benchmarks test how long it takes to read a STAC JSON from the local filesystem into an `Object`.
It's not surprising there is some variability, since they require filesystem access.

#### Read item

Reading [data/simple-item.json](data/simple-item.json):

![read item](./benches/reports/read-item/pdf.svg)

#### Read catalog

Reading [data/catalog.json](data/catalog.json):

![read catalog](./benches/reports/read-catalog/pdf.svg)

#### Read collection

Reading [data/collection.json](data/collection.json):

![read collection](./benches/reports/read-collection/pdf.svg)
