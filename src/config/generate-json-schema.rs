use conductor::config::ConductorConfig;
use schemars::schema_for;

pub fn main() {
    println!("⚙️ Generating JSON schema for Conductor config file...");
    let schema = schema_for!(ConductorConfig);
    let as_string = serde_json::to_string_pretty(&schema).unwrap();
    println!("✏️ Writing to: src/config/conductor.schema.json");
    std::fs::write("src/config/conductor.schema.json", as_string).unwrap();
    println!("✅ Done");
}
