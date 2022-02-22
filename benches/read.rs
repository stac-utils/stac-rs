use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn read_item(c: &mut Criterion) {
    c.bench_function("read-item", |b| {
        b.iter(|| stac::read(black_box("data/simple-item.json")).unwrap())
    });
}

pub fn read_collection(c: &mut Criterion) {
    c.bench_function("read-collection", |b| {
        b.iter(|| stac::read(black_box("data/collection.json")).unwrap())
    });
}

pub fn read_catalog(c: &mut Criterion) {
    c.bench_function("read-catalog", |b| {
        b.iter(|| stac::read(black_box("data/catalog.json")).unwrap())
    });
}

criterion_group!(read, read_item, read_collection, read_catalog);
criterion_main!(read);
