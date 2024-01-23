pub mod telemetry {
  use conductor_common::{graphql::GraphQLRequest, plugin::CreatablePlugin};
  use conductor_tracing::{minitrace_mgr::MinitraceManager, otel_attrs::*};
  use e2e::suite::TestSuite;
  use minitrace::{
    collector::{Config, SpanContext, SpanId},
    future::FutureExt,
    Span,
  };
  use tokio::test;

  #[test]
  async fn spans() {
    let (spans, reporter) = conductor_tracing::minitrace_mgr::test_utils::TestReporter::new();
    let plugin = telemetry_plugin::Plugin::create(telemetry_plugin::Config {
      targets: vec![telemetry_plugin::Target::Stdout],
      ..Default::default()
    })
    .await
    .unwrap();

    let mut minitrace_mgr = MinitraceManager::default();
    plugin.configure_tracing_for_test(0, Box::new(reporter), &mut minitrace_mgr);
    minitrace::set_reporter(minitrace_mgr.build_reporter(), Config::default());

    let test = TestSuite {
      plugins: vec![plugin],
      ..Default::default()
    };

    let span_context = SpanContext::new(MinitraceManager::generate_trace_id(0), SpanId::default());
    let root_span = Span::root("root", span_context);
    test
      .run_graphql_request(GraphQLRequest::default())
      .in_span(root_span)
      .await;

    minitrace::flush();

    let spans = spans.lock().unwrap();

    assert_eq!(spans.len(), 6);
    // Make sure all spans inherit the same trace id
    assert!(spans.iter().all(|v| v.trace_id == span_context.trace_id));

    let execute = spans
      .iter()
      .find(|v| v.name == "execute")
      .expect("failed to find span");
    assert!(execute.properties.is_empty());
    let graphql_parse = spans
      .iter()
      .find(|v| v.name == "graphql_parse")
      .expect("failed to find span");
    assert!(graphql_parse.properties.is_empty());
    assert_eq!(graphql_parse.parent_id, execute.span_id);
    let query = spans
      .iter()
      .find(|v| v.name == "query")
      .expect("failed to find span");
    assert_eq!(query.properties.len(), 2);
    assert_eq!(
      query.properties[0],
      (GRAPHQL_DOCUMENT.into(), "query { __typename }".into())
    );
    assert_eq!(
      query.properties[1],
      (GRAPHQL_OPERATION_TYPE.into(), "query".into())
    );
    assert_eq!(query.parent_id, execute.span_id);

    let upstream_call = spans
      .iter()
      .find(|v| v.name == "upstream_call")
      .expect("failed to find span");
    assert_eq!(upstream_call.properties.len(), 1);
    assert_eq!(
      upstream_call.properties[0],
      (CONDUCTOR_SOURCE.into(), "test".into())
    );
    assert_eq!(upstream_call.parent_id, query.span_id);

    let upstream_http_post = spans
      .iter()
      .find(|v| v.name == "POST /graphql")
      .expect("failed to find span");
    assert_eq!(upstream_http_post.properties.len(), 8);
    assert_eq!(upstream_http_post.parent_id, upstream_call.span_id);
    assert_eq!(
      upstream_http_post.properties[0],
      (HTTP_METHOD.into(), "POST".into())
    );
    assert_eq!(
      upstream_http_post.properties[1],
      (HTTP_SCHEME.into(), "http".into())
    );
    assert_eq!(
      upstream_http_post.properties[2],
      (HTTP_HOST.into(), "127.0.0.1".into())
    );
    assert_eq!(upstream_http_post.properties[3].0, HTTP_URL);
    assert_eq!(upstream_http_post.properties[4].0, NET_HOST_PORT);
    assert_eq!(
      upstream_http_post.properties[5],
      (OTEL_KIND.into(), "client".into())
    );
    assert_eq!(
      upstream_http_post.properties[6],
      (SPAN_KIND.into(), "consumer".into())
    );
    assert_eq!(
      upstream_http_post.properties[7],
      (HTTP_STATUS_CODE.into(), "200".into())
    );
  }
}
