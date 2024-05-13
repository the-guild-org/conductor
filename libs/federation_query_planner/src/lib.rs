use crate::{query_planner::plan_for_user_query, user_query::parse_user_query};
use anyhow::{Error, Ok as anyhowOk};
use conductor_common::{execute::RequestExecutionContext, plugin_manager::PluginManager};
use constants::CONDUCTOR_INTERNAL_SERVICE_RESOLVER;
use executor::{dynamically_build_schema_from_supergraph, get_dep_field_value, QueryResponse};
use futures::Future;
use graphql_parser::query::Document;
use minitrace::Span;
use no_deadlocks::RwLock as NoDeadlockRwLock;
use serde_json::json;
use serde_json::Value as SerdeValue;
use std::fs;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;
use supergraph::Supergraph;
use user_query::{FieldNode, UserQuery};

pub mod constants;
pub mod executor;
pub mod graphql_query_builder;
pub mod query_planner;
pub mod supergraph;
pub mod user_query;

pub fn unwrap_graphql_type(typename: &str) -> &str {
  let mut unwrapped = typename;

  // [Type!]!
  unwrapped = unwrapped.trim_end_matches('!');
  unwrapped = unwrapped.trim_start_matches('[');
  unwrapped = unwrapped.trim_end_matches(']');
  unwrapped = unwrapped.trim_end_matches('!');

  unwrapped
}

pub struct FederationExecutor<'a> {
  pub client: &'a minitrace_reqwest::TracedHttpClient,
  pub plugin_manager: Arc<Box<dyn PluginManager>>,
  pub supergraph: &'a Supergraph,
}

impl<'a> FederationExecutor<'a> {
  pub async fn execute_federation(
    &self,
    request_context: Arc<NoDeadlockRwLock<RequestExecutionContext>>,
    parsed_user_query: Document<'static, String>,
  ) -> Result<(String, UserQuery), Error> {
    let mut user_query = parse_user_query(parsed_user_query)?;

    plan_for_user_query(self.supergraph, &mut user_query)?;

    fs::write("plan.json", serde_json::json!(user_query).to_string())?;

    self
      .execute_query_plan(&mut user_query, request_context)
      .await?;

    fs::write("plan.json", serde_json::json!(user_query).to_string())?;

    // println!("parsed_user_query: {:#?}", user_query);

    // println!("responses {:#?}", response_vec);

    let response = user_query.to_json_result(&user_query.fields);

    // println!("response: {:#?}", response);

    anyhowOk((json!({"data": response}).to_string(), user_query))
  }

  pub async fn execute_query_plan(
    &self,
    user_query: &mut UserQuery,
    request_context: Arc<NoDeadlockRwLock<RequestExecutionContext>>,
  ) -> Result<(), Error> {
    // this all should run in paralell as it's the root Query
    // P.S: I think there might be root fields of the same subgraph, so it shouldn't send twice, will optimize this later on
    for field in &user_query.fields {
      let request_context_clone = Arc::clone(&request_context);
      self
        .execute_query_step(&user_query.fields, field.clone(), request_context_clone)
        .await?;
    }

    anyhowOk(())
  }

  pub async fn execute_query_step(
    &self,
    user_fields: &Vec<Arc<RwLock<FieldNode>>>,
    field: Arc<RwLock<FieldNode>>,
    request_context: Arc<NoDeadlockRwLock<RequestExecutionContext>>,
  ) -> Result<(), Error> {
    let query_step = {
      let x = field.read().unwrap();

      x.query_step.clone()
    };

    if let Some(query_step) = query_step {
      // if let Some(deps) = query_step.entity_query_needs_path.as_ref() {
      //   for field_path in deps {
      //     let (query_step, dep_field) = get_dep_field(field_path, user_fields.clone())?;

      //     // TODO: deduplicate this
      //     let span = Span::enter_with_local_parent(format!("subgraph {}", query_step.service_name))
      //       .with_properties(|| {
      //         [
      //           ("service_name", query_step.service_name.clone()),
      //           ("graphql.document", query_step.query.clone()),
      //         ]
      //       });
      //     let url = self
      //       .supergraph
      //       .subgraphs
      //       .get(&query_step.service_name)
      //       .unwrap();

      //     let variables_object = if let Some(dep_path) = query_step.entity_query_needs_path {
      //       // TODO: currently just assuming a single key field, but should be improved to handle more
      //       let value = get_dep_field_value(
      //         &dep_path[0],
      //         user_fields.clone(),
      //         query_step.entity_typename.unwrap(),
      //       )
      //       .unwrap()
      //       .unwrap();

      //       value
      //     } else {
      //       SerdeValue::Object(serde_json::Map::new())
      //     };

      //     // TODO: improve this by implementing https://github.com/the-guild-org/conductor-t2/issues/205
      //     let response = match self
      //       .client
      //       .post(url)
      //       .header("Content-Type", "application/json")
      //       .body(
      //         serde_json::json!({
      //             "query": query_step.query,
      //             "variables": variables_object
      //         })
      //         .to_string(),
      //       )
      //       .send()
      //       // .in_span(span)
      //       .await
      //     {
      //       Ok(resp) => resp,
      //       Err(err) => {
      //         eprintln!("Failed to send request: {}", err);
      //         return Err(anyhow::anyhow!("Failed to send request: {}", err));
      //       }
      //     };

      //     if !response.status().is_success() {
      //       eprintln!("Received error response: {:?}", response.status());
      //       return Err(anyhow::anyhow!(
      //         "Failed request with status: {}",
      //         response.status()
      //       ));
      //     }

      //     let response_data = match response.json::<QueryResponse>().await {
      //       Ok(data) => data,
      //       Err(err) => {
      //         eprintln!("Failed to parse response: {}", err);
      //         return Err(anyhow::anyhow!("Failed to parse response: {}", err));
      //       }
      //     };

      //     // Check if there were any GraphQL errors
      //     if let Some(errors) = &response_data.errors {
      //       for error in errors {
      //         eprintln!("Error: {:?}", error);
      //       }
      //     }

      //     // println!("{:#?}", response_data);

      //     dep_field.write().unwrap().response = Some(response_data);

      //     self
      //       .execute_child_steps(user_fields, dep_field, request_context.clone())
      //       .await?;
      //   }
      // }

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

        field.write().unwrap().response = Some(QueryResponse {
          data: Some(data),
          errors: Some(errors),
          extensions: None,
        });
      } else {
        let span = Span::enter_with_local_parent(format!("subgraph {}", query_step.service_name))
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

        let variables_object = if let Some(dep_path) = query_step.entity_query_needs_path {
          // TODO: currently just assuming a single key field, but should be improved to handle more
          get_dep_field_value(
            &dep_path[0],
            user_fields.clone(),
            query_step.entity_typename.unwrap(),
          )
          .unwrap()
          .unwrap()
        } else {
          SerdeValue::Object(serde_json::Map::new())
        };

        // println!("{:#?}", query_step.query);
        // println!("{:#?}", url);
        // println!("{:#?}", variables_object);

        field
          .write()
          .unwrap()
          .query_step
          .as_mut()
          .unwrap()
          .arguments = variables_object.clone();

        // TODO: improve this by implementing https://github.com/the-guild-org/conductor-t2/issues/205
        let response = match self
          .client
          .post(url)
          .header("Content-Type", "application/json")
          .body(
            serde_json::json!({
                "query": query_step.query,
                "variables": variables_object
            })
            .to_string(),
          )
          .send()
          .await
        {
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

        // check if there were any gql errors
        if let Some(errors) = &response_data.errors {
          for error in errors {
            eprintln!("Error: {:?}", error);
          }
        }

        field.write().unwrap().response = Some(response_data);
      }
    };

    self
      .execute_child_steps(user_fields, field, request_context)
      .await?;

    Ok(())
  }

  fn execute_child_steps<'b>(
    &'b self,
    user_fields: &'b Vec<Arc<RwLock<FieldNode>>>,
    field: Arc<RwLock<FieldNode>>,
    request_context: Arc<NoDeadlockRwLock<RequestExecutionContext>>,
  ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'b>> {
    Box::pin(async move {
      let children = field.read().unwrap().children.clone();
      for child in children {
        self
          .execute_query_step(user_fields, child.clone(), request_context.clone())
          .await?;
      }
      Ok(())
    })
  }
}

#[cfg(test)]
mod tests {
  use conductor_common::graphql::parse_graphql_schema;

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

    let schema = parse_graphql_schema(&supergraph_schema).unwrap();
    let supergraph = parse_supergraph(&schema).unwrap();
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

    let schema = parse_graphql_schema(&supergraph_schema).unwrap();
    let supergraph = parse_supergraph(&schema).unwrap();
    let mut user_query = parse_user_query(graphql_parser::parse_query(query).unwrap()).unwrap();

    let _query_plan = plan_for_user_query(&supergraph, &mut user_query).unwrap();

    // TODO: fix ordering, it fails bc ordering of fields in a query plan is dynamic
    // insta::assert_json_snapshot!(query_plan);
  }
}
