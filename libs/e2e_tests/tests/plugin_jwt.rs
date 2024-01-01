pub mod jwt {
  use conductor_common::{
    http::{ConductorHttpRequest, Method, StatusCode, ToHeadersMap},
    plugin::CreatablePlugin,
    serde_utils::LocalFileReference,
  };

  use e2e::suite::TestSuite;
  use httpmock::Method::POST;
  use jwt_auth_plugin::*;
  use serde_json::json;
  use tokio::test;

  static JWKS_RSA512_PRIVATE_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
  MIIEowIBAAKCAQEAjllePzkDl9e0O9Vuy1/qpSPUL8RQbuHOCQknWysfHlm6QGNq
  iyDY46AMfpaSb45bMYQjgOoL7nboe8Q1Qaz4M33PyV+/cYm9lY2cdxE72Vd7LlLF
  I+4q5uPbnX0ofb1kiD47I7KOKshbq2UzLnV6CDXBr5+LMZyXKOCNCEtvEytHwdXD
  osveJ0BzBaEY6tdJdmitGaXrqHj365Rms14x8uU6uSXZm3ZQYB/j5oiGu+JoGIIP
  GPyEQ4R0lThMCxXmmplcVUkfFrsth3WhQdzlRwXMPT1myvEA8Cro4nWTPKnL/W4e
  1r4CT1NjMDYPbDU0zJhBei8FCeCAW8DSjd7fawIDAQABAoIBACk8eFHmSUUuZnbC
  0HK32Xh3VZt0yjwky5PQhAckCcK4CX1nj1C4djwSfCwboFYSrhY9Ci/pHQW6ioR4
  BVl+KvR3qL7ULthMJ5BwUngnlOfUMMntjlBnSSRTs6X+wMEUIVBafrVLn2WDXxLa
  oSX/QBeqwu4GUMNRcnSUACb7+zRY7d7gn4xSHL4lbkQRZ+ZuWabjrmj1pDM7rGrZ
  3qMfLVsTvBaqYMlX01clZtd8IcZxA04b1eTSwok1ut6kIvLc/+DXIM0+J7GdCU5h
  DCMPk3hTIwjtj3U0ad4dGP2WiAYbR/L8Hozjvr50NgiSTB0ZEka5PfAmpu86QIq/
  +GzDN7kCgYEA+GP8GrBL9R50yADVLZ/idrBnLmPnFLAYU469jUYnIUqbNRUP1yia
  V0mnxrJS47H+uHClavIyapz6s8pBE8AHWgqjj41kihB3hmRk49/nsIbPG4fewXPw
  KLxConzoqsOdZhHHF0UaJTgq9FMpi2okoLC3BfD1j2X5OVAn/wNeq08CgYEAkrW/
  d7d7urLe/79ew46Ca9E/TZdkPIJIkFFfqxFO8+6tFtP9UEwmp0rOK9YCIv9Hs+24
  6F+TnmCQd5u+VcUarMrD5jUQB4zNEqDiBenUbpYiZDl3uLTelHNMMUVeqX4PfDvG
  gh1HosErQhkysayVyQK87/N5F0dN1DZ07b6i0yUCgYA/wMHzQ66rQl7s+rG8nR3u
  IsbI9GFaQPxtbeSe/xOKCvEdRcOkEMrUfpYufJSj1oqvYlJCydlA3fvG67GaVR5N
  8Q8cCEl22lUjTF9M0apQ97juswfslUpd2jwsIm1BbyXWDdgQ0+6rAOidfz7ZhqvS
  BqljP/53CNBX8ofhf0bsJwKBgBerJKmeu2JiayGdcR9hhV75khnle7FbX3OQ/Tsu
  /qrR7bDKIIrsziudIOfnjc6xmpLHnlY23Szm7Ueuo6VYuDX6PGKOWvis2YTQ2cYU
  dEYnCINc1hjBbUtL0pX8WApGIR9s0Vi6eo0iVuVCBXCupDearnqTsAx2X3MGGhUk
  9UXVAoGBAMDzIS2XjvzO1sIDbjbb4mIa6iQU5s/E9hV0H4sHq+yb8EWMBajwV1tZ
  TQYHV7TjRUSrEkmcinVIXi/oQCGz9og/MHGGBD0Idoww5PqjB9jTcCIoAd8PTZCp
  I3OrgFkoqk03cpX4AL2GYC2ejytAqboL6pFTfmTgg2UtvKIeaTyF
  -----END RSA PRIVATE KEY-----
  "#;

  static JWKS_RSA512_2045_PUBLIC_KEY: &str = r#"{
            "keys": [
              {
                "kty": "RSA",
                "e": "AQAB",
                "use": "sig",
                "kid": "test_id",
                "alg": "RS512",
                "n": "jllePzkDl9e0O9Vuy1_qpSPUL8RQbuHOCQknWysfHlm6QGNqiyDY46AMfpaSb45bMYQjgOoL7nboe8Q1Qaz4M33PyV-_cYm9lY2cdxE72Vd7LlLFI-4q5uPbnX0ofb1kiD47I7KOKshbq2UzLnV6CDXBr5-LMZyXKOCNCEtvEytHwdXDosveJ0BzBaEY6tdJdmitGaXrqHj365Rms14x8uU6uSXZm3ZQYB_j5oiGu-JoGIIPGPyEQ4R0lThMCxXmmplcVUkfFrsth3WhQdzlRwXMPT1myvEA8Cro4nWTPKnL_W4e1r4CT1NjMDYPbDU0zJhBei8FCeCAW8DSjd7faw"
            }
          ]
        }"#;

  #[test]
  async fn valid_token_flow() {
    let test = TestSuite {
      plugins: vec![jwt_auth_plugin::Plugin::create(jwt_auth_plugin::Config {
        jwks_providers: vec![jwt_auth_plugin::JwksProvider::Local {
          file: LocalFileReference {
            path: String::from("jwks.json"),
            contents: JWKS_RSA512_2045_PUBLIC_KEY.to_string(),
          },
        }],
        allowed_algorithms: None,
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: Some("X-Forwarded-Claims".to_string()),
        forward_token_to_upstream_header: Some("X-Forwarded-Token".to_string()),
        lookup_locations: vec![jwt_auth_plugin::LookupLocation::Header {
          name: "Authorization".to_string(),
          prefix: Some("Bearer".to_string()),
        }],
        reject_unauthenticated_requests: Some(true),
      })
      .await
      .unwrap()],
      ..Default::default()
    };
    let token = encode::<ClaimsJsonObject>(
      &JwtHeader {
        alg: Algorithm::RS512,
        ..Default::default()
      },
      &json!({
        "my_claim": "test",
        "exp": 1924942936
      }),
      &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
    )
    .unwrap();

    let formatted_token = format!("Bearer {}", token);
    let response = test
      .run_with_mock(
        ConductorHttpRequest {
          method: Method::POST,
          uri: "/graphql".to_string(),
          headers: vec![("Authorization", formatted_token.as_str())].to_headers_map(),
          ..Default::default()
        },
        |when, then| {
          when
            .method(POST)
            .path("/graphql")
            .header(
              "x-forwarded-claims",
              "{\"my_claim\":\"test\",\"exp\":1924942936}",
            )
            .header("x-forwarded-token", token);
          then
            .status(200)
            .header("content-type", "application/json")
            .body(
              json!({
                  "data": {
                      "__typename": "Query"
                  },
                  "errors": null
              })
              .to_string(),
            );
        },
      )
      .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
  }

  #[test]
  async fn invalid_token_not_rejected() {
    let test = TestSuite {
      plugins: vec![jwt_auth_plugin::Plugin::create(jwt_auth_plugin::Config {
        jwks_providers: vec![jwt_auth_plugin::JwksProvider::Local {
          file: LocalFileReference {
            path: String::from("jwks.json"),
            contents: JWKS_RSA512_2045_PUBLIC_KEY.to_string(),
          },
        }],
        allowed_algorithms: None,
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: Some("X-Forwarded-Claims".to_string()),
        forward_token_to_upstream_header: Some("X-Forwarded-Token".to_string()),
        lookup_locations: vec![jwt_auth_plugin::LookupLocation::Header {
          name: "Authorization".to_string(),
          prefix: Some("Bearer".to_string()),
        }],
        reject_unauthenticated_requests: Some(false),
      })
      .await
      .unwrap()],
      ..Default::default()
    };

    let response = test
      .run_with_mock(
        ConductorHttpRequest {
          method: Method::POST,
          uri: "/graphql".to_string(),
          ..Default::default()
        },
        |when, then| {
          when.method(POST).path("/graphql");
          then
            .status(200)
            .header("content-type", "application/json")
            .body(
              json!({
                  "data": {
                      "__typename": "Query"
                  },
                  "errors": null
              })
              .to_string(),
            );
        },
      )
      .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
  }

  #[test]
  async fn token_missing_rejection() {
    let test = TestSuite {
      plugins: vec![jwt_auth_plugin::Plugin::create(jwt_auth_plugin::Config {
        jwks_providers: vec![jwt_auth_plugin::JwksProvider::Local {
          file: LocalFileReference {
            path: String::from("jwks.json"),
            contents: JWKS_RSA512_2045_PUBLIC_KEY.to_string(),
          },
        }],
        allowed_algorithms: None,
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: Some("X-Forwarded-Claims".to_string()),
        forward_token_to_upstream_header: Some("X-Forwarded-Token".to_string()),
        lookup_locations: vec![jwt_auth_plugin::LookupLocation::Header {
          name: "Authorization".to_string(),
          prefix: Some("Bearer".to_string()),
        }],
        reject_unauthenticated_requests: Some(true),
      })
      .await
      .unwrap()],
      ..Default::default()
    };

    // token is missing
    let response = test
      .run_http_request(ConductorHttpRequest {
        method: Method::POST,
        uri: "/graphql".to_string(),
        ..Default::default()
      })
      .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(
      response.body,
      "{\"errors\":[{\"message\":\"unauthenticated request\"}]}"
    );
  }

  #[test]
  async fn token_invalid_rejection() {
    let test = TestSuite {
      plugins: vec![jwt_auth_plugin::Plugin::create(jwt_auth_plugin::Config {
        jwks_providers: vec![jwt_auth_plugin::JwksProvider::Local {
          file: LocalFileReference {
            path: String::from("jwks.json"),
            contents: JWKS_RSA512_2045_PUBLIC_KEY.to_string(),
          },
        }],
        allowed_algorithms: None,
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: Some("X-Forwarded-Claims".to_string()),
        forward_token_to_upstream_header: Some("X-Forwarded-Token".to_string()),
        lookup_locations: vec![jwt_auth_plugin::LookupLocation::Header {
          name: "Authorization".to_string(),
          prefix: Some("Bearer".to_string()),
        }],
        reject_unauthenticated_requests: Some(true),
      })
      .await
      .unwrap()],
      ..Default::default()
    };

    // token is invalid
    let response = test
      .run_http_request(ConductorHttpRequest {
        method: Method::POST,
        uri: "/graphql".to_string(),
        headers: vec![("Authorization", "Bearer XYZ")].to_headers_map(),
        ..Default::default()
      })
      .await;

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(
      response.body,
      "{\"errors\":[{\"message\":\"unauthenticated request\"}]}"
    );
  }
}
