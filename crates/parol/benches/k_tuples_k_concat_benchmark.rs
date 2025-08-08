use criterion::{Criterion, criterion_group, criterion_main};
use parol::{KTuples, analysis::k_tuples::KTuplesBuilder};
use parol_runtime::once_cell::sync::Lazy;

static K_TUPLES_1: Lazy<KTuples> = Lazy::new(|| {
    KTuplesBuilder::new()
        .k(5)
        .max_terminal_index(3)
        .terminal_indices(&[&[1, 2, 3]])
        .build()
        .unwrap()
});

static K_TUPLES_2: Lazy<KTuples> = Lazy::new(|| {
    KTuplesBuilder::new()
        .k(5)
        .max_terminal_index(3)
        .terminal_indices(&[&[2, 1]])
        .build()
        .unwrap()
});

fn k_tuples_k_concat() {
    let _result = K_TUPLES_1.clone().k_concat(&K_TUPLES_2, 10);
}

fn k_tuples_k_concat_benchmark(c: &mut Criterion) {
    c.bench_function("k_tuples_k_concat", |b| b.iter(k_tuples_k_concat));
}

fn k_tuples_k_concat_large() {
    let k_tuples_1 = KTuplesBuilder::new()
        .k(3)
        .max_terminal_index(5)
        .terminal_indices(&[&[1, 2, 3, 4, 5]])
        .build()
        .unwrap();
    let k_tuples_2 = KTuplesBuilder::new()
        .k(3)
        .max_terminal_index(5)
        .terminal_indices(&[&[5, 4, 3, 2, 1]])
        .build()
        .unwrap();
    let _result = k_tuples_1.k_concat(&k_tuples_2, 20);
}

fn k_tuples_k_concat_benchmark_large(c: &mut Criterion) {
    c.bench_function("k_tuples_k_concat_large", |b| {
        b.iter(k_tuples_k_concat_large)
    });
}

criterion_group! {
    name = benches_1;
    config = Criterion::default().sample_size(500);
    targets = k_tuples_k_concat_benchmark
}
criterion_group! {
    name = benches_large;
    config = Criterion::default().sample_size(500);
    targets = k_tuples_k_concat_benchmark_large
}

criterion_main!(benches_1, benches_large);
