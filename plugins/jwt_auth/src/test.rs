#[cfg(test)]
pub mod jwt_plugin {
  use crate::{config::JwtAuthPluginLookupLocation, plugin::LookupError};
  use conductor_common::http::{ConductorHttpRequest, ToHeadersMap};
  use jsonwebtoken::jwk::JwkSet;

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

  // Generated using https://mkjwk.org
  lazy_static::lazy_static! {
    static ref JWKS_RSA512_2045_PUBLIC_KEY: JwkSet = {
      serde_json::from_str(
          r#"{
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
        }"#).unwrap()
    };

    static ref JWKS_PS512_2045_PUBLIC_KEY: JwkSet = {
      serde_json::from_str(
          r#"{
            "keys": [
              {
                "kty": "RSA",
                "e": "AQAB",
                "use": "sig",
                "kid": "test_id_other",
                "alg": "PS512",
                "n": "6r4D9ggiekrnIBwjNM244TnHmR_mv1hXK1psJryzinG-DLhQ4fjKJ456c5ixeryZTiVLKNYxgzZeclBuqrMCLnP7ZEJ1tENHtcDs5o1On0BYNsfuDiUeASt9knD0SXxJ8mqzPKIwvW8mco4HthA6iEzn2vvzIT6UC5fwuQorXdnQXKZ0Ui9-gvp0v4M_Yi_q-tgC9OZSl82Bt6EGtM1FHJLDIHSyQzv9U1_rwdBjXDtla-TLfmSgfnEYQLG-FtKqYDJq6CK7ghSxo4DZswClsEw6_36RDv0wXnRm13cBc3SpdUf1YCTf6Xy2PTRlag_z2Y0Gl9g4ZyXCl-rGXfBRGw"
              }
            ]
        }"#).unwrap()
    };
  }

  pub mod lookup {
    use super::*;

    fn plugin_test(config: Vec<JwtAuthPluginLookupLocation>) -> crate::Plugin {
      crate::Plugin::new_from_config(crate::Config {
        jwks_providers: vec![],
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: config,
        allowed_algorithms: None,
      })
    }

    #[test]
    fn jwt_token_lookup_header() {
      // header doesn't exists
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: None,
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Err(LookupError::LookupFailed)
      );

      // header exists but empty
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: None,
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Authorization", "")].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Ok(String::from(""))
      );

      // Simple token without prefix
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: None,
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Authorization", "Test")].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Ok(String::from("Test"))
      );

      // Token with prefix, but prefix is not configured
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: None,
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Authorization", "Bearer XYZ")]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        }),
        Ok(String::from("Bearer XYZ"))
      );

      // Prefix is configured, but also with space
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer ")),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Authorization", "Bearer XYZ")]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        }),
        Ok(String::from("XYZ"))
      );

      // Prefix is configured, but as clean string (test trimming)
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Authorization", "Bearer XYZ")]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        }),
        Ok(String::from("XYZ"))
      );

      // Prefix doesn't match
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Authorization", "XYZ")].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Err(LookupError::MismatchedPrefix)
      );
    }

    #[test]
    fn jwt_token_lookup_query_param() {
      // query param doesn't exists
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::QueryParam {
          name: String::from("jwt"),
        }])
        .lookup(&ConductorHttpRequest {
          ..Default::default()
        }),
        Err(LookupError::LookupFailed)
      );

      // query param exists, but incorrect case
      // see https://stackoverflow.com/questions/24699643/are-query-string-keys-case-sensitive
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::QueryParam {
          name: String::from("JWT"),
        }])
        .lookup(&ConductorHttpRequest {
          ..Default::default()
        }),
        Err(LookupError::LookupFailed)
      );

      // query param exists and has value
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::QueryParam {
          name: String::from("jwt"),
        }])
        .lookup(&ConductorHttpRequest {
          query_string: String::from("jwt=XYZ"),
          ..Default::default()
        }),
        Ok(String::from("XYZ"))
      );
    }

    #[test]
    fn jwt_token_lookup_cookie() {
      // cookie doesn't exists
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Cookie {
          name: String::from("auth"),
        }])
        .lookup(&ConductorHttpRequest {
          ..Default::default()
        }),
        Err(LookupError::LookupFailed)
      );

      // cookie valid and found
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Cookie {
          name: String::from("auth"),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Cookie", "auth=XYZ")].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Ok(String::from("XYZ"))
      );

      // cookie with multiple keys
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Cookie {
          name: String::from("auth"),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Cookie", "test=1; v=2; auth=XYZ; t=3;")]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        }),
        Ok(String::from("XYZ"))
      );

      // empty cookie with multiple keys
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Cookie {
          name: String::from("auth"),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Cookie", "")].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Err(LookupError::LookupFailed)
      );

      // invalid cookie
      assert_eq!(
        plugin_test(vec![JwtAuthPluginLookupLocation::Cookie {
          name: String::from("auth"),
        }])
        .lookup(&ConductorHttpRequest {
          headers: vec![("Cookie", ";;;;;;")].to_headers_map().unwrap(),
          ..Default::default()
        }),
        Err(LookupError::LookupFailed)
      );
    }
  }

  pub mod flow {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde_json::{json, Value};

    use crate::plugin::JwtError;

    use super::*;

    fn plugin_test(config: crate::Config) -> crate::Plugin {
      crate::Plugin::new_from_config(config)
    }

    #[test]
    fn failed_lookup() {
      let p = plugin_test(crate::Config {
        jwks_providers: vec![],
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![crate::config::JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer ")),
        }],
        allowed_algorithms: None,
      });

      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          ..Default::default()
        },
      );

      assert!(result.is_err_and(|e| e == JwtError::LookupFailed(LookupError::LookupFailed)));
    }

    #[test]
    fn invalid_token_header() {
      let p = plugin_test(crate::Config {
        jwks_providers: vec![],
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![crate::config::JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }],
        allowed_algorithms: None,
      });

      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", "Bearer ABC")]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_err_and(
        |e| e == JwtError::InvalidJwtHeader(jsonwebtoken::errors::ErrorKind::InvalidToken.into())
      ));
    }

    #[test]
    fn no_matching_jwks() {
      let p = plugin_test(crate::Config {
        jwks_providers: vec![],
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![crate::config::JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }],
        allowed_algorithms: None,
      });

      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "test": "test",
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_PS512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_err_and(|e| e == JwtError::FailedToLocateProvider));
    }

    #[test]
    fn valid_token_and_claims_extraction() {
      let p = plugin_test(crate::Config {
        jwks_providers: vec![],
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![crate::config::JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }],
        allowed_algorithms: None,
      });

      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "test": "test",
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_ok());
      assert!(result
        .unwrap()
        .0
        .claims
        .get("test")
        .is_some_and(|v| v == "test"));
    }

    #[test]
    fn issuers_validation() {
      let p = plugin_test(crate::Config {
        jwks_providers: vec![],
        audiences: None,
        issuers: Some(vec![
          String::from("https://test.com"),
          String::from("https://test2.com"),
        ]),
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![crate::config::JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }],
        allowed_algorithms: None,
      });

      // iss is valid
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "test": "test",
          "iss": "https://test.com",
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_ok());

      // iss not set
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "test": "test",
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_err_and(|e| e
        == JwtError::AllProvidersFailedToDecode(vec!(JwtError::FailedToDecodeToken(
          jsonwebtoken::errors::ErrorKind::InvalidIssuer.into()
        )))));

      // iss mismatch
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "test": "test",
          "iss": "http://other.com",
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_err_and(|e| e
        == JwtError::AllProvidersFailedToDecode(vec!(JwtError::FailedToDecodeToken(
          jsonwebtoken::errors::ErrorKind::InvalidIssuer.into()
        )))));
    }

    #[test]
    fn audiences_validation() {
      let p = plugin_test(crate::Config {
        jwks_providers: vec![],
        audiences: Some(vec![
          String::from("bookstore_android.apps.googleusercontent.com"),
          String::from("bookstore_web.apps.googleusercontent.com"),
        ]),
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![crate::config::JwtAuthPluginLookupLocation::Header {
          name: String::from("Authorization"),
          prefix: Some(String::from("Bearer")),
        }],
        allowed_algorithms: None,
      });

      // aud is valid, matches only one
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "aud": [
            "bookstore_android.apps.googleusercontent.com"
          ],
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_ok());

      // aud is valid, matches multiple
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "aud": [
            "bookstore_android.apps.googleusercontent.com",
            "bookstore_web.apps.googleusercontent.com"
          ],
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_ok());

      // aud is not valid, one aud is not matching
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "aud": [
            "bookstore_android.apps.googleusercontent.com",
            "bookstore_web.apps.googleusercontent.com",
            "other"
          ],
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_err_and(|e| e
        == JwtError::AllProvidersFailedToDecode(vec![JwtError::FailedToDecodeToken(
          jsonwebtoken::errors::ErrorKind::InvalidAudience.into()
        )])));

      // aud is empty
      let token = encode::<Value>(
        &Header {
          alg: jsonwebtoken::Algorithm::RS512,
          ..Default::default()
        },
        &json!({
          "aud": [],
          "exp": 1924942936
        }),
        &EncodingKey::from_rsa_pem(JWKS_RSA512_PRIVATE_PEM.as_bytes()).unwrap(),
      )
      .unwrap();

      let formatted_token = format!("Bearer {}", token);
      let result = p.authenticate(
        &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        &ConductorHttpRequest {
          headers: vec![("Authorization", formatted_token.as_str())]
            .to_headers_map()
            .unwrap(),
          ..Default::default()
        },
      );

      assert!(result.is_err_and(|e| e
        == JwtError::AllProvidersFailedToDecode(vec![JwtError::FailedToDecodeToken(
          jsonwebtoken::errors::ErrorKind::InvalidAudience.into()
        )])));
    }
  }

  pub mod jwks_matching {
    use super::*;
    use crate::{config::JwksProviderSourceConfig, plugin::JwtError};

    fn plugin_test(config: Vec<JwksProviderSourceConfig>) -> crate::Plugin {
      crate::Plugin::new_from_config(crate::Config {
        jwks_providers: config,
        audiences: None,
        issuers: None,
        forward_claims_to_upstream_header: None,
        forward_token_to_upstream_header: None,
        reject_unauthenticated_requests: None,
        lookup_locations: vec![],
        allowed_algorithms: None,
      })
    }

    #[test]
    pub fn jwks_matching() {
      // Algorithm matching
      assert!(plugin_test(vec![])
        .find_matching_jwks(
          &jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::RS512,
            ..Default::default()
          },
          &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        )
        .is_ok());

      // Algorithm not matching
      assert_eq!(
        plugin_test(vec![]).find_matching_jwks(
          &jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::ES384,
            ..Default::default()
          },
          &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        ),
        Err(JwtError::FailedToLocateProvider)
      );

      // kid not matching, but algorithm does
      assert!(plugin_test(vec![])
        .find_matching_jwks(
          &jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::RS512,
            kid: Some(String::from("test_id_2")),
            ..Default::default()
          },
          &vec![&JWKS_RSA512_2045_PUBLIC_KEY],
        )
        .is_ok());

      // kid matching
      assert!(plugin_test(vec![])
        .find_matching_jwks(
          &jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::RS512,
            kid: Some(String::from("test_id")),
            ..Default::default()
          },
          &vec![&JWKS_RSA512_2045_PUBLIC_KEY, &JWKS_PS512_2045_PUBLIC_KEY],
        )
        .is_ok_and(|v| v.keys[0].common.key_id.as_ref().unwrap().eq("test_id")));
    }
  }
}
