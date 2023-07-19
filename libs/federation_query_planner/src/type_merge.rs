use serde_json::{json, Map, Value};

use crate::{
    executor::QueryResponse,
    user_query::{FieldNode, UserQuery},
};

// Helper function to correctly nest the child value into the result.
fn nest_value(result: &mut Value, path: Vec<&str>, child_value: Value) {
    // Start from the root of the result which is a Value
    let mut current = result;

    // Iterate through the path except for the last element
    for key in path.iter().take(path.len() - 1) {
        let key_str = (*key).to_string(); // Convert &str to String

        // Navigate through or create the nested objects as needed
        let entry = current
            .as_object_mut()
            .expect("Should be an object")
            .entry(key_str)
            .or_insert_with(|| Value::Object(Map::new()));

        current = entry; // Move our reference down to this level
    }

    // Insert the final value at the end of the path
    if let Some(last_key) = path.last() {
        if let Some(obj) = current.as_object_mut() {
            obj.insert(last_key.to_string(), child_value); // Use the last element from the path as the key
        }
    }
}

pub fn construct_user_response(
    user_query: UserQuery,
    responses: Vec<Vec<((String, String), QueryResponse)>>,
) -> String {
    let mut response_data = Value::Object(Map::new()); // Start with a Value::Object instead of a raw Map

    for field in &user_query.fields {
        // This needs to recursively construct the response with nesting
        construct_field_response(field, &responses, &mut response_data, Vec::new());
    }

    json!({ "data": response_data }).to_string()
}

fn construct_field_response(
    field: &FieldNode,
    responses: &Vec<Vec<((String, String), QueryResponse)>>,
    result: &mut Value, // Change type to &mut Value
    path: Vec<&str>,
) {
    if field.should_be_cleaned {
        return;
    }

    let mut current_path = path.clone();
    let field_name = field.alias.as_ref().unwrap_or(&field.field);
    current_path.push(field_name);

    if let Some(relevant_queries) = &field.relevant_sub_queries {
        for (source, sub_query) in relevant_queries {
            if let Some(sub_response) = find_response(responses, source, sub_query) {
                if let Some(sub_response_data) = &sub_response.data {
                    // Now, instead of inserting directly, we need to nest the value
                    nest_value(result, current_path.clone(), sub_response_data.clone());
                }
            }
        }
    }

    // Recursively construct responses for nested fields
    for child_field in &field.children {
        construct_field_response(child_field, responses, result, current_path.clone());
    }
}

fn find_response<'a>(
    responses: &'a Vec<Vec<((String, String), QueryResponse)>>,
    source: &'a str,
    sub_query: &'a str,
) -> Option<&'a QueryResponse> {
    for response_group in responses {
        for ((response_source, response_query), response) in response_group {
            if response_source == source && response_query.ends_with(sub_query) {
                // The endswith check is a simplification. In practice, you might need a more robust comparison
                return Some(response);
            }
        }
    }
    None
}
