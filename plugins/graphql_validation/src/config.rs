use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
// #[schemars(example = "graphql_validation_example_1")]
pub struct GraphQLValidationPluginConfig {}
