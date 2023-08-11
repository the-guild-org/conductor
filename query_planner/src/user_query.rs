use graphql_parser::{
    parse_query,
    query::{Definition, Field, OperationDefinition, Selection},
    schema::Value,
};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct FieldNode<'a> {
    pub field: String,
    pub alias: Option<String>,
    pub arguments: Vec<(String, Value<'a, String>)>,
    pub children: Vec<FieldNode<'a>>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct UserQuery<'a> {
    pub operation_type: OperationType,
    pub operation_name: Option<String>,
    pub arguments: Vec<(String, String, Option<Value<'a, String>>)>,
    pub fields: Vec<FieldNode<'a>>,
}

pub fn parse_user_query<'a>(query: &'a str) -> UserQuery<'a> {
    let parsed_query = parse_query::<String>(&query);

    let mut user_query = UserQuery {
        operation_name: None,
        operation_type: OperationType::Query,
        arguments: vec![],
        fields: vec![],
    };

    match parsed_query {
        Ok(query) => {
            for definition in &query.definitions {
                match definition {
                    Definition::Operation(OperationDefinition::Query(q)) => {
                        user_query.operation_type = OperationType::Query;
                        user_query.operation_name = q.name.clone();

                        user_query.arguments = q
                            .variable_definitions
                            .iter()
                            .map(|e| {
                                (
                                    e.name.to_string(),
                                    e.var_type.to_string(),
                                    e.default_value.to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(q.selection_set.clone()));
                    }
                    Definition::Operation(OperationDefinition::Mutation(m)) => {
                        user_query.operation_type = OperationType::Mutation;
                        user_query.operation_name = m.name.clone();

                        user_query.arguments = m
                            .variable_definitions
                            .iter()
                            .map(|e| {
                                (
                                    e.name.to_string(),
                                    e.var_type.to_string(),
                                    e.default_value.to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(m.selection_set.clone()));
                    }
                    Definition::Operation(OperationDefinition::Subscription(s)) => {
                        user_query.operation_type = OperationType::Subscription;
                        user_query.operation_name = s.name.clone();

                        user_query.arguments = s
                            .variable_definitions
                            .iter()
                            .map(|e| {
                                (
                                    e.name.to_string(),
                                    e.var_type.to_string(),
                                    e.default_value.to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(s.selection_set.clone()));
                    }
                    Definition::Operation(OperationDefinition::SelectionSet(q)) => {
                        user_query.fields = handle_selection_set(q.clone());
                    }
                    _ => {}
                }
            }
        }
        Err(e) => println!("Failed to parse the query: {:#?}", e),
    }

    user_query
}

fn handle_selection_set<'a>(
    selection_set: graphql_parser::query::SelectionSet<'a, String>,
) -> Vec<FieldNode<'a>> {
    let mut fields = Vec::new();

    for selection in &selection_set.items {
        if let Selection::Field(Field {
            name,
            selection_set: field_selection_set,
            arguments,
            alias,
            ..
        }) = selection
        {
            let children = handle_selection_set(field_selection_set.clone());

            let x = arguments
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            fields.push(FieldNode {
                field: name.to_string(),
                children,
                alias: alias.to_owned(),
                arguments: x,
            });
        } else if let Selection::Field(Field { name, .. }) = selection {
            fields.push(FieldNode {
                field: name.to_string(),
                children: Vec::new(),
                alias: None,
                arguments: Vec::new(),
            });
        }
    }

    fields
}
