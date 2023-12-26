use serde::de::Error as DeError;
use serde_json::{from_str, Error as SerdeError, Map, Value};

pub fn parse_and_extract_json_map_value(value: &str) -> Result<Map<String, Value>, SerdeError> {
  let parsed_json = from_str::<Value>(value);

  match parsed_json {
    Ok(Value::Object(v)) => Ok(v),
    Ok(_) => Err(DeError::custom("expected object")),
    Err(e) => Err(e),
  }
}
