use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use federation_query_planner::{
  query_planner::plan_for_user_query, supergraph::parse_supergraph, user_query::parse_user_query,
};

const SAMPLE_SIZE: usize = 100;

fn criterion_benchmark(c: &mut Criterion) {
  let supergraph = fs::read_to_string("./benches/fixtures/complex-supergraph.graphql").unwrap();
  let query = fs::read_to_string("./benches/fixtures/huge-query.graphql").unwrap();

  // c.bench_function("Parsing User Query", |b| {
  //     b.iter(|| {
  // let user_query = parse_user_query(&query);

  //         black_box(user_query)
  //     })
  // });
  // c.bench_function("Parsing Supergraph Schema", |b| {
  //     b.iter(|| {
  // let parsed_supergraph = parse_supergraph(&supergraph);

  //         black_box(parsed_supergraph)
  //     })
  // });

  let parsed_supergraph = parse_supergraph(&supergraph).unwrap();
  let mut user_query = parse_user_query(&query);

  c.bench_function("Construct Query Plan", |b| {
    b.iter(|| {
      let plan = plan_for_user_query(&parsed_supergraph, &mut user_query);

      black_box(plan)
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
