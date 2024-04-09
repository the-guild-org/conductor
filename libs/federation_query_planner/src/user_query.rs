use anyhow::{anyhow, Result};
use graphql_parser::query::{Definition, Document, Field, OperationDefinition, Selection};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Map, Value};
use std::{
  collections::HashMap,
  fmt::{Display, Formatter},
  sync::{Arc, RwLock},
};

use crate::{
  constants::CONDUCTOR_INTERNAL_SERVICE_RESOLVER,
  executor::QueryResponse,
  query_planner::QueryStep,
  supergraph::{GraphQLType, Supergraph},
  unwrap_graphql_type,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldNode {
  pub field: String,
  pub alias: Option<String>,
  pub arguments: Vec<QueryArgument>,
  #[serde(serialize_with = "serialize_field_node")]
  #[serde(deserialize_with = "deserialize_field_node")]
  pub children: Vec<Arc<RwLock<FieldNode>>>,
  pub sources: Vec<String>,
  pub type_name: Option<String>,
  pub is_list: bool,
  pub parent_type_name: Option<String>,
  pub key_fields: Option<String>,
  pub owner: Option<String>,
  pub requires: Option<String>,
  pub should_be_cleaned: bool,
  pub is_introspection: bool,
  pub query_step: Option<QueryStep>,
  pub response: Option<QueryResponse>,
  #[serde(serialize_with = "serialize_depends_on_path")]
  #[serde(deserialize_with = "deserialize_depends_on_path")]
  pub depends_on_path: Arc<RwLock<Option<Vec<Vec<String>>>>>,
  pub key_field_path: Option<Vec<String>>,
  pub str_path: Vec<String>,
  pub order_index: usize,
}

fn serialize_field_node<S>(val: &Vec<Arc<RwLock<FieldNode>>>, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let field_nodes: Vec<_> = val.iter().map(|e| e.read().unwrap().clone()).collect();
  Serialize::serialize(&field_nodes, s)
}

fn deserialize_field_node<'de, D>(d: D) -> Result<Vec<Arc<RwLock<FieldNode>>>, D::Error>
where
  D: Deserializer<'de>,
{
  let field_nodes: Vec<FieldNode> = Deserialize::deserialize(d)?;

  std::result::Result::Ok(
    field_nodes
      .into_iter()
      .map(|node| Arc::new(RwLock::new(node)))
      .collect(),
  )
}

fn serialize_depends_on_path<S>(
  depends_on_path: &Arc<RwLock<Option<Vec<Vec<String>>>>>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let lock = depends_on_path.read().unwrap();
  serializer.serialize_some(&*lock)
}

fn deserialize_depends_on_path<'de, D>(
  deserializer: D,
) -> Result<Arc<RwLock<Option<Vec<Vec<String>>>>>, D::Error>
where
  D: Deserializer<'de>,
{
  let opt_vec_vec_str = Option::<Vec<Vec<String>>>::deserialize(deserializer)?;
  std::result::Result::Ok(Arc::new(RwLock::new(opt_vec_vec_str)))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationType {
  Query,
  Mutation,
  Subscription,
}
impl Display for OperationType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      // can be `query`, but why not save some space
      OperationType::Query => write!(f, ""),
      OperationType::Mutation => write!(f, "mutation"),
      OperationType::Subscription => write!(f, "subscription"),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct QueryArgument {
  pub name: String,
  pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryDefinedArgument {
  pub name: String,
  pub default_value: Option<String>,
}

type QueryDefinedArguments = Vec<QueryDefinedArgument>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternalGraphQLFragment {
  pub str_definition: String,
  #[serde(serialize_with = "serialize_field_node")]
  #[serde(deserialize_with = "deserialize_field_node")]
  pub fields: Vec<Arc<RwLock<FieldNode>>>,
}

pub struct GraphQLFragment<'a> {
  pub type_name: &'a str,
  pub fragment: &'a InternalGraphQLFragment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fragments {
  items: HashMap<String, InternalGraphQLFragment>,
}

impl Fragments {
  pub fn get_fragment<'a>(&'a self, name: &'a str) -> Result<GraphQLFragment> {
    match self.items.iter().find(|(key, _)| key == &name) {
      Some((type_name, graphql_fragment)) => Ok(GraphQLFragment {
        type_name,
        fragment: graphql_fragment,
      }),
      None => Err(anyhow!(format!(
        "fragment named \"{name}\" is not defined in your query!",
      ))),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery {
  pub operation_type: OperationType,
  pub arguments: Vec<QueryDefinedArgument>,
  #[serde(serialize_with = "serialize_field_node")]
  #[serde(deserialize_with = "deserialize_field_node")]
  pub fields: Vec<Arc<RwLock<FieldNode>>>,
  pub fragments: Fragments,
}

impl UserQuery {
  pub fn read_field_node(node: Arc<RwLock<FieldNode>>) -> Result<FieldNode, anyhow::Error> {
    let field_node = node.read().map_err(|e| anyhow!("Read lock error: {}", e))?;
    Ok(field_node.clone())
  }

  pub fn write_field_node<F>(
    node: Arc<RwLock<FieldNode>>,
    modify_fn: F,
  ) -> Result<(), anyhow::Error>
  where
    F: FnOnce(&mut FieldNode),
  {
    let mut field_node = node
      .write()
      .map_err(|e| anyhow!("Write lock error: {}", e))?;
    modify_fn(&mut *field_node);
    Ok(())
  }

  pub fn to_json_result(&self, fields: &[Arc<RwLock<FieldNode>>]) -> Value {
    let mut result = Map::new();

    for field_node in fields {
      let field_node = field_node.read().unwrap();
      if let Some(response) = &field_node.response {
        if field_node.is_introspection {
          let (key, json_value) = field_node
            .response
            .clone()
            .unwrap()
            .data
            .unwrap()
            .as_object()
            .and_then(|obj| obj.iter().next())
            .map(|(k, v)| (k.clone(), v.clone()))
            .unwrap();
          result.insert(key, json_value);
        } else {
          let mut field_data = response
            .data
            .clone()
            .unwrap_or_default()
            .get(&field_node.field)
            .unwrap()
            .clone();

          self.merge_children(&mut field_data, &field_node.children);
          result.insert(field_node.field.clone(), field_data);
        }
      }
    }

    Value::Object(result)
  }

  fn merge_children(&self, parent_data: &mut Value, children: &[Arc<RwLock<FieldNode>>]) {
    for child_node_arc in children {
      let child_node = child_node_arc.read().unwrap();
      if let Some(response) = &child_node.response {
        let mut child_data = response.data.clone().unwrap_or_default();
        self.merge_children(&mut child_data, &child_node.children);

        if let Some(query_step) = &child_node.query_step {
          if let Some(entity_query_needs_path) = &query_step.entity_query_needs_path {
            self.merge_entity_query_response(
              parent_data,
              &child_node,
              &mut child_data,
              entity_query_needs_path,
            );
          }
        } else {
          self
            .merge_field_data(
              parent_data,
              &child_node.alias.as_ref().unwrap_or(&child_node.field),
              child_data,
            )
            .unwrap();
        }
      }
    }
  }

  fn merge_entity_query_response(
    &self,
    parent_data: &mut Value,
    child_node: &FieldNode,
    child_data: &mut Value,
    entity_query_needs_path: &[Vec<String>],
  ) {
    if let Value::Array(parent_array) = parent_data {
      if let Value::Object(child_obj) = child_data {
        if let Some(entities) = child_obj.get_mut("_entities") {
          if let Value::Array(entity_array) = entities {
            for parent_item in parent_array.iter_mut() {
              if let Value::Object(parent_item_object) = parent_item {
                let parent_key_values = self
                  // exclude first one, when it's the root field
                  .get_key_values(&parent_item_object, &entity_query_needs_path[1..])
                  .unwrap();

                for entity_item in entity_array.iter_mut() {
                  if let Value::Object(entity_object) = entity_item {
                    let entity_key_values = self
                      .get_key_values(entity_object, entity_query_needs_path)
                      .unwrap_or_default();
                    if entity_key_values == parent_key_values {
                      self
                        .merge_field_data(parent_item, &child_node.field, entity_item.take())
                        .unwrap();
                      break;
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  fn get_key_values(
    &self,

    object: &Map<String, Value>,
    paths: &[Vec<String>],
  ) -> Result<Vec<Value>> {
    let mut key_values = Vec::new();
    for path in paths {
      let mut value = Value::Object(object.clone());
      for key in path {
        value = match value {
          Value::Object(obj) => obj
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Key not found: {}", key))?,
          _ => return Err(anyhow!("Expected an object, found: {:?}", value)),
        };
      }
      key_values.push(value);
    }
    Ok(key_values)
  }

  fn merge_field_data(
    &self,
    parent_data: &mut Value,
    field_name: &str,
    child_data: Value,
  ) -> Result<()> {
    if let Value::Object(parent_object) = parent_data {
      parent_object.insert(
        field_name.to_string(),
        child_data.get(field_name).unwrap().clone(),
      );
    }
    Ok(())
  }

  pub fn populate_supergraph_metadata(&mut self, supergraph: &Supergraph) -> Result<()> {
    let query_type = supergraph.get_gql_type("Query", "")?;

    populate(
      &mut self.fields,
      query_type,
      supergraph,
      Some("Query"),
      &self.fragments,
      &mut vec![],
    )
  }
}
// this is outside, because we need to reference `self` immutably and mutably in the same context.
fn populate(
  fields: &mut Vec<Arc<RwLock<FieldNode>>>,
  graphql_type: &GraphQLType,
  supergraph: &Supergraph,
  parent_type_name: Option<&str>,
  fragments: &Fragments,
  parent_path: &mut Vec<String>,
) -> Result<()> {
  let mut idx = 0;

  while idx < fields.len() {
    let x = fields.get(idx).unwrap().clone();
    let field = &mut *x.write().unwrap();

    // handle fragment spreads
    if let Some(fragment_name) = field
      .field
      .strip_prefix("...")
      .and_then(|s| s.split("...").next())
    {
      let GraphQLFragment {
        type_name,
        fragment,
      } = fragments.get_fragment(fragment_name)?;
      let mut fragment_fields = fragment.fields.clone();
      let next_gql_type = supergraph.get_gql_type(type_name, "Fragment")?;

      // populate for the fragments
      populate(
        &mut fragment_fields,
        next_gql_type,
        supergraph,
        Some(type_name),
        fragments,
        parent_path,
      )?;

      // replace the Fragment usage with its fields
      fields.remove(idx);
      fields.splice(idx..idx, fragment_fields.drain(..));
      continue;
    } else {
      idx += 1;

      if field.is_introspection {
        // In the case of detecting an introspection query
        // Handle introspection queries internally
        field.field = format!(
          "{}|{}",
          field.field,
          fragments
            .items
            .values()
            .map(|a| a.str_definition.clone())
            .collect::<Vec<String>>()
            .join(" ")
        );
        field.sources = vec![String::from(CONDUCTOR_INTERNAL_SERVICE_RESOLVER)];
      } else if field.field == "__typename" {
        // Special handling for __typename field:
        // Just continue to the next iteration,
        // as __typename is a meta field that doesn't need further resolution.
        continue;
      } else {
        let gql_field =
          graphql_type.get_field(&field.field, parent_type_name.unwrap_or("Query"))?;

        // println!("{:#?}", gql_field);
        let child_type_name = unwrap_graphql_type(&gql_field.field_type);

        field.parent_type_name = parent_type_name.map(String::from);

        field.sources = gql_field.sources.clone();
        if graphql_type.owner.is_some()
          && field.sources.contains(graphql_type.owner.as_ref().unwrap())
        {
          field.owner = graphql_type.owner.clone();
        }

        field.type_name = Some(unwrap_graphql_type(gql_field.field_type.as_str()).to_string());
        field.is_list = gql_field.field_type.starts_with('[');
        field.order_index = idx - 1;
        // don't include the field itself as a key field
        if Some(&field.field.to_string()) != graphql_type.key_fields.as_ref() {
          field.key_fields = graphql_type.key_fields.clone();
        }

        if let Some(ref key_fields) = field.key_fields {
          // Build the path to the key field
          let mut key_path = parent_path.clone();
          key_path.push(key_fields.clone());
          field.key_field_path = Some(key_path);
        } else {
          field.key_field_path = None;
        }

        field.requires = gql_field.requires.clone();

        let mut x = parent_path.clone();
        x.push(field.field.to_string());

        field.str_path = x;

        if !field.children.is_empty() {
          // let new_field = FieldNode {
          //   field: String::from("__typename"),
          //   alias: None,
          //   arguments: vec![],
          //   children: vec![],
          //   sources: field.sources.clone(),
          //   type_name: None,
          //   is_list: false,
          //   parent_type_name: None,
          //   key_fields: None,
          //   owner: None,
          //   requires: None,
          //   should_be_cleaned: true, // clean it in the response merging phase
          //   is_introspection: false,
          //   query_step: None,
          //   response: None,
          //   depends_on_path: None,
          // };

          // field.children.push(new_field);

          let next_gql_type = supergraph.get_gql_type(child_type_name, "Object Type")?;

          if let Some(key_field) = &graphql_type.key_fields {
            if fields
              .iter()
              .find(|&e| {
                let x = match e.try_read() {
                  Ok(e) => e.clone(),
                  Err(_e) => field.clone(),
                };

                &x.field == key_field
              })
              .is_none()
            {
              fields.push(Arc::new(RwLock::new(FieldNode {
                field: key_field.to_string(),
                // TODO: should include children in case it's a selection set
                children: vec![],
                alias: None,
                arguments: vec![],
                parent_type_name: parent_type_name.map(|e| e.to_string()),
                sources: vec![],
                type_name: None,
                is_list: false,
                key_fields: None,
                key_field_path: None,
                owner: None,
                requires: None,
                should_be_cleaned: true,
                is_introspection: false,
                query_step: None,
                response: None,
                depends_on_path: Arc::new(RwLock::new(None)),
                str_path: vec![],
                order_index: 0,
              })))
            }
          }

          let mut parent_path2 = parent_path.clone();
          parent_path2.push(field.field.to_string());

          populate(
            &mut field.children,
            next_gql_type,
            supergraph,
            Some(unwrap_graphql_type(gql_field.field_type.as_str())),
            fragments,
            &mut parent_path2,
          )?;
        }
      }
    }
  }

  Ok(())
}

fn seek_root_fields_capacity(parsed_query: &Document<'_, String>) -> usize {
  parsed_query
    .definitions
    .iter()
    .find_map(|e| match e {
      Definition::Operation(val) => match val {
        OperationDefinition::Query(e) => Some(e.selection_set.items.len()),
        OperationDefinition::Mutation(e) => Some(e.selection_set.items.len()),
        OperationDefinition::Subscription(e) => Some(e.selection_set.items.len()),
        OperationDefinition::SelectionSet(e) => Some(e.items.len()),
      },
      _ => None,
    })
    .unwrap_or(0)
}

pub fn parse_user_query(parsed_query: Document<'static, String>) -> Result<UserQuery> {
  let mut user_query = UserQuery {
    operation_type: OperationType::Query,
    arguments: vec![],
    fields: Vec::with_capacity(seek_root_fields_capacity(&parsed_query)),
    fragments: Fragments {
      items: HashMap::new(),
    },
  };

  for definition in parsed_query.definitions {
    match definition {
      Definition::Operation(OperationDefinition::Query(q)) => {
        user_query.operation_type = OperationType::Query;

        user_query.arguments = q
          .variable_definitions
          .into_iter()
          .map(|e| QueryDefinedArgument {
            name: e.name,
            default_value: e.default_value.map(|e| e.to_string()),
          })
          .collect::<Vec<_>>();

        user_query.fields.extend(handle_selection_set(
          &user_query.arguments,
          q.selection_set,
        )?);
      }
      Definition::Operation(OperationDefinition::Mutation(m)) => {
        user_query.operation_type = OperationType::Mutation;

        user_query.arguments = m
          .variable_definitions
          .into_iter()
          .map(|e| QueryDefinedArgument {
            name: e.name,
            default_value: e.default_value.map(|e| e.to_string()),
          })
          .collect::<Vec<_>>();

        user_query.fields.extend(handle_selection_set(
          &user_query.arguments,
          m.selection_set,
        )?);
      }
      Definition::Operation(OperationDefinition::Subscription(s)) => {
        user_query.operation_type = OperationType::Subscription;

        user_query.arguments = s
          .variable_definitions
          .into_iter()
          .map(|e| QueryDefinedArgument {
            name: e.name,
            default_value: e.default_value.map(|e| e.to_string()),
          })
          .collect::<Vec<_>>();

        user_query.fields.extend(handle_selection_set(
          &user_query.arguments,
          s.selection_set,
        )?);
      }
      Definition::Operation(OperationDefinition::SelectionSet(e)) => {
        user_query.fields = handle_selection_set(&user_query.arguments, e)?;
      }
      Definition::Fragment(e) => {
        let fragments_fields =
          handle_selection_set(&user_query.arguments, e.selection_set.clone())?;

        for (index, field_node_arc) in fragments_fields.iter().enumerate() {
          let mut field_node = field_node_arc.write().unwrap();
          field_node.order_index = index;
        }

        user_query.fragments.items.insert(
          e.name.to_string(),
          InternalGraphQLFragment {
            str_definition: format!("{}", e),
            fields: fragments_fields,
          },
        );
      }
    }
  }

  Ok(user_query)
}

fn handle_selection_set(
  defined_arguments: &QueryDefinedArguments,
  selection_set: graphql_parser::query::SelectionSet<'_, String>,
) -> Result<Vec<Arc<RwLock<FieldNode>>>> {
  let mut fields = Vec::with_capacity(selection_set.items.len());

  for selection in selection_set.items {
    match selection {
      Selection::Field(Field {
        name,
        selection_set: field_selection_set,
        arguments,
        alias,
        ..
      }) => {
        let is_introspection = name.starts_with("__");

        let (name, children) = if is_introspection {
          (
            format!(
              "{name}{}",
              if !field_selection_set.items.is_empty() {
                field_selection_set.to_string()
              } else {
                String::new()
              }
            ),
            vec![],
          )
        } else {
          (
            name,
            handle_selection_set(defined_arguments, field_selection_set)?,
          )
        };

        let arguments = arguments
          .into_iter()
          .map(|(arg_name, value)| {
            let value = value.to_string();
            let value = if value.starts_with('$') {
              defined_arguments
                .iter()
                .find(|e| e.name == value[1..])
                .unwrap_or_else(|| panic!("Argument {} is used but was never defined!", value))
                .default_value
                .as_ref()
                .unwrap_or_else(|| panic!("No default value for {}!", value))
                .to_string()
            } else {
              value
            };

            QueryArgument {
              name: arg_name,
              value,
            }
          })
          .collect();

        let field_node = FieldNode {
          field: name,
          children,
          alias,
          arguments,
          parent_type_name: Some(String::from("Query")),
          sources: vec![],
          type_name: None,
          is_list: false,
          key_fields: None,
          key_field_path: None,
          owner: None,
          requires: None,
          should_be_cleaned: false,
          is_introspection,
          query_step: None,
          response: None,
          depends_on_path: Arc::new(RwLock::new(None)),
          str_path: vec![],
          order_index: 0,
        };

        //           if let Some(dependencies) = identify_dependencies(&name, &supergraph) {
        //   let field_dependencies = dependencies.iter().map(|dep_field| {
        //     FieldDependency {
        //       field_name: dep_field.to_string(),
        //       parent_type: // determine the parent type,
        //       path: // logic to determine the path,
        //     }
        //   }).collect();

        //   field_node.depends_on = Some(field_dependencies);
        // }

        fields.push(Arc::new(RwLock::new(field_node)));
      }
      Selection::FragmentSpread(e) => {
        fields.push(Arc::new(RwLock::new(FieldNode {
          field: format!("...{}", e.fragment_name),
          children: vec![],
          alias: None,
          arguments: vec![],
          parent_type_name: None,
          sources: vec![],
          type_name: None,
          is_list: false,
          key_fields: None,
          key_field_path: None,
          owner: None,
          requires: None,
          should_be_cleaned: false,
          is_introspection: false,
          query_step: None,
          response: None,
          depends_on_path: Arc::new(RwLock::new(None)),
          str_path: vec![],
          order_index: 0,
        })));
      }
      _ => {}
    }
  }

  Ok(fields)
}
