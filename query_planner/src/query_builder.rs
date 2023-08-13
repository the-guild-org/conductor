use graphql_parser::schema::Value;

use crate::user_query::{FieldNode, OperationType, QueryArgument, UserQuery};

pub fn stringify_arguments<'a>(arguments: &Vec<QueryArgument>) -> String {
    let mut result = String::new();
    for QueryArgument { name, value, .. } in arguments {
        result.push_str(&format!("{}: {}, ", name, value));
    }
    result.trim_end_matches(", ").to_string()
}

pub fn stringify_query_arguments<'a>(
    arguments: &Vec<(
        std::string::String,
        std::string::String,
        Option<Value<'a, std::string::String>>,
    )>,
) -> String {
    let mut result = String::new();
    for (name, type_, opt_value) in arguments {
        match opt_value {
            Some(value) => {
                result.push_str(&format!(
                    "${}: {} = {}, ",
                    name,
                    type_,
                    format!("{}", value)
                ));
            }
            None => {
                result.push_str(&format!("${}: {}, ", name, type_));
            }
        }
    }
    result.trim_end_matches(", ").to_string()
}

// Recursive function to convert FieldNode to a GraphQL query string
fn field_node_to_string<'a>(field_node: &FieldNode) -> String {
    let mut result = String::new();
    if let Some(alias) = &field_node.alias {
        result.push_str(&format!("{}: ", alias));
    }
    result.push_str(&field_node.field);
    if !field_node.arguments.is_empty() {
        result.push_str(&format!("({})", stringify_arguments(&field_node.arguments)));
    }
    if !field_node.children.is_empty() {
        result.push_str(" {");
        for child in &field_node.children {
            result.push_str(&field_node_to_string(child));
        }
        result.push_str("}");
    }
    result.push_str(" ");
    result
}

// Function to convert UserQuery to a GraphQL query string
pub fn user_query_to_string(user_query: &UserQuery) -> String {
    let operation_type_str = match user_query.operation_type {
        OperationType::Query => "query",
        OperationType::Mutation => "mutation",
        OperationType::Subscription => "subscription",
    };

    let mut result = String::new();
    if let Some(operation_name) = &user_query.operation_name {
        result.push_str(&format!("{} {} ", operation_type_str, operation_name));
    } else {
        result.push_str(&format!("{} ", operation_type_str));
    }

    // if !user_query.arguments.is_empty() {
    //     result.push_str(&format!("({})", stringify_arguments(&user_query.arguments)));
    // }

    result.push_str(" {");
    for field_node in &user_query.fields {
        result.push_str(&field_node_to_string(field_node));
    }
    result.push_str("}");

    result
}
