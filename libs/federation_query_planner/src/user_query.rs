use anyhow::{Ok, Result};
use graphql_parser::query::{Definition, Document, Field, OperationDefinition, Selection};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldNode {
    pub field: String,
    pub alias: Option<String>,
    pub arguments: Vec<QueryArgument>,
    pub children: Vec<FieldNode>,
    pub sources: Vec<String>,
    pub type_name: Option<String>,
    pub parent_type_name: Option<String>,
    pub key_fields: Option<String>,
    pub owner: Option<String>,
    pub requires: Option<String>,
    pub should_be_cleaned: bool,
    pub relevant_sub_queries: Option<Vec<(String, String)>>,
    pub is_introspection: bool,
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
            // can be `query`, but why not save some space
            OperationType::Query => write!(f, ""),
            OperationType::Mutation => write!(f, "mutation"),
            OperationType::Subscription => write!(f, "subscription"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphQLFragment {
    pub str_definition: String,
    pub fields: Vec<FieldNode>,
}

pub type Fragments = HashMap<String, GraphQLFragment>;

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

pub fn parse_user_query(parsed_query: Document<'static, String>) -> Result<UserQuery> {
    let mut user_query = UserQuery {
        operation_type: OperationType::Query,
        arguments: vec![],
        fields: Vec::with_capacity(seek_root_fields_capacity(&parsed_query)),
        fragments: HashMap::new(),
    };

    for definition in parsed_query.definitions {
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

                user_query.fields.extend(handle_selection_set(
                    &user_query.arguments,
                    q.selection_set,
                )?);
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

                user_query.fields.extend(handle_selection_set(
                    &user_query.arguments,
                    m.selection_set,
                )?);
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

                user_query.fields.extend(handle_selection_set(
                    &user_query.arguments,
                    s.selection_set,
                )?);
            }
            Definition::Operation(OperationDefinition::SelectionSet(e)) => {
                user_query.fields = handle_selection_set(&user_query.arguments, e)?;
            }
            Definition::Fragment(e) => {
                user_query.fragments.insert(
                    e.name.to_string(),
                    GraphQLFragment {
                        str_definition: format!("{}", e),
                        fields: handle_selection_set(&user_query.arguments, e.selection_set)?,
                    },
                );
            }
        }
    }

    Ok(user_query)
}

fn handle_selection_set(
    defined_arguments: &QueryDefinedArguments,
    selection_set: graphql_parser::query::SelectionSet<'_, String>,
) -> Result<Vec<FieldNode>> {
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
                let is_introspection = name.starts_with("__");
                let (name, children) = if is_introspection {
                    (format!("{name}{}", field_selection_set.to_string()), vec![])
                } else {
                    (
                        name,
                        handle_selection_set(defined_arguments, field_selection_set)?,
                    )
                };

                let arguments = arguments
                    .into_iter()
                    .map(|(arg_name, value)| {
                        let value = value.to_string();
                        let value = if value.starts_with('$') {
                            defined_arguments
                                .iter()
                                .find(|e| e.name == value[1..])
                                .expect(&format!(
                                    "Argument {} is used but was never defined!",
                                    value
                                ))
                                .default_value
                                .as_ref()
                                .expect(&format!("No default value for {}!", value))
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
                    parent_type_name: None,
                    sources: vec![],
                    type_name: None,
                    key_fields: None,
                    owner: None,
                    requires: None,
                    should_be_cleaned: false,
                    relevant_sub_queries: None,
                    is_introspection,
                });
            }
            Selection::FragmentSpread(e) => {
                fields.push(FieldNode {
                    field: format!("...{}", e.fragment_name),
                    children: vec![],
                    alias: None,
                    arguments: vec![],
                    parent_type_name: None,
                    sources: vec![],
                    type_name: None,
                    key_fields: None,
                    owner: None,
                    requires: None,
                    should_be_cleaned: false,
                    relevant_sub_queries: None,
                    is_introspection: false,
                });
            }
            _ => {}
        }
    }

    Ok(fields)
}
