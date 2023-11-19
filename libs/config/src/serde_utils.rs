use std::{fmt, path::Path};

use schemars::JsonSchema;
use serde::{de::Visitor, Deserialize};
use std::fs::read_to_string;
use tracing::debug;

struct LocalFileReferenceVisitor {}

impl<'de> Visitor<'de> for LocalFileReferenceVisitor {
    type Value = LocalFileReference;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expected a valid local file path")
    }

    fn visit_str<E>(self, file_path: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        debug!("loading local file reference from path {:?}", file_path);
        let contents = read_to_string(Path::new(file_path)).expect("Failed to read file");

        Ok(LocalFileReference {
            path: file_path.to_string(),
            contents,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LocalFileReference {
    pub path: String,
    pub contents: String,
}

impl JsonSchema for LocalFileReference {
    fn schema_name() -> String {
        "LocalFileReference".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            format: Some("path".to_string()),
            ..Default::default()
        })
    }
}

impl<'de> Deserialize<'de> for LocalFileReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(LocalFileReferenceVisitor {})
    }
}
