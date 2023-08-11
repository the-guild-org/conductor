use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::{
    query_planner::QueryResponse,
    user_query::{FieldNode, UserQuery},
};

pub fn merge_responses(
    final_response: &mut HashMap<String, Value>,
    query: &FieldNode,
    responses: &[QueryResponse],
) {
    for field in &query.children {
        for response in responses {
            if let Some(data) = &response.data {
                let merged_data = merge_field(field, data);
                if let Value::Object(merged_obj) = &merged_data {
                    for (k, v) in merged_obj.iter() {
                        final_response
                            .entry(k.clone())
                            .or_insert_with(Value::default)
                            .as_object_mut()
                            .unwrap()
                            .extend(v.as_object().unwrap().clone());
                    }
                }
            }
        }
    }
}

fn merge_field(field: &FieldNode, data: &Value) -> Value {
    match data {
        Value::Array(arr) => {
            let merged_arr: Vec<Value> = arr
                .iter()
                .map(|entry| {
                    let mut merged_entry = Map::new();
                    for child in &field.children {
                        if let Some(child_data) = entry.get(&child.field) {
                            let merged_data = merge_field(child, child_data);
                            merged_entry.insert(child.field.clone(), merged_data);
                        }
                    }
                    Value::Object(merged_entry)
                })
                .collect();

            return Value::Array(merged_arr);
        }
        Value::Object(obj) => {
            let mut merged_obj = Map::new();
            if let Some(existing_data) = obj.get(&field.field) {
                let field_data = merge_field(field, existing_data);
                merged_obj.insert(field.field.clone(), field_data);
            } else {
                for child in &field.children {
                    if let Some(child_data) = obj.get(&child.field) {
                        let merged_data = merge_field(child, child_data);
                        if let Value::Object(merged_data_obj) = merged_data {
                            for (k, v) in merged_data_obj {
                                merged_obj.insert(k, v);
                            }
                        } else {
                            merged_obj.insert(child.field.clone(), merged_data);
                        }
                    }
                }
            }
            return Value::Object(merged_obj);
        }
        _ => return data.clone(),
    }
}
