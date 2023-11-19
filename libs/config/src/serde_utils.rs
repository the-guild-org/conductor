use std::{cell::RefCell, fmt, path::PathBuf};

use schemars::JsonSchema;
use serde::{de::Visitor, Deserialize};
use std::fs::read_to_string;
use tracing::debug;

use crate::BASE_PATH;

pub struct LocalFileReferenceVisitor {
    base_path: RefCell<PathBuf>,
}

impl LocalFileReferenceVisitor {
    pub fn new(base_path: RefCell<PathBuf>) -> Self {
        Self { base_path }
    }
}

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

        let base_path = self.base_path.into_inner();
        let full_path = base_path.join(file_path);

        let contents = read_to_string(&full_path).map_err(|_| E::custom("Failed to read file"))?;

        Ok(LocalFileReference {
            path: full_path.to_string_lossy().into_owned(),
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
        let base_path = BASE_PATH.with(|e| e.clone());
        let visitor = LocalFileReferenceVisitor::new(base_path);
        deserializer.deserialize_str(visitor)
    }
}
