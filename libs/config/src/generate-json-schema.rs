use conductor_config::ConductorConfig;
use schemars::{
    gen::SchemaSettings,
    schema::SchemaObject,
    visit::{visit_schema_object, Visitor},
};

#[derive(Debug, Clone)]
pub struct PatchDescriptionVisitor;

impl Visitor for PatchDescriptionVisitor {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        if let Some(desc) = schema
            .metadata
            .as_mut()
            .and_then(|m| m.description.as_mut())
        {
            *desc = desc.replace("\n\n", "\n");
        }

        visit_schema_object(self, schema);
    }
}

pub fn main() {
    println!("⚙️ Generating JSON schema for Conductor config file...");
    // Please keep this 2019/09, see https://github.com/GREsau/schemars/issues/42#issuecomment-642603632
    // Website documentation generator depends on this.
    let schema = SchemaSettings::draft2019_09()
        .with_visitor(PatchDescriptionVisitor {})
        .into_generator()
        .into_root_schema_for::<ConductorConfig>();
    let as_string = serde_json::to_string_pretty(&schema).unwrap();
    println!("✏️ Writing to: libs/config/conductor.schema.json");
    std::fs::write("libs/config/conductor.schema.json", as_string).unwrap();
    println!("✅ Done");
}
