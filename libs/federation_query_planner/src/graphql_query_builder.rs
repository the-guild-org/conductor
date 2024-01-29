use crate::user_query::{FieldNode,  QueryArgument};

pub fn contains_entities_query(field_strings: &str) -> bool {
  field_strings.contains("_entities(representations: $representations)")
}

pub fn generate_entities_query(typename: &str, key_fields: &str, selection_set: &str) -> String {
  assert!(
    !typename.is_empty(),
    "Typename of the parent field must not be empty when generating an _entity query!"
  );
  format!(
    "_entities(representations: $representations) {{ ... on {} {{ {} {} }} }}",
    typename, key_fields, selection_set
  )
}

pub fn generate_query_for_field(
  operation_type: String,
  sub_query: String,
  // arguments: Vec<QueryArgument>,
  fragments: &str,
) -> String {
  if contains_entities_query(&sub_query) {
    // TODO: clean this up
    format!(
      "{} ($representations: [_Any!]!) {{ {} }}",
      if operation_type.is_empty() {
        "query"
      } else {
        &operation_type
      },
      sub_query
    )
  } else {
    // let arguments = if !arguments.is_empty() {
    //     format!("({})", stringify_arguments(&arguments))
    // } else {
    //     String::new()
    // };
    // TODO: add arguments
    format!("{} {{ {} }} {fragments}", operation_type, sub_query)
  }
}
pub fn parse_into_selection_set(field: &FieldNode) -> String {
  if field.arguments.is_empty() {
    field.field.to_string()
  } else {
    format!(
      "{}({})",
      field.field.to_string(),
      field
        .arguments
        .iter()
        .map(|QueryArgument { name, value }| format!("{}: {}", name, value))
        .collect::<Vec<String>>()
        .join(",")
    )
  }
}