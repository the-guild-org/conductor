use conductor_config::ConductorConfig;
use schemars::gen::SchemaSettings;

pub fn main() {
    println!("⚙️ Generating JSON schema for Conductor config file...");
    // Please keep this 2019/09, see https://github.com/GREsau/schemars/issues/42#issuecomment-642603632
    // Website documentation generator depends on this.
    let schema = SchemaSettings::draft2019_09()
        .into_generator()
        .into_root_schema_for::<ConductorConfig>();
    let as_string = serde_json::to_string_pretty(&schema).unwrap();
    println!("✏️ Writing to: libs/config/conductor.schema.json");
    std::fs::write("libs/config/conductor.schema.json", as_string).unwrap();
    println!("✅ Done");
}
