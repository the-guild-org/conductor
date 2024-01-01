use core::future::Future;

#[cfg(target_arch = "wasm32")]
pub fn call_async<T>(future: impl Future<Output = T>) -> impl Future<Output = T> + Send {
  send_wrapper::SendWrapper::new(future)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call_async<F>(future: F) -> F
where
  F: Future,
{
  future
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_http_client() -> reqwest::ClientBuilder {
  use std::time::Duration;

  reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(10))
    .tcp_keepalive(Duration::from_secs(120))
}

#[cfg(target_arch = "wasm32")]
pub fn create_http_client() -> reqwest::ClientBuilder {
  reqwest::Client::builder()
}
