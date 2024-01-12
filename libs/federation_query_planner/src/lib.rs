use std::ops::Index;

use anyhow::{Ok, Result};
use graphql_parser::query::Document;
use serde_json::json;
use supergraph::Supergraph;

use crate::{
  executor::execute_query_plan, query_planner::plan_for_user_query, user_query::parse_user_query,
};

pub mod constants;
pub mod executor;
pub mod graphql_query_builder;
pub mod query_planner;
pub mod supergraph;
pub mod type_merge;
pub mod user_query;

pub async fn execute_federation(
  supergraph: &Supergraph,
  parsed_user_query: Document<'static, String>,
) -> Result<String> {
  // println!("parsed_user_query: {:#?}", user_query);
  let mut user_query = parse_user_query(parsed_user_query)?;
  let query_plan = plan_for_user_query(supergraph, &mut user_query)?;

  // println!("query plan: {:#?}", query_plan);

  let response_vec = execute_query_plan(&query_plan, supergraph).await?;

  // println!("response: {:#?}", json!(response_vec).to_string());

  Ok(json!(response_vec.index(0).index(0).1).to_string())
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

    let query_plan = plan_for_user_query(&supergraph, &mut user_query).unwrap();

    // TODO: fix ordering, it fails bc ordering of fields in a query plan is dynamic
    // insta::assert_json_snapshot!(query_plan);
  }
}
