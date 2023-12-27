use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use conductor::run_services;
use conductor_engine::gateway::ConductorGateway;
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

static GW_ONCE: Once = Once::new();
static GW_MUTEX: Mutex<Option<thread::JoinHandle<Result<(), Error>>>> = Mutex::new(None);

fn start_gateway() {
  GW_ONCE.call_once(|| {
    let handle = thread::spawn(|| {
      let rt = Runtime::new().unwrap();
      rt.block_on(run_services(&String::from("./config.yaml")))
    });
    let mut gw = GW_MUTEX.lock().expect("Failed to lock GW_MUTEX");
    *gw = Some(handle);
  });
}
static SERVER_ONCE: Once = Once::new();
static SERVER_MUTEX: Mutex<Option<thread::JoinHandle<Result<(), Error>>>> = Mutex::new(None);

#[post("/graphql")]
async fn hello() -> impl Responder {
  HttpResponse::Ok().body(
    json!({
      "data": {
        "country": {
          "name": "Egypt"
        }
      }
    })
    .to_string(),
  )
}

async fn start_mocked_server() -> std::io::Result<()> {
  HttpServer::new(|| App::new().service(hello))
    .bind(("127.0.0.1", 4444))?
    .run()
    .await
}

fn start_server() {
  SERVER_ONCE.call_once(|| {
    let handle = thread::spawn(|| {
      let rt = Runtime::new().unwrap();
      rt.block_on(start_mocked_server())
    });
    let mut server = SERVER_MUTEX.lock().expect("Failed to lock SERVER_MUTEX");
    *server = Some(handle);
  });
}

const SAMPLE_SIZE: usize = 100;
const CONCURRENCY_LEVEL: usize = 10;

fn criterion_benchmark(c: &mut Criterion) {
  start_server();
  start_gateway();
  thread::sleep(std::time::Duration::from_secs(5));

  let rt = Runtime::new().unwrap();

  c.bench_function("request hot path without HTTP server", |b| {
    b.iter(|| {
      // let gateway = ConductorGateway::
    })
  });

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
