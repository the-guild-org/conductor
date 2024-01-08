pub use conductor_common::http::Bytes;
pub use http::{Request, Response};
use opentelemetry_http::{HttpClient, HttpError};

#[derive(Debug)]
pub struct WasmHttpClient {
  inner: reqwest::Client,
}

impl WasmHttpClient {
  pub fn new() -> Self {
    Self {
      inner: wasm_polyfills::create_http_client().build().unwrap(),
    }
  }
}

#[async_trait::async_trait]
impl HttpClient for WasmHttpClient {
  async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
    Box::pin(wasm_polyfills::call_async(async move {
      let request = request.try_into()?;
      let maybe_response = self.inner.execute(request).await;

      match maybe_response {
        Ok(mut response) => {
          let headers = std::mem::take(response.headers_mut());
          let mut http_response = Response::builder()
            .status(response.status())
            .body(response.bytes().await?)?;
          *http_response.headers_mut() = headers;

          Ok(http_response)
        }
        Err(e) => Err(HttpError::from(e)),
      }
    }))
    .await
  }
}
