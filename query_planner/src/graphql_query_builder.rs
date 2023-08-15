use graphql_parser::schema::Value;

use crate::{
    query_planner::contains_entities_query,
    user_query::{FieldNode, Fragments, QueryArgument},
};

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

pub fn generate_entities_query(typename: String, selection_set: String) -> String {
    assert!(
        !typename.is_empty(),
        "Typename of the parent field must not be empty when generating an _entity query!"
    );
    format!(
        "_entities(representations: $representations) {{\n... on {} {{\n{}\n}}",
        typename, selection_set
    )
}

pub fn generate_query_for_field(
    operation_type: &str,
    field: &FieldNode,
    field_strings: &[String],
    fragments: &Fragments,
) -> String {
    let selection_set = field_strings.join(" ");
    let fragments_to_include = fragments
        .iter()
        .filter_map(|(name, definition)| {
            if selection_set.contains(&format!("...{}", name.to_string())) {
                Some(definition.clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    if contains_entities_query(field_strings) {
        format!(
            "{} \n\n {} ($representations: [_Any!]!) {{ {} }} }}",
            fragments_to_include, operation_type, selection_set
        )
    } else {
        let arguments = if !field.arguments.is_empty() {
            format!("({})", stringify_arguments(&field.arguments))
        } else {
            String::new()
        };
        format!(
            "{} \n\n {} {{ {}{} {{ {} }} }}",
            fragments_to_include, operation_type, field.field, arguments, selection_set
        )
    }
}
