we can't upgrade from reqwest `0.11.27` to `0.12.8` because:

```sh
124 |           .with_http_client(reqwest::Client::new())
    |            ---------------- ^^^^^^^^^^^^^^^^^^^^^^ the trait opentelemetry_http::HttpClient is not implemented for reqwest::Client
    |            |
    |            required by a bound introduced by this call
```