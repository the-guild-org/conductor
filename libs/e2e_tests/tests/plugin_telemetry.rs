pub mod telemetry {
  use conductor_common::{
    http::{ConductorHttpRequest, Method},
    plugin::CreatablePlugin,
  };
  use e2e::suite::TestSuite;
  use predicates::{constant::always, ord::eq};
  use tokio::test;
  use tracing::Level;
  use tracing_capture::{predicates::*, CaptureLayer, SharedStorage};
  use tracing_subscriber::layer::SubscriberExt;

  #[test]
  async fn spans() {
    let plugin = opentelemetry_plugin::Plugin::create(opentelemetry_plugin::Config {
      targets: vec![opentelemetry_plugin::Target::Stdout {
        level: opentelemetry_plugin::OpenTelemetryTracesLevel::Info,
      }],
      ..Default::default()
    })
    .await
    .unwrap();
    let (mut tracing_manager, root_layer) = opentelemetry_plugin::TracingManager::new(
      &opentelemetry_plugin::LoggerConfigFormat::Json,
      "info",
      false,
    )
    .unwrap();
    plugin
      .configure_tracing("/", &mut tracing_manager)
      .expect("failed to create tracing layer");

    let test = TestSuite {
      plugins: vec![plugin],
      ..Default::default()
    };

    let storage = SharedStorage::default();
    let subscriber = tracing_subscriber::registry()
      .with(root_layer)
      .with(CaptureLayer::new(&storage));

    {
      let _guard = tracing::subscriber::set_default(subscriber);

      test
        .run_http_request(ConductorHttpRequest {
          method: Method::POST,
          uri: "/graphql".to_string(),
          ..Default::default()
        })
        .await;

      tracing_manager.shutdown().await;
    }

    let storage = storage.lock();
    let spans = storage.scan_spans();

    let predicate = level(Level::DEBUG) & name(eq("gateway_flow"));
    spans.single(&predicate);

    let predicate = level(Level::DEBUG) & name(eq("on_downstream_http_request"));
    spans.single(&predicate);

    let predicate = level(Level::INFO) & name(eq("graphql_parse"));
    spans.single(&predicate);

    let predicate = level(Level::INFO) & name(eq("graphql_execute"));
    spans.single(&predicate);

    let predicate = level(Level::DEBUG) & name(eq("on_downstream_graphql_request"));
    spans.single(&predicate);

    let predicate = level(Level::INFO) & name(eq("upstream_http"));
    spans.single(&predicate);

    let predicate = level(Level::DEBUG) & name(eq("on_upstream_http_request"));
    spans.single(&predicate);

    let predicate = level(Level::DEBUG) & name(eq("on_upstream_graphql_request"));
    spans.single(&predicate);

    let predicate = level(Level::DEBUG) & name(eq("on_downstream_http_response"));
    spans.single(&predicate);

    let predicate = level(Level::INFO)
      & name(eq("HTTP request"))
      & field("http.method", [always()])
      & field("http.scheme", [always()])
      & field("http.host", [always()])
      & field("net.host.port", [always()])
      & field("otel.kind", [always()])
      & field("otel.name", [always()])
      & field("http.status_code", [always()])
      & field("http.user_agent", [always()]);
    spans.single(&predicate);
  }
}
