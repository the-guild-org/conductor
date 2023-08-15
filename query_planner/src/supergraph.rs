use std::{collections::HashMap, error::Error};

use graphql_parser::{
    parse_schema,
    schema::{Definition as SchemaDefinition, TypeDefinition, Value},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphQLField {
    pub field_type: String,
    pub source: String,
    pub requires: Option<String>,
    pub provides: Option<String>,
    pub external: bool,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphQLType {
    pub key_fields: Vec<String>,
    pub fields: HashMap<String, GraphQLField>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Supergraph {
    pub types: HashMap<String, GraphQLType>,
    pub subgraphs: HashMap<String, String>,
}

fn get_argument_value(args: &Vec<(String, Value<'_, String>)>, key: &str) -> Option<String> {
    args.into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.to_string().trim().to_string())
}

pub fn parse_supergraph(supergraph_schema: &str) -> Result<Supergraph, Box<dyn Error>> {
    let result = parse_schema::<String>(&supergraph_schema)?;

    let mut parsed_supergraph = Supergraph::default();

    for e in result.definitions {
        match e {
            SchemaDefinition::TypeDefinition(t) => match t {
                // 1. Get Subgraphs name and their corresponding URLs
                TypeDefinition::Enum(a) => {
                    for mut value in a.values {
                        // we aren't at the correct subgraphs enum definition if it is empty
                        if value.directives.len() <= 0 {
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
                    let mut graphql_type_subgraph = String::from("None");

                    for directive in obj.directives {
                        match directive.name.as_str() {
                            "join__type" => {
                                if let Some(graph) =
                                    get_argument_value(&directive.arguments, "graph")
                                {
                                    graphql_type_subgraph = graph;

                                    // 4. Get entity's keys
                                    if let Some(key) =
                                        get_argument_value(&directive.arguments, "key")
                                    {
                                        let key = key.to_string().trim_matches('"').to_string();
                                        if !graphql_type.key_fields.contains(&key) {
                                            graphql_type.key_fields.push(key);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    for field in obj.fields {
                        let mut graphql_type_field = GraphQLField {
                            source: graphql_type_subgraph.clone(),
                            field_type: field.field_type.to_string(),
                            requires: None,
                            provides: None,
                            external: false,
                        };

                        for field_directive in field.directives {
                            if field_directive.name == "join__field" {
                                for (k, v) in &field_directive.arguments {
                                    match k.as_str() {
                                        // 5. Get the field's subgraph owner
                                        "graph" => {
                                            if field_directive
                                                .arguments
                                                .iter()
                                                // We're excluding `@join__field(external: true)` because we want the owning subgraph not the one referencing it
                                                .find(|(key, val)| {
                                                    key == "external" && val.to_string() == "true"
                                                })
                                                .is_none()
                                            {
                                                graphql_type_field.source = v.to_string();
                                            }
                                        }
                                        // 6. Get other useful directives
                                        "requires" => {
                                            graphql_type_field.requires = Some(v.to_string());
                                        }
                                        "provides" => {
                                            graphql_type_field.provides = Some(v.to_string());
                                        }
                                        "external" => {
                                            graphql_type_field.external = v.to_string() == "true";
                                        }
                                        _ => {}
                                    }
                                }
                            }
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
            },
            _ => {}
        }
    }

    if parsed_supergraph.subgraphs.is_empty() || parsed_supergraph.types.is_empty() {
        return Err("Your Supergraph Schema doesn't seem to be correct! The Parser has resulted in 0 types, and 0 subgraphs.".into());
    }

    Ok(parsed_supergraph)
}
