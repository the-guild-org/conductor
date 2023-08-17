use serde_json::Value;

use crate::{
    executor::QueryResponse,
    user_query::{FieldNode, UserQuery},
};
fn merge_into_shape(shape: &mut Value, data: &Value) {
    match (shape, data) {
        (Value::Object(shape_map), Value::Object(data_map)) => {
            for (key, value) in shape_map.iter_mut() {
                if let Some(data_value) = data_map.get(key) {
                    merge_into_shape(value, data_value);
                }
            }
        }
        (shape_val, data_val) => {
            *shape_val = data_val.clone();
        }
        _ => {}
    }
}

fn merge_recursive(
    remaining_fields: &mut Vec<FieldNode>,
    data: &serde_json::Value,
    final_data: &mut serde_json::Value,
) {
    if let Value::Object(data_map) = data {
        for (key, value) in data_map.iter() {
            // Remove a field from the list of fields to be processed after it's merged
            let position = remaining_fields.iter().position(|x| x.field == *key);
            if let Some(pos) = position {
                let mut field = remaining_fields.remove(pos);
                if final_data[key].is_null() {
                    final_data[key] = value.clone();
                } else {
                    merge_into_shape(&mut final_data[key], value);
                }
                merge_recursive(&mut field.children, value, &mut final_data[key]);
            }
        }
    }
}

pub fn merge_responses(user_query: &UserQuery, responses: Vec<QueryResponse>) -> QueryResponse {
    let mut result_data = Value::Object(serde_json::map::Map::new());

    let mut fields_to_process = user_query.fields;

    for response in &responses {
        if let Some(data) = &response.data {
            if let Value::Object(ref mut map) = &mut result_data {
                merge_recursive(&mut fields_to_process, data, map);
            }
        }
    }

    QueryResponse {
        data: Some(Value::Object(result_data)),
        errors: None, // Errors can be merged similarly if necessary
    }
}
