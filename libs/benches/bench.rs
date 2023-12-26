use conductor::run_services;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::future::join_all;
use hyper::Client;
use hyper::{Body, Request};
use serde_json::json;
use std::io::Error;
use std::sync::{Mutex, Once};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

static START: Once = Once::new();
static SERVER: Mutex<Option<thread::JoinHandle<Result<(), Error>>>> = Mutex::new(None);

fn start_server() {
  START.call_once(|| {
    let handle = thread::spawn(|| {
      let rt = Runtime::new().unwrap();
      rt.block_on(run_services(&String::from("./config.yaml")))
    });
    let mut server = SERVER.lock().expect("Failed to lock SERVER");
    *server = Some(handle);
  });
}

const SAMPLE_SIZE: usize = 100;
const CONCURRENCY_LEVEL: usize = 10;

fn criterion_benchmark(c: &mut Criterion) {
  start_server();
  thread::sleep(std::time::Duration::from_secs(5));

  let rt = Runtime::new().unwrap();

  c.bench_function("Single Request /graphql", |b| {
    b.iter(|| {
      let client = Client::new();
      let body = json!({
          "query": "query GetCountryCode($code: ID!) { country(code: $code) { name } }",
          "variables": { "code": "EG" },
          "operationName": "GetCountryCode"
      });
      let serialized_body = body.to_string();

      let request = Request::builder()
        .method("POST")
        .uri("http://localhost:8000/graphql")
        .header("content-type", "application/json")
        .body(Body::from(serialized_body))
        .unwrap();

      let response = rt.block_on(client.request(request));
      black_box(response)
    })
  });

  c.bench_function(
    &format!("{} Concurrent Requests /graphql", CONCURRENCY_LEVEL),
    |b| {
      b.iter(|| {
        let futures: Vec<_> = (0..CONCURRENCY_LEVEL)
          .map(|_| {
            let client = Client::new();
            let body = json!({
                "query": "query GetCountryCode($code: ID!) { country(code: $code) { name } }",
                "variables": { "code": "EG" },
                "operationName": "GetCountryCode"
            });
            let serialized_body = body.to_string();

            let request = Request::builder()
              .method("POST")
              .uri("http://localhost:8000/graphql")
              .header("content-type", "application/json")
              .body(Body::from(serialized_body))
              .unwrap();

            client.request(request)
          })
          .collect();

        let responses = rt.block_on(join_all(futures));
        black_box(responses)
      })
    },
  );
}

fn configure_benchmark() -> Criterion {
  Criterion::default()
    .sample_size(SAMPLE_SIZE)
    .measurement_time(Duration::from_secs(60))
}

criterion_group! {
    name = benches;
    config = configure_benchmark();
    targets = criterion_benchmark
}
criterion_main!(benches);
