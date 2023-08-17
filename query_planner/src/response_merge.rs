use crate::{
    executor::QueryResponse,
    supergraph::Supergraph,
    user_query::{FieldNode, UserQuery},
};

type Value = serde_json::Value;

pub fn merge_responses(
    responses: Vec<QueryResponse>,
    user_query: &UserQuery,
    supergraph: &Supergraph,
) -> QueryResponse {
    let mut merged_data = serde_json::Map::new();

    for response in &responses {
        if let Some(data) = &response.data {
            if let Value::Array(data_arr) = data {
                for data_obj in data_arr.iter() {
                    // Process each data object within the array
                    if let Value::Object(actual_obj) = data_obj {
                        for (field, value) in actual_obj.iter() {
                            println!("Processing field: {}", field);
                            match merged_data.get(field) {
                                Some(existing_value) => {
                                    let merged_field_type = get_field_type(user_query, field);
                                    if let Some(merged_field_type) = merged_field_type {
                                        if let Some(type_data) =
                                            supergraph.types.get(&merged_field_type)
                                        {
                                            let key_fields = &type_data.key_fields;
                                            let merged_value =
                                                deep_merge(existing_value, value, key_fields);
                                            merged_data.insert(field.clone(), merged_value);
                                        }
                                    }
                                }
                                None => {
                                    merged_data.insert(field.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    QueryResponse {
        data: Some(Value::Object(merged_data)), // Convert Map to Value
        errors: None,                           // You might also want to merge errors if needed
    }
}

fn deep_merge(existing: &Value, new: &Value, key_fields: &Vec<String>) -> Value {
    match (existing, new) {
        (Value::Array(existing_arr), Value::Array(new_arr)) => {
            Value::Array(merge_array(existing_arr, new_arr, key_fields))
        }
        (Value::Object(existing_obj), Value::Object(new_obj)) => {
            let mut merged = existing_obj.clone();
            for (key, value) in new_obj.iter() {
                match merged.get(key) {
                    Some(existing_value) => {
                        let merged_value = deep_merge(existing_value, value, key_fields);
                        merged.insert(key.clone(), merged_value);
                    }
                    None => {
                        merged.insert(key.clone(), value.clone());
                    }
                }
            }
            Value::Object(merged)
        }
        (_, _) => new.clone(),
    }
}

fn merge_array(
    existing_array: &Vec<Value>,
    new_array: &Vec<Value>,
    key_fields: &Vec<String>,
) -> Vec<Value> {
    let mut result = existing_array.clone();

    for new_item in new_array.iter() {
        let mut found = false;

        for existing_item in &mut result {
            let all_keys_match = key_fields.iter().all(|key_field| {
                match (existing_item.get(key_field), new_item.get(key_field)) {
                    (Some(ev), Some(nv)) => ev == nv,
                    _ => false,
                }
            });

            if all_keys_match {
                *existing_item = deep_merge(existing_item, new_item, key_fields);
                found = true;
                break;
            }
        }

        if !found {
            result.push(new_item.clone());
        }
    }

    result
}

fn get_field_type(user_query: &UserQuery, field: &str) -> Option<String> {
    for node in &user_query.fields {
        if let Some(ty) = find_field_type(node, field) {
            return Some(ty);
        }
    }
    None
}

fn find_field_type(node: &FieldNode, field: &str) -> Option<String> {
    if node.field == field {
        return Some(node.field.clone());
    }

    for child in &node.children {
        if let Some(ty) = find_field_type(child, field) {
            return Some(ty);
        }
    }

    None
}
