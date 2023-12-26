use conductor_config::ConductorConfig;
use schemars::gen::SchemaSettings;
use schemars::schema::SchemaObject;
use schemars::visit::{visit_schema_object, Visitor};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct MyVisitor;

impl Visitor for MyVisitor {
  fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
    let metadata = schema.metadata();

    for example in metadata.examples.iter_mut() {
      if let Value::Object(object) = example {
        match object.remove("$wrapper") {
          Some(Value::Object(mut wrapper)) => match wrapper.remove("plugin") {
            Some(Value::Object(mut plugin_ref)) => {
              if let Some(plugin_name) = plugin_ref.remove("name") {
                if let Some(plugin_name) = plugin_name.as_str() {
                  let metadata: Option<Value> = object.remove("$metadata");
                  let new_structure = json!({
                      "$metadata": metadata,
                      "type": plugin_name,
                      "enabled": true,
                      "config": object
                  });

                  *example = new_structure;
                }
              }
            }
            _ => {}
          },
          _ => {}
        }
      }
    }

    visit_schema_object(self, schema);
  }
}

pub fn main() {
  println!("⚙️ Generating JSON schema for Conductor config file...");
  // Please keep this 2019/09, see https://github.com/GREsau/schemars/issues/42#issuecomment-642603632
  // Website documentation generator depends on this.
  let schema = SchemaSettings::draft2019_09()
    .with_visitor(MyVisitor {})
    .into_generator()
    .into_root_schema_for::<ConductorConfig>();
  let as_string = serde_json::to_string_pretty(&schema).unwrap();
  println!("✏️ Writing to: libs/config/conductor.schema.json");
  std::fs::write("libs/config/conductor.schema.json", as_string).unwrap();
  println!("✅ Done");
}
