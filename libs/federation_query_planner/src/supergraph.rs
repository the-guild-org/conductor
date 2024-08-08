use anyhow::{anyhow, Ok, Result};
use conductor_common::graphql::ParsedGraphQLSchema;
use graphql_parser::schema::{Definition as SchemaDefinition, TypeDefinition, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct GraphQLField {
  pub field_type: String,
  pub sources: Vec<String>,
  pub requires: Option<String>,
  pub provides: Option<String>,
  pub external: bool,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct GraphQLType {
  pub key_fields: Option<String>,
  pub fields: HashMap<String, GraphQLField>,
  pub owner: Option<String>,
}

impl GraphQLType {
  pub fn get_field(&self, name: &str, parent_type_name: &str) -> Result<&GraphQLField> {
    match self.fields.get(name) {
      Some(f) => Ok(f),
      None => Err(anyhow!(format!(
        "Field \"{}\" is not available on type {}",
        name, parent_type_name
      ))),
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Supergraph {
  pub types: HashMap<String, GraphQLType>,
  pub subgraphs: HashMap<String, String>,
}

impl<'a> Supergraph {
  pub fn get_gql_type(
    &'a self,
    name: &'a str,
    item_description: &'a str,
  ) -> Result<&'a GraphQLType> {
    match self.types.get(name) {
      Some(t) => Ok(t),
      None => {
        return Err(anyhow!(format!(
          "{item_description} \"{name}\" not defined in your in supergraph schema!",
        )))
      }
    }
  }
}

fn get_argument_value(args: &[(String, Value<'_, String>)], key: &str) -> Option<String> {
  args
    .iter()
    .find(|(k, _)| k == key)
    .map(|(_, v)| v.to_string().trim().to_string())
}

pub fn parse_supergraph(
  supergraph_schema: &ParsedGraphQLSchema,
) -> Result<Supergraph, anyhow::Error> {
  let result = supergraph_schema.clone();
  let mut parsed_supergraph = Supergraph::default();

  for e in result.definitions {
    if let SchemaDefinition::TypeDefinition(t) = e {
      match t {
        // 1. Get Subgraphs name and their corresponding URLs
        TypeDefinition::Enum(a) => {
          for mut value in a.values {
            // we aren't at the correct subgraphs enum definition if it is empty
            if value.directives.is_empty() {
              continue;
            }

            // Select the first one, because in any supergraph, there will always be just one defining the subgraphs
            // We're using `.remove(0)` here to get ownership over the first item, to avoid references + clones
            let directive = value.directives.remove(0);
            let arguments = directive.arguments;

            // `join__graph` enum contains a map of the subgraphs
            if directive.name == "join__graph" {
              let name = get_argument_value(&arguments, "name")
                .unwrap()
                .trim_matches('"')
                .to_uppercase();
              let url = get_argument_value(&arguments, "url")
                .unwrap()
                .trim_matches('"')
                .to_string();

              parsed_supergraph.subgraphs.insert(name, url);
            }
          }
        }
        TypeDefinition::Object(obj) => {
          // 2. Get each graphql type
          let mut graphql_type = GraphQLType::default();

          // 3. Get the subgraph, the type belongs to, this is useful in cases where the individual fields are not
          // annotated with a `@join__field(graph: $SUBGRAPH)`, and all the type's fields belong to the type's subgraph origin
          let mut graphql_type_subgraphs = Vec::new();

          for directive in obj.directives {
            match directive.name.as_str() {
              "join__type" => {
                if let Some(graph) = get_argument_value(&directive.arguments, "graph") {
                  graphql_type_subgraphs.push(graph);

                  // 4. Get entity's keys
                  if let Some(key) = get_argument_value(&directive.arguments, "key") {
                    let key = key.to_string().trim_matches('"').to_string();
                    graphql_type.key_fields = Some(key);
                  }
                }
              }
              "join__owner" => {
                if let Some(graph) = get_argument_value(&directive.arguments, "graph") {
                  graphql_type.owner = Some(graph.trim_matches('"').to_string());
                }
              }
              _ => {}
            }
          }

          for field in obj.fields {
            // start with an empty vector, intending to populate it with specific or inherited sources
            let mut specific_sources_found = false;
            let mut collected_sources = Vec::new(); // this will collect subgraphs specified by @join__field

            let mut graphql_type_field = GraphQLField {
              sources: Vec::new(), // We will assign the correct sources later
              field_type: field.field_type.to_string(),
              requires: None,
              provides: None,
              external: false,
            };

            // loop through each directive to configure the field
            for field_directive in field.directives {
              if field_directive.name == "join__field" {
                for (k, v) in &field_directive.arguments {
                  match k.as_str() {
                    "graph" => {
                      let subgraph = v.to_string().trim_matches('\"').to_string();
                      if !collected_sources.contains(&subgraph) {
                        collected_sources.push(subgraph);
                      }
                      specific_sources_found = true; // We have found specific sources for this field
                    }
                    "requires" => {
                      graphql_type_field.requires =
                        Some(v.to_string().trim_matches('\"').to_string());
                    }
                    "provides" => {
                      graphql_type_field.provides =
                        Some(v.to_string().trim_matches('\"').to_string());
                    }
                    "external" => {
                      graphql_type_field.external = v.to_string() == "true";
                    }
                    _ => {}
                  }
                }
              }
            }

            // decide on the sources to use: specific if any were found, otherwise inherit from parent type
            if specific_sources_found {
              graphql_type_field.sources = collected_sources;
            } else {
              graphql_type_field.sources = graphql_type_subgraphs.clone();
            }

            graphql_type
              .fields
              .insert(field.name.clone(), graphql_type_field);
          }
          parsed_supergraph
            .types
            .insert(obj.name.clone(), graphql_type);
        }
        _ => {}
      }
    }
  }

  if parsed_supergraph.subgraphs.is_empty() || parsed_supergraph.types.is_empty() {
    return Err(anyhow::anyhow!("Your Supergraph Schema doesn't seem to be correct! The Parser has resulted in 0 types, and 0 subgraphs."));
  }

  Ok(parsed_supergraph)
}

#[cfg(test)]
mod tests {
  use super::*;
  use graphql_parser::parse_schema;
  use insta::assert_debug_snapshot;

  #[test]
  fn test_parse_basic_supergraph() {
    let schema = r#"
    schema @link(url: "https://specs.apollo.dev/link/v1.0")
           @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION) {
      query: Query
    }

    directive @join__graph(name: String!, url: String!) on ENUM_VALUE
    directive @join__type(graph: join__Graph!, key: join__FieldSet) on OBJECT | INTERFACE

    enum join__Graph {
      ACCOUNTS @join__graph(name: "accounts", url: "http://0.0.0.0:4001/graphql")
    }

    type Query @join__type(graph: ACCOUNTS) {
      me: User @join__field(graph: ACCOUNTS)
    }

    type User @join__type(graph: ACCOUNTS, key: "id") {
      id: ID!
      name: String @join__field(graph: ACCOUNTS)
    }
    "#;

    let supergraph_schema = parse_schema(schema).expect("Failed to parse schema");
    let parsed_supergraph =
      parse_supergraph(&supergraph_schema).expect("Failed to parse supergraph");

    assert_debug_snapshot!(parsed_supergraph);
  }

  #[test]
  fn test_complex_directives_and_types() {
    let schema = r#"
    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        PRODUCTS @join__graph(name: "products", url: "http://0.0.0.0:4003/graphql")
        INVENTORY @join__graph(name: "inventory", url: "http://0.0.0.0:4002/graphql")
    }

    type Product @join__type(graph: PRODUCTS, key: "upc")
                 @join__type(graph: INVENTORY, key: "upc") {
        upc: String! @join__field(graph: PRODUCTS)
        weight: Int @join__field(graph: INVENTORY, external: true)
        price: Int @join__field(graph: PRODUCTS)
    }

    type Query @join__type(graph: PRODUCTS) {
        topProducts: [Product] @join__field(graph: PRODUCTS)
    }
    "#;

    let supergraph_schema = parse_schema(schema).expect("Failed to parse schema");
    let parsed_supergraph =
      parse_supergraph(&supergraph_schema).expect("Failed to parse supergraph");
    assert_debug_snapshot!(parsed_supergraph);
  }

  #[test]
  fn test_integration_with_subgraphs() {
    let schema = r#"
    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        ACCOUNTS @join__graph(name: "accounts", url: "http://0.0.0.0:4001/graphql")
        REVIEWS @join__graph(name: "reviews", url: "http://0.0.0.0:4004/graphql")
    }

    type Review @join__type(graph: REVIEWS, key: "id") {
        id: ID!
        body: String
        author: User @join__field(graph: ACCOUNTS, requires: "username")
    }

    type User @join__type(graph: ACCOUNTS, key: "id") {
        id: ID!
        username: String @join__field(graph: ACCOUNTS)
        reviews: [Review] @join__field(graph: REVIEWS)
    }
    "#;

    let supergraph_schema = parse_schema(schema).expect("Failed to parse schema");
    let parsed_supergraph =
      parse_supergraph(&supergraph_schema).expect("Failed to parse supergraph");
    assert_debug_snapshot!(parsed_supergraph);
  }

  #[test]
  fn test_external_fields_and_dependencies() {
    let schema = r#"
    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        INVENTORY @join__graph(name: "inventory", url: "http://0.0.0.0:4002/graphql")
        PRODUCTS @join__graph(name: "products", url: "http://0.0.0.0:4003/graphql")
    }

    type Inventory @join__type(graph: INVENTORY, key: "id") {
        id: ID!
        productID: String @join__field(graph: PRODUCTS, external: true)
        stockLevel: Int
    }
    "#;

    let supergraph_schema = parse_schema(schema).expect("Failed to parse schema");
    let parsed_supergraph =
      parse_supergraph(&supergraph_schema).expect("Failed to parse supergraph");
    assert_debug_snapshot!(parsed_supergraph);
  }

  #[test]
  fn test_recursive_type_references() {
    let schema = r#"
    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        PRODUCTS @join__graph(name: "products", url: "http://0.0.0.0:4003/graphql")
    }

    type Category @join__type(graph: PRODUCTS, key: "id") {
        id: ID!
        parentCategory: Category @join__field(graph: PRODUCTS)
        products: [Product] @join__field(graph: PRODUCTS)
    }
    "#;

    let supergraph_schema = parse_schema(schema).expect("Failed to parse schema");
    let parsed_supergraph =
      parse_supergraph(&supergraph_schema).expect("Failed to parse supergraph");
    assert_debug_snapshot!(parsed_supergraph);
  }
}
