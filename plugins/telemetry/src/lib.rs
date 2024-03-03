mod config;
mod plugin;

#[cfg(target_arch = "wasm32")]
pub mod wasm_reporter;

#[cfg(not(target_arch = "wasm32"))]
pub mod reporter;

pub use config::TelemetryPluginConfig as Config;
pub use config::TelemetryTarget as Target;
pub use plugin::TelemetryPlugin as Plugin;
