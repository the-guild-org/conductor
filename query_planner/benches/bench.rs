use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use query_planner::user_query::parse_user_query;

const SAMPLE_SIZE: usize = 100;

fn criterion_benchmark(c: &mut Criterion) {
    let query = fs::read_to_string("./benches/huge-query.graphql").unwrap();

    c.bench_function("Parsing User Query", |b| {
        b.iter(|| {
            let user_query = parse_user_query(&query);

            black_box(user_query)
        })
    });
}

fn configure_benchmark() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}

criterion_group! {
    name = benches;
    config = configure_benchmark();
    targets = criterion_benchmark
}
criterion_main!(benches);
