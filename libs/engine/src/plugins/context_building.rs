use conductor_config::plugins::ContextBuildingPluginConfig;

use crate::request_execution_context::RequestExecutionContext;

use super::core::Plugin;

pub struct ContextBuildingPlugin(pub ContextBuildingPluginConfig);

#[async_trait::async_trait]
impl Plugin for ContextBuildingPlugin {
    // async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {}
}
