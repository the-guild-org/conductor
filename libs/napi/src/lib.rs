#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use conductor::run_services;
#[actix_web::main]
#[napi]
pub async fn execute_conductor(config_file_path: String) -> Result<(), napi::Error> {
  run_services(&config_file_path)
    .await
    .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))
}

#[napi]
pub fn shutdown_server() {
  panic!("Exited process!")
}
