use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use parol::analysis::{FirstCache, FollowCache};

fn benchmark_real_grammar_scenarios(c: &mut Criterion) {
    // Benchmark with realistic grammar scenarios
    let large_grammar = create_complex_grammar();

    c.bench_function("follow_k_large_grammar", |b| {
        b.iter(|| {
            let first_cache = FirstCache::new();
            let follow_cache = FollowCache::new();
            parol::analysis::follow_k(&large_grammar, 3, &first_cache, &follow_cache)
        })
    });
}

fn create_complex_grammar() -> parol::GrammarConfig {
    // Download the reference grammar file from here
    // https://raw.githubusercontent.com/CogniPilot/rumoca/e83e5d3/modelica.par
    // The transformed grammar has following properties:
    // * LL(3)
    // * 99 terminals
    // * 305 non-terminals
    // * 504 productions
    // * 505 equations in equation system for FIRST(k)
    // * 542 equations in equation system for FOLLOW(k)
    // The calculation of FIRST(k) and FOLLOW(k) sets takes usually about 5 seconds.

    let grammar_content = {
        let url = "https://raw.githubusercontent.com/CogniPilot/rumoca/e83e5d3/modelica.par";
        let response = reqwest::blocking::get(url).expect("Failed to download grammar file");
        response
            .text()
            .expect("Failed to read grammar file content")
    };
    parol::obtain_grammar_config_from_string(&grammar_content, false)
        .expect("Failed to parse grammar")
}

criterion_group! {
    name = analysis_benches;
    config = Criterion::default().
        warm_up_time(Duration::from_secs(10)).
        sample_size(10).
        measurement_time(Duration::from_secs(60));
    targets = benchmark_real_grammar_scenarios
}
criterion_main!(analysis_benches);
