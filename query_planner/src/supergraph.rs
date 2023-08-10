use std::{collections::HashMap, error::Error};

use graphql_parser::{
    parse_schema,
    schema::{Definition as SchemaDefinition, TypeDefinition, Value},
};

#[derive(Debug, Default, PartialEq)]
pub struct GraphQLField {
    pub field_type: String,
    pub source: String,
    pub requires: Option<String>,
    pub provides: Option<String>,
    pub external: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct GraphQLType {
    pub key_fields: Vec<String>,
    pub fields: HashMap<String, GraphQLField>,
}

#[derive(Debug, Default)]
pub struct Supergraph {
    pub types: HashMap<String, GraphQLType>,
    pub services: HashMap<String, String>,
}

fn get_argument_value<'a>(
    args: Vec<(String, Value<'a, String>)>,
    key: &str,
) -> Option<Value<'a, String>> {
    args.into_iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

pub fn parse_supergraph(supergraph_schema: &str) -> Result<Supergraph, Box<dyn Error>> {
    let result = parse_schema::<String>(&supergraph_schema)?;

    let mut desired_structure = Supergraph::default();

    for e in result.definitions {
        match e {
            SchemaDefinition::TypeDefinition(ope) => match ope {
                TypeDefinition::Enum(a) => {
                    for value in a.values {
                        if let Some(directive) = value.directives.first() {
                            if directive.name == "join__graph" {
                                let name = get_argument_value(directive.arguments.clone(), "name")
                                    .unwrap()
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string()
                                    .to_uppercase();
                                let url = get_argument_value(directive.arguments.clone(), "url")
                                    .unwrap()
                                    .to_string()
                                    .trim_matches('"')
                                    .to_string();

                                desired_structure.services.insert(name, url);
                            }
                        }
                    }
                }
                TypeDefinition::Object(obj) => {
                    let mut desired_type = GraphQLType::default();
                    let mut parent_type = None;

                    for directive in &obj.directives {
                        match directive.name.as_str() {
                            "join__type" => {
                                if let Some(graph) =
                                    get_argument_value(directive.arguments.clone(), "graph")
                                {
                                    parent_type = Some(graph.to_string());
                                    if let Some(key) =
                                        get_argument_value(directive.arguments.clone(), "key")
                                    {
                                        let key_string =
                                            key.to_string().trim_matches('"').to_string();
                                        if !desired_type.key_fields.contains(&key_string) {
                                            desired_type.key_fields.push(key_string);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    for field in &obj.fields {
                        let mut desired_field = GraphQLField {
                            source: parent_type.clone().unwrap_or_default(),
                            field_type: field.field_type.to_string(),
                            requires: None,
                            provides: None,
                            external: false,
                        };

                        for field_directive in &field.directives {
                            if field_directive.name == "join__field" {
                                for (k, v) in &field_directive.arguments {
                                    match k.as_str() {
                                        "graph" => {
                                            if field_directive
                                                .arguments
                                                .iter()
                                                .find(|(key, val)| {
                                                    key == "external" && val.to_string() == "true"
                                                })
                                                .is_none()
                                            {
                                                desired_field.source = v.to_string();
                                            }
                                        }
                                        "requires" => {
                                            desired_field.requires = Some(v.to_string());
                                        }
                                        "provides" => {
                                            desired_field.provides = Some(v.to_string());
                                        }
                                        "external" => {
                                            desired_field.external = v.to_string() == "true";
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        desired_type
                            .fields
                            .insert(field.name.clone(), desired_field);
                    }

                    desired_structure
                        .types
                        .insert(obj.name.clone(), desired_type);
                }
                _ => {}
            },
            _ => {}
        }
    }

    if desired_structure.services.is_empty() || desired_structure.types.is_empty() {
        return Err("Couldn't find relevant directives in your supergraph schema!".into());
    }

    Ok(desired_structure)
}
