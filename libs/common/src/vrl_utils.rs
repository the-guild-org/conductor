use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::serde_utils::LocalFileReference;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "from")]
pub enum VrlConfigReference {
  #[serde(rename = "inline")]
  #[schemars(title = "inline")]
  /// Inline string for a VRL code snippet. The string is parsed and executed as a VRL plugin.
  Inline { content: String },
  #[serde(rename = "file")]
  #[schemars(title = "file")]
  /// File reference to a VRL file. The file is loaded and executed as a VRL plugin.
  File { path: LocalFileReference },
}

impl VrlConfigReference {
  pub fn contents(&self) -> &String {
    match self {
      VrlConfigReference::Inline { content } => content,
      VrlConfigReference::File { path } => &path.contents,
    }
  }
}
