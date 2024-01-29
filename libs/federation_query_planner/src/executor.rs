use std::sync::{Arc, RwLock};

use crate::{query_planner::QueryStep, user_query::FieldNode};

use super::supergraph::Supergraph;
use anyhow::Result;
use async_graphql::{dynamic::*, Value};
use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeValue;

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
pub struct QueryResponse {
  pub data: Option<SerdeValue>,
  pub errors: Option<Vec<SerdeValue>>,
  pub extensions: Option<SerdeValue>,
}

pub fn dynamically_build_schema_from_supergraph(supergraph: &Supergraph) -> Schema {
  let mut schema_builder = Schema::build("Query", None, None);

  for (type_name, graphql_type) in &supergraph.types {
    let mut obj = Object::new(type_name);

    for (field_name, gql) in &graphql_type.fields {
      let is_non_null = gql.field_type.ends_with('!');
      let base_type_name = gql.field_type.trim_matches(['[', ']', '!']);
      let mut base_type_ref = TypeRef::named(base_type_name);

      if is_non_null {
        base_type_ref = TypeRef::NonNull(Box::new(base_type_ref));
      }

      let field_type = if gql.field_type.starts_with('[') && gql.field_type.ends_with(']') {
        TypeRef::List(Box::new(base_type_ref))
      } else {
        base_type_ref
      };

      obj = obj.field(Field::new(field_name, field_type, move |_| {
        let future = Box::pin(async move {
          Ok(Some(FieldValue::from(Value::String(
            "Dynamic value".to_string(),
          ))))
        });
        FieldFuture::new(future)
      }));
    }

    // Register each custom type
    schema_builder = schema_builder.register(obj);
  }

  let mut query = Object::new("Query");
  if let Some(query_type) = supergraph.types.get("Query") {
    for (field_name, gql) in &query_type.fields {
      let field_type = if gql.field_type.starts_with('[') && gql.field_type.ends_with(']') {
        let inner_type = gql.field_type.trim_matches(['[', ']']);
        TypeRef::List(Box::new(TypeRef::named_nn(inner_type)))
      } else {
        TypeRef::named_nn(&gql.field_type)
      };

      query = query.field(Field::new(field_name, field_type, move |_| {
        let future = Box::pin(async move {
          Ok(Some(FieldValue::from(Value::String(
            "Dynamic value".to_string(),
          ))))
        });
        FieldFuture::new(future)
      }));
    }
  }

  schema_builder
    .register(query)
    .finish()
    .expect("Schema build failed")
}

// bc, we return the query step of the parent and the field of the `@key`, requests are sent twice when facing those two fields
// 1. key field
// 2. root field having a `query_step`
// need to optimize.
pub fn get_dep_field<'a>(
  field_path: &'a Vec<String>,
  fields_vec: Vec<Arc<RwLock<FieldNode>>>,
) -> Result<(QueryStep, Arc<RwLock<FieldNode>>)> {
  let mut current_vec = fields_vec;
  let mut query_step = None;
  let mut last_index = None;

  for key in field_path {
    let (found_index, found_query_step) = current_vec
      .iter()
      .enumerate()
      .find_map(|(index, node)| {
        if node.read().unwrap().field == *key {
          Some((index, node.read().unwrap().query_step.clone()))
        } else {
          None
        }
      })
      .ok_or("Field not found")
      .unwrap();

    // Assign last parent query step as the parent of that key field to fetch
    if let Some(step) = found_query_step {
      query_step = Some(step);
    }

    if key == field_path.last().unwrap() {
      last_index = Some(found_index);
      break;
    }

    let x = current_vec[found_index].read().unwrap().clone();
    current_vec = x.children.clone();
  }

  let final_index = last_index.ok_or("No final node found").unwrap();
  let final_node = current_vec[final_index].clone();

  Ok((query_step.unwrap(), final_node))
}

pub fn get_dep_field_value<'a>(
  field_path: &'a [String],
  fields_vec: Vec<Arc<RwLock<FieldNode>>>,
  entity_typename: String,
) -> Result<Option<SerdeValue>, String> {
  let mut current_vec = fields_vec;
  let mut last_response: Option<SerdeValue> = None;
  let mut is_entity_query = false;

  for key in field_path {
    let found_index = current_vec
      .iter()
      .enumerate()
      .find_map(|(index, node)| {
        if node.read().unwrap().field == *key {
          Some(index)
        } else {
          None
        }
      })
      .ok_or_else(|| format!("Field '{}' not found in path", key))?;

    {
      let node_read = current_vec[found_index].read().unwrap();
      if let Some(ref response) = node_read.response {
        if let Some(ref data) = response.data {
          last_response = Some(data.clone());
          is_entity_query = node_read
            .query_step
            .as_ref()
            .unwrap()
            .entity_query_needs_path
            .is_some();
        }
      }
    }

    let next_vec = current_vec[found_index].read().unwrap().clone();
    current_vec = next_vec.children;
  }

  if let Some(last_response_data) = last_response {
    if let SerdeValue::Object(obj) = last_response_data {
      let items = if is_entity_query {
        obj.get("_entities").and_then(|v| v.as_array())
      } else {
        obj
          .get(&field_path[field_path.len() - 2])
          .and_then(|v| v.as_array())
      };

      if let Some(items) = items {
        let id_values: Vec<SerdeValue> = items
          .iter()
          .filter_map(|item| {
            recursively_collect_values(
              item,
              &field_path[1..field_path.len() - 1],
              field_path.last().unwrap(),
              &entity_typename,
            )
          })
          .flatten()
          .collect();
        let mut final_output = serde_json::Map::new();
        final_output.insert(
          String::from("representations"),
          SerdeValue::Array(id_values),
        );
        return Ok(Some(SerdeValue::Object(final_output)));
      }
    }
  }

  Ok(None)
}

pub fn recursively_collect_values(
  value: &SerdeValue,
  remaining_path: &[String],
  target_key: &String,
  entity_typename: &String,
) -> Option<Vec<SerdeValue>> {
  match (remaining_path.first(), value) {
    (Some(next_key), SerdeValue::Object(obj)) => {
      // For objects, continue the recursion
      obj.get(next_key).and_then(|next_value| {
        recursively_collect_values(
          next_value,
          &remaining_path[1..],
          target_key,
          entity_typename,
        )
      })
    }
    (Some(_), SerdeValue::Array(array)) => {
      // ror arrays, apply recursion to each element
      let mut results = vec![];
      for item in array {
        if let Some(mut extracted_values) =
          recursively_collect_values(item, remaining_path, target_key, entity_typename)
        {
          results.append(&mut extracted_values);
        }
      }
      if !results.is_empty() {
        Some(results)
      } else {
        None
      }
    }
    (None, SerdeValue::Object(obj)) => {
      // collect the target value from the object
      obj.get(target_key).map(|v| {
        let mut map = serde_json::Map::new();
        map.insert(
          "__typename".to_string(),
          SerdeValue::String(entity_typename.clone()),
        );
        map.insert(target_key.clone(), v.clone());
        vec![SerdeValue::Object(map)]
      })
    }
    _ => None, // no match found or end of recursion
  }
}
