use std::pin::Pin;

use super::supergraph::Supergraph;
use async_graphql::{dynamic::*, Error, Value};
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeValue;

pub fn find_objects_matching_criteria(
  json: &SerdeValue,
  typename: &str,
  field: &str,
) -> Vec<SerdeValue> {
  let mut matching_objects = Vec::new();

  match json {
    SerdeValue::Object(map) => {
      if let (Some(typename_value), Some(field_value)) = (
        map.get("__typename").and_then(|v| v.as_str()),
        map.get(field),
      ) {
        if typename_value == typename {
          let mut result = serde_json::Map::new();
          result.insert(
            "__typename".to_string(),
            SerdeValue::String(typename.to_string()),
          );
          result.insert(field.to_string(), field_value.clone());
          matching_objects.push(SerdeValue::Object(result));
        }
      }
      for (_, value) in map {
        matching_objects.extend(find_objects_matching_criteria(value, typename, field));
      }
    }
    SerdeValue::Array(arr) => {
      for element in arr {
        matching_objects.extend(find_objects_matching_criteria(element, typename, field));
      }
    }
    _ => {}
  }

  matching_objects
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct QueryResponse {
  pub data: Option<SerdeValue>,
  pub errors: Option<Vec<SerdeValue>>,
  pub extensions: Option<SerdeValue>,
}

pub fn dynamically_build_schema_from_supergraph(supergraph: &Supergraph) -> Schema {
  let mut query = Object::new("Query");

  // Dynamically create object types and fields
  for (type_name, graphql_type) in &supergraph.types {
    let mut obj = Object::new(type_name);

    for field_name in graphql_type.fields.keys() {
      let field_type = TypeRef::named_nn(TypeRef::STRING); // Adjust based on `field.field_type`
      obj = obj.field(Field::new(field_name, field_type, move |_| {
        let future: Pin<Box<dyn Future<Output = Result<Option<FieldValue>, Error>> + Send>> =
          Box::pin(async move {
            Ok(Some(FieldValue::from(Value::String(
              "Dynamic value".to_string(),
            ))))
          });
        FieldFuture::new(future)
      }));
    }

    // Adjust the creation of Object TypeRef
    // Placeholder logic - replace with the correct object creation
    let obj_type_ref = TypeRef::named(TypeRef::STRING); // This needs to be correctly set
    query = query.field(Field::new(type_name, obj_type_ref, move |_| {
      let future: Pin<Box<dyn Future<Output = Result<Option<FieldValue>, Error>> + Send>> =
        Box::pin(async move { Ok(Some(FieldValue::from(Value::Object(Default::default())))) });
      FieldFuture::new(future)
    }));
  }

  // Construct and return the schema

  Schema::build("Query", None, None)
    .register(query)
    .finish()
    .expect("Introspection schema build failed")
}
