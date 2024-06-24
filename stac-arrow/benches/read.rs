//! How fast is `RecordBatch` -> `Vec<Map<String, Value>>` when going through
//! full serialization vs the deprecated `record_batches_to_json_rows`?

use arrow::array::RecordBatch;
use arrow_json::ArrayWriter;
use criterion::{criterion_group, criterion_main, Criterion};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde_json::{Map, Value};
use std::fs::File;

#[allow(deprecated)]
fn record_batches_to_json_rows(record_batch: &RecordBatch) {
    let _ = arrow_json::writer::record_batches_to_json_rows(&[record_batch]).unwrap();
}

fn writer(record_batch: &RecordBatch) {
    let mut writer = ArrayWriter::new(Vec::new());
    writer.write(record_batch).unwrap();
    writer.finish().unwrap();
    let _: Vec<Map<String, Value>> =
        serde_json::from_reader(writer.into_inner().as_slice()).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    let file = File::open("data/naip.parquet").unwrap();
    let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .unwrap()
        .build()
        .unwrap();
    let mut record_batch = reader.next().unwrap().unwrap();
    let index = record_batch.schema().index_of("geometry").unwrap();
    record_batch.remove_column(index);
    group.bench_function("record_batches_to_json_rows", |b| {
        b.iter(|| record_batches_to_json_rows(&record_batch))
    });
    group.bench_function("writer", |b| b.iter(|| writer(&record_batch)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
