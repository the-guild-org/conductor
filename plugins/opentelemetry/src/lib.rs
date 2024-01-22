mod config;
mod plugin;
#[cfg(target_arch = "wasm32")]
mod wasm_runtime;

pub use config::OpenTelemetryTarget as Target;
pub use config::TelemetryPluginConfig as Config;
pub use plugin::TelemetryPlugin as Plugin;
