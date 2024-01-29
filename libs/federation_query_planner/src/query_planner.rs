use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  ops::Index,
  sync::{Arc, RwLock},
  vec,
};

use crate::{
  graphql_query_builder::{
    generate_entities_query, generate_query_for_field, parse_into_selection_set,
  },
  supergraph::Supergraph,
  user_query::{FieldNode, OperationType, UserQuery},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryStep {
  pub service_name: String,
  pub query: String,
  pub arguments: Option<HashMap<String, String>>,
  pub entity_query_needs_path: Option<Vec<Vec<String>>>,
  pub entity_typename: Option<String>,
}

pub fn plan_for_user_query(supergraph: &Supergraph, user_query: &mut UserQuery) -> Result<()> {
  user_query.populate_supergraph_metadata(supergraph)?;

  for field in user_query.fields.clone() {
    let field_node = UserQuery::read_field_node(field.clone())?;
    let mut current_path = vec![];
    current_path.push(field_node.field.clone());

    let prev_field = field_node.clone();

    resolve_children(
      &user_query.operation_type,
      field,
      prev_field,
      None,
      false,
      supergraph,
    );
  }

  Ok(())
}

fn is_scalar(type_name: &str) -> bool {
  let built_in_scalars = ["Int", "Float", "String", "Boolean", "ID"];

  built_in_scalars.contains(&type_name)
}

fn resolve_children(
  operation_type: &OperationType,
  field: Arc<RwLock<FieldNode>>,
  prev_field: FieldNode,
  parent_source: Option<&str>,
  nested: bool,
  supergraph: &Supergraph,
) -> String {
  let read_field = UserQuery::read_field_node(field.clone()).unwrap();
  let current_source = determine_owner(
    &read_field.sources,
    read_field.owner.as_ref(),
    parent_source,
  );

  // this makes sure that we only update the prev field when we change from an entity to another
  let new_prev_field = if nested {
    prev_field.clone()
  } else {
    read_field.clone()
  };

  let children_results: Vec<_> = read_field
    .children
    .clone()
    .iter_mut()
    .filter_map(|e| {
      let read_field = UserQuery::read_field_node(e.clone()).unwrap();
      let source = determine_owner(
        &read_field.sources,
        read_field.owner.as_ref(),
        Some(&current_source),
      );

      let res = resolve_children(
        operation_type,
        e.clone(),
        new_prev_field.clone(),
        Some(&current_source),
        source == current_source,
        supergraph,
      );

      if source != current_source || res.is_empty() {
        None
      } else {
        Some(res)
      }
    })
    .collect();

  let res = if children_results.is_empty() {
    // this check is to avoid fetching a field that's an object with no children fields which will result in the error: "must have a selection of subfields"
    if read_field.is_introspection || is_scalar(read_field.type_name.as_ref().unwrap()) {
      parse_into_selection_set(&read_field)
    } else {
      String::new()
    }
  } else {
    format!(
      "{} {{ {} }}",
      parse_into_selection_set(&read_field),
      children_results.join(" ")
    )
  };

  if !nested && !res.is_empty() {
    let current_source_str = current_source.to_string();

    let step = if read_field.key_fields.is_some()
            // don't do an entity query on a root Query resolvable field
            && read_field.parent_type_name.is_some()
    {
      // println!("{}", prev_field.field);
      // println!("^^^^^");

      // let dependent_field_path = prev_field
      //   .children
      //   .iter()
      //   .find(|&e| {
      //     let x = e.read().unwrap().clone();

      //     println!("- {}", read_field.field);
      //     println!("{}", x.field);
      //     println!("{:?}", read_field.key_fields);
      //     println!("-----------------");

      //     &x.field == read_field.key_fields.as_ref().unwrap()
      //   })
      //   .unwrap()
      //   .read()
      //   .unwrap()
      //   .str_path
      //   .clone();

      QueryStep {
        query: generate_query_for_field(
          operation_type.to_string(),
          generate_entities_query(
            read_field.parent_type_name.as_ref().unwrap(),
            read_field.key_fields.as_ref().unwrap(),
            &res.clone(),
          ),
          "",
        ),
        entity_typename: read_field.parent_type_name,
        service_name: current_source_str,
        arguments: None,
        // TODO: handle multiple keys in case of selection set
        entity_query_needs_path: Some(vec![read_field.key_field_path.unwrap()]),
      }
    } else {
      // fragments will be only there in the case of introspection
      let selection_set_with_fragments = res.split('|').collect::<Vec<&str>>();

      QueryStep {
        query: generate_query_for_field(
          operation_type.to_string(),
          selection_set_with_fragments.index(0).to_string(),
          selection_set_with_fragments.get(1).unwrap_or(&""), //  field.arguments
        ),

        service_name: current_source_str,
        arguments: None,
        entity_query_needs_path: None,
        entity_typename: None,
      }
    };

    UserQuery::write_field_node(field, |x| {
      x.query_step = Some(step.clone());
    })
    .unwrap();
  }

  res
}

fn determine_owner(
  field_sources: &[String],
  owner: Option<&String>,
  parent_source: Option<&str>,
) -> String {
  // 1. Check if there's only one join, if yes, just return it
  if field_sources.len() == 1 {
    return field_sources.first().unwrap().clone();
  }

  // 2. Check if it has an owner defined
  if let Some(owner_str) = owner {
    return owner_str.to_string();
  }

  // 3. Check if the parent source is present in field sources and return it
  if let Some(p) = parent_source {
    let parent_soruce_str = p.to_string();
    if field_sources.contains(&parent_soruce_str) {
      return parent_soruce_str;
    }
  }

  // 4. If no match for parent source, return the first one as default
  field_sources.first().cloned().expect("No sources found")
}
