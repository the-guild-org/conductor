mod config;
mod plugin;

pub use config::OpenTelemetryTarget as Target;
pub use config::TelemetryPluginConfig as Config;
pub use plugin::TelemetryPlugin as Plugin;
