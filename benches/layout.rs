use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use stac::{Catalog, Item, Layout, Result, Stac};

fn layout_items(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("layout-items");
    group.plot_config(plot_config);
    for items in [1, 10, 100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(items), items, |b, &items| {
            let (mut stac, root) = Stac::new(Catalog::new("an-id")).unwrap();
            for i in 0..items {
                stac.add_child(root, Item::new(format!("item-{}", i)))
                    .unwrap();
            }
            let layout = Layout::new("root");
            b.iter(|| {
                let _ = layout
                    .layout(&mut stac)
                    .collect::<Result<Vec<()>>>()
                    .unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(layout, layout_items);
criterion_main!(layout);
