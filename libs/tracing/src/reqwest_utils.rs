use reqwest_middleware::ClientBuilder;
use reqwest_middleware::ClientWithMiddleware;

pub type TracedHttpClient = ClientWithMiddleware;

pub fn traced_reqwest(raw_client: reqwest::Client) -> TracedHttpClient {
  ClientBuilder::new(raw_client)
    .with(minitrace_reqwest::MinitraceReqwestMiddleware::default())
    .build()
}
