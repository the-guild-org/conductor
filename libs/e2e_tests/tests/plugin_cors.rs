use conductor_common::http::{
    header::{
        ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS,
        CONTENT_LENGTH, ORIGIN, VARY,
    },
    ConductorHttpRequest, HttpHeadersMap, Method, StatusCode,
};
use e2e::suite::TestSuite;
use tokio::test;

#[test]
async fn options_zero_content_length() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(Default::default()))],
        ..Default::default()
    };
    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(CONTENT_LENGTH),
        Some(&"0".parse().unwrap())
    );
}

#[test]
async fn default_methods() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(Default::default()))],
        ..Default::default()
    };

    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
        Some(&"*".parse().unwrap())
    );
}

#[test]
async fn override_methods() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(cors_plugin::Config {
            allowed_methods: Some("GET, POST".into()),
            ..Default::default()
        }))],
        ..Default::default()
    };

    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
        Some(&"GET, POST".parse().unwrap())
    );
}

#[test]
async fn post_default_options_allow_all_origins() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(Default::default()))],
        ..Default::default()
    };

    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::POST,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&"*".parse().unwrap())
    );
}

#[test]
async fn options_default_options_allow_all_origins() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(Default::default()))],
        ..Default::default()
    };

    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&"*".parse().unwrap())
    );
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
        Some(&"*".parse().unwrap())
    );
}

#[test]
async fn wildcard_config_reflects_origin() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(cors_plugin::Config {
            allowed_origin: Some("*".to_string()),
            ..Default::default()
        }))],
        ..Default::default()
    };

    let mut req_headers = HttpHeadersMap::new();
    req_headers.insert(
        ACCESS_CONTROL_REQUEST_HEADERS,
        "x-header-1, x-header-2".parse().unwrap(),
    );
    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&"*".parse().unwrap())
    );
    assert_eq!(response.headers.get(VARY), Some(&"Origin".parse().unwrap()));
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
        Some(&"*".parse().unwrap())
    );
}

#[test]
async fn override_origin() {
    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(cors_plugin::Config {
            allowed_origin: Some("http://my-server.com".to_string()),
            ..Default::default()
        }))],
        ..Default::default()
    };

    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&"http://my-server.com".parse().unwrap())
    );
    assert_eq!(response.headers.get(VARY), Some(&"Origin".parse().unwrap()));
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
        Some(&"*".parse().unwrap())
    );
}

#[test]
async fn reflects_origin() {
    let mut req_headers = HttpHeadersMap::new();
    req_headers.insert(ORIGIN, "http://my-server.com".parse().unwrap());

    let test = TestSuite {
        plugins: vec![Box::new(cors_plugin::Plugin(cors_plugin::Config {
            allowed_origin: Some("reflect".to_string()),
            ..Default::default()
        }))],
        ..Default::default()
    };

    let response = test
        .run_http_request(ConductorHttpRequest {
            method: Method::OPTIONS,
            uri: "/graphql".to_string(),
            headers: req_headers,
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&"http://my-server.com".parse().unwrap())
    );
    assert_eq!(response.headers.get(VARY), Some(&"Origin".parse().unwrap()));
    assert_eq!(
        response.headers.get(ACCESS_CONTROL_ALLOW_METHODS),
        Some(&"*".parse().unwrap())
    );
}
