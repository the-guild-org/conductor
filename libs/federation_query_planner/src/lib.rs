use std::{ops::Index, sync::Arc};

use anyhow::{anyhow, Error};
use conductor_common::http::{ConductorHttpRequest, HttpHeadersMap};
use conductor_common::{execute::RequestExecutionContext, plugin_manager::PluginManager};
use constants::CONDUCTOR_INTERNAL_SERVICE_RESOLVER;
use executor::{
  dynamically_build_schema_from_supergraph, find_objects_matching_criteria, QueryResponse,
};
use futures::future::join_all;
use graphql_parser::query::Document;
use minitrace::Span;
use query_planner::QueryStep;
use query_planner::{Parallel, QueryPlan};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use reqwest::Method;
use serde_json::json;
use serde_json::Value as SerdeValue;
use supergraph::Supergraph;

use crate::{query_planner::plan_for_user_query, user_query::parse_user_query};

pub mod constants;
pub mod executor;
pub mod graphql_query_builder;
pub mod query_planner;
pub mod supergraph;
pub mod type_merge;
pub mod user_query;

pub struct FederationExecutor<'a> {
  pub client: &'a minitrace_reqwest::TracedHttpClient,
  pub plugin_manager: Arc<Box<dyn PluginManager>>,
  pub supergraph: &'a Supergraph,
}

impl<'a> FederationExecutor<'a> {
  pub async fn execute_federation(
    &mut self,
    request_context: &'a mut RequestExecutionContext,
    parsed_user_query: Document<'static, String>,
  ) -> Result<(String, QueryPlan), Error> {
    // println!("parsed_user_query: {:#?}", user_query);
    let mut user_query = parse_user_query(parsed_user_query)?;
    let query_plan = plan_for_user_query(self.supergraph, &mut user_query)?;

    // println!("query plan: {:#?}", query_plan);

    let response_vec = self
      .execute_query_plan(&query_plan, request_context)
      .await?;

    // println!("response: {:#?}", json!(response_vec).to_string());

    Ok((
      json!(response_vec.index(0).index(0).1).to_string(),
      query_plan,
    ))
  }

  pub async fn execute_query_plan(
    &self,
    query_plan: &QueryPlan,
    request_context: &'a mut RequestExecutionContext,
  ) -> Result<Vec<Vec<((String, String), QueryResponse)>>, Error> {
    let mut all_futures = Vec::new();

    for step in &query_plan.parallel_steps {
      match step {
        Parallel::Sequential(query_steps) => {
          let future = self.execute_sequential(query_steps, request_context);
          all_futures.push(future);
        }
      }
    }

    let results: Result<Vec<_>, _> = join_all(all_futures).await.into_iter().collect();

    match results {
      Ok(val) => Ok(val),
      Err(e) => Err(anyhow!(e)),
    }
  }

  pub async fn execute_sequential(
    &self,
    query_steps: &Vec<QueryStep>,
    request_context: &'a mut RequestExecutionContext,
  ) -> Result<Vec<((String, String), QueryResponse)>, Error> {
    let mut data_vec = vec![];
    let mut entity_arguments: Option<SerdeValue> = None;

    for (i, query_step) in query_steps.iter().enumerate() {
      let data = self
        .execute_query_step(query_step, entity_arguments.clone(), request_context)
        .await;

      match data {
        Ok(data) => {
          data_vec.push((
            (query_step.service_name.clone(), query_step.query.clone()),
            data,
          ));

          if i + 1 < query_steps.len() {
            let next_step = &query_steps[i + 1];
            match &next_step.entity_query_needs {
              Some(needs) => {
                data_vec.iter().find(|&data| {
                  if let Some(x) = data.1.data.as_ref() {
                    // recursively search and find match
                    let y = find_objects_matching_criteria(
                      x,
                      &needs.__typename,
                      &needs.fields.clone().into_iter().next().unwrap(),
                    );

                    if y.is_empty() {
                      return false;
                    } else {
                      entity_arguments = Some(SerdeValue::from(y));
                      return true;
                    }
                  }

                  false
                });

                Some(serde_json::json!({ "representations": entity_arguments }))
              }
              None => None,
            }
          } else {
            None
          };
        }
        Err(err) => return Err(err),
      }
    }

    let x: Vec<((String, String), QueryResponse)> = data_vec
      .into_iter()
      .map(|(plan_meta, response)| {
        let new_response = QueryResponse {
          data: response.data,
          // Initialize other fields of QueryResponse as needed
          errors: response.errors,
          extensions: None,
        };
        (plan_meta, new_response)
      })
      .collect::<Vec<((std::string::String, std::string::String), QueryResponse)>>();

    Ok(x)
  }

  pub async fn execute_query_step(
    &self,
    query_step: &QueryStep,
    entity_arguments: Option<SerdeValue>,
    request_context: &'a mut RequestExecutionContext,
  ) -> Result<QueryResponse, Error> {
    let is_introspection = query_step.service_name == CONDUCTOR_INTERNAL_SERVICE_RESOLVER;

    if is_introspection {
      let schema = dynamically_build_schema_from_supergraph(self.supergraph);

      // Execute the introspection query
      // TODO: whenever excuting a query step, we need to take the query out of the step's struct instead of copying it
      let request = async_graphql::Request::new(query_step.query.to_string());
      let response = schema.execute(request).await;

      let data = serde_json::to_value(response.data)?;
      let errors = response
        .errors
        .iter()
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();

      Ok(QueryResponse {
        data: Some(data),
        errors: Some(errors),
        extensions: None,
      })
    } else {
      let _span = Span::enter_with_local_parent(format!("subgraph {}", query_step.service_name))
        .with_properties(|| {
          [
            ("service_name", query_step.service_name.clone()),
            ("graphql.document", query_step.query.clone()),
          ]
        });
      let url = self
        .supergraph
        .subgraphs
        .get(&query_step.service_name)
        .unwrap();

      let variables_object = if let Some(arguments) = &entity_arguments {
        serde_json::json!({ "representations": arguments })
      } else {
        SerdeValue::Object(serde_json::Map::new())
      };

      let mut upstream_request = ConductorHttpRequest {
        method: Method::POST,
        body: serde_json::json!({
            "query": query_step.query,
            "variables": variables_object
        })
        .to_string()
        .into(),
        uri: url.to_string(),
        query_string: "".to_string(),
        headers: Default::default(),
      };

      upstream_request
        .headers
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

      self
        .plugin_manager
        .on_upstream_http_request(request_context, &mut upstream_request)
        .await;

      if request_context.is_short_circuit() {
        return Err(anyhow::anyhow!("short circuit"));
      }

      let upstream_req = self
        .client
        .request(upstream_request.method, upstream_request.uri)
        .headers(upstream_request.headers)
        .body(upstream_request.body);

      let response = match upstream_req.send().await {
        Ok(resp) => resp,
        Err(err) => {
          eprintln!("Failed to send request: {}", err);
          return Err(anyhow::anyhow!("Failed to send request: {}", err));
        }
      };

      if !response.status().is_success() {
        eprintln!("Received error response: {:?}", response.status());
        return Err(anyhow::anyhow!(
          "Failed request with status: {}",
          response.status()
        ));
      }

      let response_data = match response.json::<QueryResponse>().await {
        Ok(data) => data,
        Err(err) => {
          eprintln!("Failed to parse response: {}", err);
          return Err(anyhow::anyhow!("Failed to parse response: {}", err));
        }
      };

      // Check if there were any GraphQL errors
      if let Some(errors) = &response_data.errors {
        for error in errors {
          eprintln!("Error: {:?}", error);
        }
      }

      Ok(response_data)
    }
  }
}

#[cfg(test)]
mod tests {

  #[tokio::test]
  async fn generates_query_plan() {
    use crate::{
      query_planner::plan_for_user_query, supergraph::parse_supergraph,
      user_query::parse_user_query,
    };

    let query = r#"
          fragment User on User {
              id
              username
              name
          }
  
          fragment Review on Review {
              id
              body
          }
  
          fragment Product on Product {
              inStock
              price
              shippingEstimate
              upc
              weight
              name
          }
  
          query TestQuery {
              users {
                  ...User
                  reviews {
                      ...Review
                      product {
                          ...Product
                          reviews {
                              ...Review
                          }
                      }
                  }
              }
          }
  "#;

    let supergraph_schema = r#"schema
    @link(url: "https://specs.apollo.dev/link/v1.0")
    @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION) {
    query: Query
  }
  
  directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE
  
  directive @join__field(
    graph: join__Graph
    requires: join__FieldSet
    provides: join__FieldSet
    type: String
    external: Boolean
    override: String
    usedOverridden: Boolean
  ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION
  
  directive @join__graph(name: String!, url: String!) on ENUM_VALUE
  
  directive @join__implements(
    graph: join__Graph!
    interface: String!
  ) repeatable on OBJECT | INTERFACE
  
  directive @join__type(
    graph: join__Graph!
    key: join__FieldSet
    extension: Boolean! = false
    resolvable: Boolean! = true
    isInterfaceObject: Boolean! = false
  ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR
  
  directive @join__unionMember(
    graph: join__Graph!
    member: String!
  ) repeatable on UNION
  
  directive @link(
    url: String
    as: String
    for: link__Purpose
    import: [link__Import]
  ) repeatable on SCHEMA
  
  scalar join__FieldSet
  
  enum join__Graph {
    ACCOUNTS @join__graph(name: "accounts", url: "http://localhost:5000/graphql")
    INVENTORY
      @join__graph(name: "inventory", url: "http://localhost:5001/graphql")
    PRODUCTS @join__graph(name: "products", url: "http://localhost:5002/graphql")
    REVIEWS @join__graph(name: "reviews", url: "http://localhost:5003/graphql")
  }
  
  scalar link__Import
  
  enum link__Purpose {
    """
    `SECURITY` features provide metadata necessary to securely resolve fields.
    """
    SECURITY
  
    """
    `EXECUTION` features provide metadata necessary for operation execution.
    """
    EXECUTION
  }
  
  type Product
    @join__type(graph: INVENTORY, key: "upc")
    @join__type(graph: PRODUCTS, key: "upc")
    @join__type(graph: REVIEWS, key: "upc") {
    upc: String!
    weight: Int
      @join__field(graph: INVENTORY, external: true)
      @join__field(graph: PRODUCTS)
    price: Int
      @join__field(graph: INVENTORY, external: true)
      @join__field(graph: PRODUCTS)
    inStock: Boolean @join__field(graph: INVENTORY)
    shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
    name: String @join__field(graph: PRODUCTS)
    reviews: [Review] @join__field(graph: REVIEWS)
  }
  
  type Query
    @join__type(graph: ACCOUNTS)
    @join__type(graph: INVENTORY)
    @join__type(graph: PRODUCTS)
    @join__type(graph: REVIEWS) {
    me: User @join__field(graph: ACCOUNTS)
    user(id: ID!): User @join__field(graph: ACCOUNTS)
    users: [User] @join__field(graph: ACCOUNTS)
    topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
  }
  
  type Review @join__type(graph: REVIEWS, key: "id") {
    id: ID!
    body: String
    product: Product
    author: User @join__field(graph: REVIEWS, provides: "username")
  }
  
  type User
    @join__type(graph: ACCOUNTS, key: "id")
    @join__type(graph: REVIEWS, key: "id") {
    id: ID!
    name: String @join__field(graph: ACCOUNTS)
    username: String
      @join__field(graph: ACCOUNTS)
      @join__field(graph: REVIEWS, external: true)
    birthday: Int @join__field(graph: ACCOUNTS)
    reviews: [Review] @join__field(graph: REVIEWS)
  }
  "#
    .to_string();

    let _supergraph = parse_supergraph(&supergraph_schema).unwrap();
    let _user_query = parse_user_query(graphql_parser::parse_query(query).unwrap());

    let supergraph_schema = r#"schema
  @link(url: "https://specs.apollo.dev/link/v1.0")
  @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION) {
  query: Query
}

directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

directive @join__field(
  graph: join__Graph
  requires: join__FieldSet
  provides: join__FieldSet
  type: String
  external: Boolean
  override: String
  usedOverridden: Boolean
) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

directive @join__implements(
  graph: join__Graph!
  interface: String!
) repeatable on OBJECT | INTERFACE

directive @join__type(
  graph: join__Graph!
  key: join__FieldSet
  extension: Boolean! = false
  resolvable: Boolean! = true
  isInterfaceObject: Boolean! = false
) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

directive @join__unionMember(
  graph: join__Graph!
  member: String!
) repeatable on UNION

directive @link(
  url: String
  as: String
  for: link__Purpose
  import: [link__Import]
) repeatable on SCHEMA

scalar join__FieldSet

enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://localhost:5000/graphql")
  INVENTORY
    @join__graph(name: "inventory", url: "http://localhost:5001/graphql")
  PRODUCTS @join__graph(name: "products", url: "http://localhost:5002/graphql")
  REVIEWS @join__graph(name: "reviews", url: "http://localhost:5003/graphql")
}

scalar link__Import

enum link__Purpose {
  """
  `SECURITY` features provide metadata necessary to securely resolve fields.
  """
  SECURITY

  """
  `EXECUTION` features provide metadata necessary for operation execution.
  """
  EXECUTION
}

type Product
  @join__type(graph: INVENTORY, key: "upc")
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: REVIEWS, key: "upc") {
  upc: String!
  weight: Int
    @join__field(graph: INVENTORY, external: true)
    @join__field(graph: PRODUCTS)
  price: Int
    @join__field(graph: INVENTORY, external: true)
    @join__field(graph: PRODUCTS)
  inStock: Boolean @join__field(graph: INVENTORY)
  shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
  name: String @join__field(graph: PRODUCTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}

type Query
  @join__type(graph: ACCOUNTS)
  @join__type(graph: INVENTORY)
  @join__type(graph: PRODUCTS)
  @join__type(graph: REVIEWS) {
  me: User @join__field(graph: ACCOUNTS)
  user(id: ID!): User @join__field(graph: ACCOUNTS)
  users: [User] @join__field(graph: ACCOUNTS)
  topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
}

type Review @join__type(graph: REVIEWS, key: "id") {
  id: ID!
  body: String
  product: Product
  author: User @join__field(graph: REVIEWS, provides: "username")
}

type User
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id") {
  id: ID!
  name: String @join__field(graph: ACCOUNTS)
  username: String
    @join__field(graph: ACCOUNTS)
    @join__field(graph: REVIEWS, external: true)
  birthday: Int @join__field(graph: ACCOUNTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}
"#
    .to_string();

    let supergraph = parse_supergraph(&supergraph_schema).unwrap();
    let mut user_query = parse_user_query(graphql_parser::parse_query(query).unwrap()).unwrap();

    let _query_plan = plan_for_user_query(&supergraph, &mut user_query).unwrap();

    // TODO: fix ordering, it fails bc ordering of fields in a query plan is dynamic
    // insta::assert_json_snapshot!(query_plan);
  }
}
