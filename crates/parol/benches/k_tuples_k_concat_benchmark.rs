use criterion::{criterion_group, criterion_main, Criterion};
use parol::{KTuple, KTuples};
use parol_runtime::once_cell::sync::Lazy;

static K_TUPLES_1: Lazy<KTuples> =
    Lazy::new(|| KTuples::of(&[KTuple::new(2).with_terminal_indices(&[1, 2, 3])], 5));

static K_TUPLES_2: Lazy<KTuples> =
    Lazy::new(|| KTuples::of(&[KTuple::new(2).with_terminal_indices(&[2, 1])], 5));

fn k_tuples_k_concat() {
    let _result = K_TUPLES_1.clone().k_concat(&K_TUPLES_2, 10);
}

fn k_tuples_k_concat_benchmark(c: &mut Criterion) {
    c.bench_function("k_tuples_k_concat", |b| b.iter(k_tuples_k_concat));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(500);
    targets = k_tuples_k_concat_benchmark
}
criterion_main!(benches);
