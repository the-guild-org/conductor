use graphql_parser::{
    parse_query,
    query::{Definition, Document, Field, OperationDefinition, Selection},
};
use rayon::vec;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldNode {
    pub field: String,
    pub alias: Option<String>,
    pub arguments: Vec<QueryArgument>,
    pub children: Vec<FieldNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}
impl Display for OperationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Query => write!(f, "query"),
            OperationType::Mutation => write!(f, "mutation"),
            OperationType::Subscription => write!(f, "subscription"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryArgument {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryDefinedArgument {
    pub name: String,
    pub default_value: Option<String>,
}

type QueryDefinedArguments = Vec<QueryDefinedArgument>;

pub type Fragments = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery {
    // Note: We don't currently need to track the operation name, it's useless for query planning, can be useful for analytics later on...
    pub operation_type: OperationType,
    pub arguments: Vec<QueryDefinedArgument>,
    pub fields: Vec<FieldNode>,
    pub fragments: Fragments,
}

fn seek_root_fields_capacity(parsed_query: &Document<'_, String>) -> usize {
    parsed_query
        .definitions
        .iter()
        .find_map(|e| match e {
            Definition::Operation(val) => match val {
                OperationDefinition::Query(e) => Some(e.selection_set.items.len()),
                OperationDefinition::Mutation(e) => Some(e.selection_set.items.len()),
                OperationDefinition::Subscription(e) => Some(e.selection_set.items.len()),
                OperationDefinition::SelectionSet(e) => Some(e.items.len()),
            },
            _ => None,
        })
        .unwrap_or(0)
}

pub fn parse_user_query(query: &str) -> UserQuery {
    let parsed_query = parse_query::<String>(&query);

    let mut user_query = UserQuery {
        operation_type: OperationType::Query,
        arguments: vec![],
        fields: Vec::with_capacity(seek_root_fields_capacity(parsed_query.as_ref().unwrap())),
        fragments: HashMap::new(),
    };

    match parsed_query {
        Ok(query) => {
            for definition in query.definitions {
                match definition {
                    Definition::Operation(OperationDefinition::Query(q)) => {
                        user_query.operation_type = OperationType::Query;

                        user_query.arguments = q
                            .variable_definitions
                            .into_iter()
                            .map(|e| QueryDefinedArgument {
                                name: e.name,
                                default_value: e.default_value.map(|e| e.to_string()),
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(&user_query.arguments, q.selection_set));
                    }
                    Definition::Operation(OperationDefinition::Mutation(m)) => {
                        user_query.operation_type = OperationType::Mutation;

                        user_query.arguments = m
                            .variable_definitions
                            .into_iter()
                            .map(|e| QueryDefinedArgument {
                                name: e.name,
                                default_value: e.default_value.map(|e| e.to_string()),
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(&user_query.arguments, m.selection_set));
                    }
                    Definition::Operation(OperationDefinition::Subscription(s)) => {
                        user_query.operation_type = OperationType::Subscription;

                        user_query.arguments = s
                            .variable_definitions
                            .into_iter()
                            .map(|e| QueryDefinedArgument {
                                name: e.name,
                                default_value: e.default_value.map(|e| e.to_string()),
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(&user_query.arguments, s.selection_set));
                    }
                    Definition::Operation(OperationDefinition::SelectionSet(e)) => {
                        user_query.fields = handle_selection_set(&user_query.arguments, e);
                    }
                    Definition::Fragment(e) => {
                        user_query
                            .fragments
                            .insert(e.name.to_string(), format!("{}", e));
                    }
                    _ => {}
                }
            }
        }
        Err(e) => println!("Failed to parse the query: {:#?}", e),
    }

    user_query
}

fn handle_selection_set(
    defined_arguments: &QueryDefinedArguments,
    selection_set: graphql_parser::query::SelectionSet<'_, String>,
) -> Vec<FieldNode> {
    let mut fields = Vec::with_capacity(selection_set.items.len());

    for selection in selection_set.items {
        match selection {
            Selection::Field(Field {
                name,
                selection_set: field_selection_set,
                arguments,
                alias,
                ..
            }) => {
                let children = handle_selection_set(defined_arguments, field_selection_set);

                let arguments = arguments
                    .into_iter()
                    .map(|(arg_name, value)| {
                        let value = value.to_string();
                        let value = if value.starts_with("$") {
                            defined_arguments
                                .iter()
                                .find(|e| e.name == value[1..])
                                .expect(
                                    format!("Argument {} is used but was never defined!", value)
                                        .as_str(),
                                )
                                .default_value
                                .as_ref()
                                .expect(format!("No default value for {}!", value).as_str())
                                .to_string()
                        } else {
                            value
                        };

                        QueryArgument {
                            name: arg_name,
                            value,
                        }
                    })
                    .collect();

                fields.push(FieldNode {
                    field: name,
                    children,
                    alias,
                    arguments,
                });
            }
            Selection::FragmentSpread(e) => {
                fields.push(FieldNode {
                    field: format!("...{}", e.fragment_name),
                    children: vec![],
                    alias: None,
                    arguments: vec![],
                });
            }
            _ => {}
        }
    }

    fields
}
