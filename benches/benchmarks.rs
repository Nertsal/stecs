mod bevy;
mod gecs;
mod hecs;
mod legion;
mod stecs;
// mod zero; // Disabled because it requires a build script

use criterion::{criterion_group, measurement::Measurement, BenchmarkGroup, Criterion};

/// Entities for simple benches.
pub const N_ENTITIES: usize = 10_000;
/// Entities for fragmented benches.
pub const N_ENTITIES_FRAG: usize = 20;
/// Entities for filter benches.
pub const N_ENTITIES_FILTER: usize = 10_000;
/// Entities with velocity component for filter benches.
pub const N_ENTITIES_FILTER_VEL: usize = 5_000;

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_insert");
    configure_group(&mut group);
    group.bench_function("stecs", |b| {
        let mut bench = stecs::simple_insert::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("bevy_ecs", |b| {
        let mut bench = bevy::simple_insert::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("hecs", |b| {
        let mut bench = hecs::simple_insert::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("legion", |b| {
        let mut bench = legion::simple_insert::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("gecs", |b| {
        let mut bench = gecs::simple_insert::Benchmark::new();
        b.iter(|| bench.run())
    });
    // group.bench_function("zero_ecs", |b| {
    //     let mut bench = zero::simple_insert::Benchmark::new();
    //     b.iter(|| bench.run())
    // });
    group.finish();
}

fn bench_simple_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_iter");
    configure_group(&mut group);
    group.bench_function("stecs", |b| {
        let mut bench = stecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("bevy_ecs", |b| {
        let mut bench = bevy::simple_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("hecs", |b| {
        let mut bench = hecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("legion", |b| {
        let mut bench = legion::simple_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("gecs", |b| {
        let mut bench = gecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    // group.bench_function("zero_ecs", |b| {
    //     let mut bench = zero::simple_iter::Benchmark::new();
    //     b.iter(|| bench.run())
    // });
    group.finish();
}

fn bench_fragmented_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmented_iter");
    configure_group(&mut group);
    group.bench_function("stecs", |b| {
        let mut bench = stecs::fragmented_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("bevy_ecs", |b| {
        let mut bench = bevy::fragmented_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("hecs", |b| {
        let mut bench = hecs::fragmented_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("legion", |b| {
        let mut bench = legion::fragmented_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    // group.bench_function("zero_ecs", |b| {
    //     let mut bench = zero::fragmented_iter::Benchmark::new();
    //     b.iter(|| bench.run())
    // });
    group.finish();
}

fn bench_filter_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_iter");
    configure_group(&mut group);
    group.bench_function("stecs", |b| {
        let mut bench = stecs::filter_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("bevy_ecs", |b| {
        let mut bench = bevy::filter_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("hecs", |b| {
        let mut bench = hecs::filter_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("legion", |b| {
        let mut bench = legion::filter_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("gecs", |b| {
        let mut bench = gecs::filter_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    // group.bench_function("zero_ecs", |b| {
    //     let mut bench = zero::filter_iter::Benchmark::new();
    //     b.iter(|| bench.run())
    // });
    group.finish();
}

fn bench_compare_manual(c: &mut Criterion) {
    let mut group = c.benchmark_group("querying methods");
    configure_group(&mut group);
    group.bench_function("macro", |b| {
        let mut bench = stecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run())
    });
    group.bench_function("semi_manual", |b| {
        let mut bench = stecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run_semi_manual())
    });
    group.bench_function("iterator_zip", |b| {
        let mut bench = stecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run_iter_zip())
    });
    group.bench_function("manual", |b| {
        let mut bench = stecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run_manual())
    });
    group.bench_function("manual_unchecked", |b| {
        let mut bench = stecs::simple_iter::Benchmark::new();
        b.iter(|| bench.run_manual_unchecked())
    });
    group.finish();
}

fn configure_group<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    group.significance_level(0.01).sample_size(500);
}

criterion_group!(
    benchmarks,
    bench_build,
    bench_simple_iter,
    bench_fragmented_iter,
    bench_filter_iter,
    bench_compare_manual,
);

fn main() {
    // Avoid parallelization for fair comparisons
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();

    benchmarks();
    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
