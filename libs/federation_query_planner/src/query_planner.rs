use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, vec};

use crate::{
  constants::CONDUCTOR_INTERNAL_SERVICE_RESOLVER,
  graphql_query_builder::{batch_subqueries, generate_query_for_field},
  supergraph::{GraphQLType, Supergraph},
  user_query::{FieldNode, GraphQLFragment, UserQuery},
};

pub type EntityQueryNeeds = Option<EntityQuerySearch>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
  pub service_name: String,
  pub query: String,
  pub arguments: Option<HashMap<String, String>>,
  pub entity_query_needs: EntityQueryNeeds,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityQuerySearch {
  pub __typename: String,
  pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Parallel {
  Sequential(Vec<QueryStep>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPlan {
  pub parallel_steps: Vec<Parallel>,
}

// fn extract_required_fields_from_requires_string(requires: &str) -> Vec<String> {
//     let parsed_query = parse_query::<String>(requires).expect("Failed to parse requires string");

//     let mut fields = Vec::new();

//     for definition in parsed_query.definitions {
//         match definition {
//             Definition::Operation(operation) => match operation {
//                 OperationDefinition::Query(q) => {
//                     fields.extend(extract_fields_from_selection(&q.selection_set.items));
//                 }
//                 OperationDefinition::Mutation(m) => {
//                     fields.extend(extract_fields_from_selection(&m.selection_set.items));
//                 }
//                 OperationDefinition::Subscription(s) => {
//                     fields.extend(extract_fields_from_selection(&s.selection_set.items));
//                 }
//                 OperationDefinition::SelectionSet(e) => {
//                     fields.extend(extract_fields_from_selection(&e.items));
//                 }
//             },
//             _ => {}
//         }
//     }

//     fields
// }

// fn extract_fields_from_selection(selection_set: &Vec<Selection<String>>) -> Vec<String> {
//     let mut fields = Vec::new();

//     for selection in selection_set {
//         if let Selection::Field(Field { name, .. }) = selection {
//             fields.push(name.clone());
//         }
//     }

//     fields
// }

fn build_intermediate_structure(
  graphql_type: &GraphQLType,
  supergraph: &Supergraph,
  fields: &mut Vec<FieldNode>,
  parent_type_name: Option<&str>,
  fragments: &HashMap<String, GraphQLFragment>,
) -> Result<()> {
  let mut idx = 0;

  while idx < fields.len() {
    let field = &mut fields[idx];
    // field.children.sort_by(|a, b| b.owner.cmp(&a.owner));

    // Handle fragment spreads
    if let Some(fragment_name) = field.field.split("...").nth(1) {
      let (graphql_type_name, fragment) = fragments
        .iter()
        .find(|(key, _)| key == &fragment_name)
        .unwrap_or_else(|| panic!("{} fragment is not defined in your query!", fragment_name));
      let mut fragment_fields = fragment.fields.clone();

      let next_gql_type: &GraphQLType = match supergraph.types.get(graphql_type_name) {
        Some(t) => t,
        None => {
          return Err(anyhow!(format!(
            "Fragment object type \"{}\" not found in supergraph",
            graphql_type_name
          )))
        }
      };

      build_intermediate_structure(
        next_gql_type,
        supergraph,
        &mut fragment_fields,
        None,
        fragments,
      )?;

      fields.remove(idx);
      fields.splice(idx..idx, fragment_fields.drain(..));
      continue;
    } else {
      idx += 1;

      if field.field == "__typename" {
        // Special handling for __typename field:
        // Just continue to the next iteration,
        // as __typename is a meta field that doesn't need further resolution.
        continue;
      } else if field.is_introspection {
        // In the case of detecting an introspection query
        // Handle introspection queries internally
        field.sources = vec![String::from(CONDUCTOR_INTERNAL_SERVICE_RESOLVER)];
      } else {
        let gql_field = match graphql_type.fields.get(&field.field) {
          Some(f) => f,
          None => {
            return Err(anyhow!(format!(
              "Field \"{}\" is not available on type {}",
              field.field,
              parent_type_name.unwrap_or("Query")
            )))
          }
        };

        let child_type_name = unwrap_graphql_type(&gql_field.field_type);

        field.parent_type_name = parent_type_name.map(String::from);
        field.sources = gql_field.sources.clone();
        if graphql_type.owner.is_some()
          && field.sources.contains(graphql_type.owner.as_ref().unwrap())
        {
          field.owner = graphql_type.owner.clone();
        }

        field.type_name = Some(unwrap_graphql_type(gql_field.field_type.as_str()).to_string());
        field.key_fields = graphql_type.key_fields.clone();
        field.requires = gql_field.requires.clone();

        if !field.children.is_empty() {
          let new_field = FieldNode {
            field: String::from("__typename"),
            alias: None,
            arguments: vec![],
            children: vec![],
            sources: field.sources.clone(),
            type_name: None,
            parent_type_name: None,
            key_fields: None,
            owner: None,
            requires: None,
            should_be_cleaned: true, // clean it in the response merging phase
            relevant_sub_queries: None,
            is_introspection: false,
          };

          field.children.push(new_field);

          let next_gql_type: &GraphQLType = match supergraph.types.get(child_type_name) {
            Some(t) => t,
            None => {
              return Err(anyhow!(format!(
                "Type \"{}\" not found in supergraph",
                child_type_name
              )))
            }
          };

          build_intermediate_structure(
            next_gql_type,
            supergraph,
            &mut field.children,
            Some(unwrap_graphql_type(gql_field.field_type.as_str())),
            fragments,
          )?;
        }
      }
    }

    // TODO: add back `requires` fields.
    // if let Some(required_fields_string) = &field.requires {
    //     let required_fields = extract_required_fields_from_requires_string(&format!(
    //         "{{{}}}",
    //         required_fields_string
    //     ));

    //     for required_field in required_fields {
    //         // Check if this required_field is already a child of the parent
    //         if !field
    //             .children
    //             .iter()
    //             .any(|child| child.field == required_field)
    //         {
    //             let supergraph_parent_type = supergraph
    //                 .types
    //                 .get(field.parent_type_name.as_ref().unwrap());
    //             let that_required_field = supergraph_parent_type
    //                 .and_then(|gql_type| gql_type.fields.get(&required_field))
    //                 .expect(&format!(

    //                     "requires field {} doesn't exist on the parent type {}, your supergraph schema has an error!",
    //                     required_field, parent_type_name.unwrap()
    //                 ));

    //             let new_field = FieldNode {
    //                 field: required_field,
    //                 alias: None,
    //                 arguments: vec![],
    //                 children: vec![],
    //                 sources: that_required_field.sources.clone(),
    //                 type_name: Some(that_required_field.field_type.clone()),
    //                 parent_type_name: field.parent_type_name.clone(),
    //                 key_fields: supergraph_parent_type.unwrap().key_fields.clone(),
    //                 owner: supergraph_parent_type.unwrap().owner.clone(),
    //                 requires: that_required_field.requires.clone(),
    //                 should_be_cleaned: true, // clean it in the response merging phase
    //             };
    //             field.children.push(new_field);
    //         }
    //     }
    // }
  }

  Ok(())
}

pub fn plan_for_user_query(
  supergraph: &Supergraph,
  user_query: &mut UserQuery,
) -> Result<QueryPlan> {
  let (_name, query_fields) = supergraph
    .types
    .iter()
    .find(|(name, _t)| name == &"Query")
    .expect(
      // TODO: should be handled at startup instead
      "Query type object is not defined in your supergraph schema!",
    );

  build_intermediate_structure(
    query_fields,
    supergraph,
    &mut user_query.fields,
    None,
    &user_query.fragments,
  )?;

  let mut mappings: Vec<(String, String, EntityQueryNeeds)> = vec![];

  for field in &mut user_query.fields {
    build_fields_mappings_to_subgraphs(field, None, &mut mappings, supergraph);
  }

  // TODO: that `.rev()` might be expensive!
  let mappings = batch_subqueries(mappings.into_iter().rev().collect());

  // TODO: uncomment this
  // batch_subqueries_in_user_query(user_query);
  // fs::write(
  //     "user-query.json",
  //     serde_json::to_string(user_query).unwrap(),
  // );

  let steps: Parallel = Parallel::Sequential(
    mappings
      .into_iter()
      .map(|(subgraph, e, entity_query_needs)| QueryStep {
        arguments: None,
        query: generate_query_for_field(user_query.operation_type.to_string(), e),
        service_name: subgraph.clone(),
        entity_query_needs,
      })
      .collect::<Vec<QueryStep>>(),
  );

  Ok(QueryPlan {
    parallel_steps: vec![steps],
  })
}

fn build_fields_mappings_to_subgraphs(
  field: &mut FieldNode,
  parent_source: Option<&str>,
  results: &mut Vec<(String, String, EntityQueryNeeds)>,
  supergraph: &Supergraph,
) {
  resolve_children(
    field,
    parent_source,
    results,
    false,
    supergraph,
    (None, &mut None),
  );
}

type ParentInfo<'a> = (Option<String>, &'a mut Option<Vec<(String, String)>>);

fn resolve_children(
  field: &mut FieldNode,
  parent_source: Option<&str>,
  results: &mut Vec<(String, String, EntityQueryNeeds)>,
  nested: bool,
  _supergraph: &Supergraph,
  (persisted_parent_type_name, shared_parent_type_name_field): ParentInfo,
) -> String {
  let current_source = determine_owner(&field.sources, field.owner.as_ref(), parent_source);

  let children_results: Vec<_> = field
    .children
    .iter_mut()
    .filter_map(|e| {
      let source = determine_owner(&e.sources, e.owner.as_ref(), parent_source);

      let res = resolve_children(
        e,
        Some(&current_source),
        results,
        source == current_source,
        _supergraph,
        if persisted_parent_type_name == field.parent_type_name {
          (
            persisted_parent_type_name.clone(),
            shared_parent_type_name_field,
          )
        } else {
          (
            field.parent_type_name.clone(),
            &mut field.relevant_sub_queries,
          )
        },
      );

      if source != current_source || res.is_empty() {
        None
      } else {
        Some(res)
      }
    })
    .collect();

  // Return an empty string if the field has no valid children and is not itself a source
  // if children_results.is_empty() && !field.sources.contains(&current_source) {
  //     return String::with_capacity(0);
  // }

  let res = if children_results.is_empty() {
    field.field.to_string()
  } else {
    format!("{} {{ {} }}", field.field, children_results.join(" "))
  };

  if !nested && !res.is_empty() {
    let current_source_str = current_source.to_string();

    let (result, entity_key_map) = if field.key_fields.is_some()
            // don't do an entity query on a root Query resolvable field
            && field.parent_type_name.is_some()
    {
      // If no children, populate the current field
      if !field.children.is_empty() {
        field.relevant_sub_queries.get_or_insert(vec![]).push((
          current_source_str.clone(),
          format!("{}#{}", field.parent_type_name.as_ref().unwrap(), &res),
        ));
      } else {
        shared_parent_type_name_field.get_or_insert(vec![]).push((
          current_source_str.clone(),
          format!("{}#{}", field.parent_type_name.as_ref().unwrap(), &res),
        ));
      }
      (
        format!("{}#{}", field.parent_type_name.as_ref().unwrap(), &res),
        Some(EntityQuerySearch {
          __typename: field.parent_type_name.as_ref().unwrap().clone(),
          fields: vec![field.key_fields.as_ref().unwrap().clone()],
        }),
      )
    } else {
      field.relevant_sub_queries = Some(vec![(current_source_str.clone(), res.clone())]);
      (res.clone(), None)
    };

    // if let Some(graphql_type) = supergraph.types.get(field.type_name.as_ref().unwrap()) {
    //     ensure_key_fields_included_for_type(
    //         graphql_type,
    //         &mut results.get_mut(&current_source).unwrap(),
    //     );
    // }

    results.push((current_source_str, result, entity_key_map));
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

pub fn contains_entities_query(field_strings: &str) -> bool {
  field_strings.contains("_entities(representations: $representations)")
}

pub fn get_type_info_of_field<'a>(
  field_name: &'a str,
  supergraph: &'a Supergraph,
) -> (Option<&'a GraphQLType>, Option<&'a str>) {
  for (type_name, type_def) in &supergraph.types {
    if let Some(field_def) = type_def.fields.get(field_name) {
      return (
        supergraph
          .types
          .get(unwrap_graphql_type(&field_def.field_type)),
        Some(type_name),
      );
    }
  }
  (None, None)
}

// fn build_queries_services_map<'a>(
//     field: &FieldNode,
//     fragments: &Fragments,
//     selection_set: &mut FieldSelectionSet,
// ) {
//     let (field_type, field_type_name) = get_type_info_of_field(&field.field, &field.supergraph);
//     let field_type = match field_type {
//         Some(ft) => ft,
//         None => return,
//     };

//     // Use the provided parent_type_name or the one from get_type_info_of_field
//     let field_type_name = field.parent_type_name.or(field_type_name);

//     for subfield in &field.children {
//         let is_fragment = subfield.field.starts_with("...");

//         if is_fragment {
//             let fragment_name = subfield
//                 .field
//                 .split("...")
//                 .nth(1)
//                 .expect("Incorrect fragment usage!");
//             let fragment_fields = fragments.get(fragment_name).expect(&format!(
//                 "The used \"{}\" Fragment is not defined!",
//                 &fragment_name
//             ));

//             for frag_field in &fragment_fields.fields {
//                 let fragment_query = process_field(frag_field, &field.supergraph, fragments);

//                 if let Some(field_def) = field_type.fields.get(&frag_field.field) {
//                     selection_set.add_field(&field_def.source, fragment_query);
//                 }
//             }
//         } else if let Some(field_def) = field_type.fields.get(&subfield.field) {
//             let subfield_selection = if subfield.children.is_empty() {
//                 subfield.field.clone()
//             } else {
//                 build_queries_services_map(subfield, fragments, selection_set);
//                 format!(
//                     "{} {{ {} }}",
//                     subfield.field,
//                     selection_set
//                         .get_fields_for_service(&field_def.source)
//                         .unwrap_or(&vec![])
//                         .join(" ")
//                 )
//             };

//             // Get the actual type name of the field.
//             let actual_typename =
//                 get_type_name_of_field(subfield.field.to_string(), None, &field.supergraph)
//                     .unwrap_or_default();

//             let entity_typename =
//                 get_type_name_of_field(field.field.to_string(), None, &field.supergraph)
//                     .unwrap_or_default()
//                     .to_string();

//             let key_fields_option = field.supergraph.types.get(&actual_typename);

//             if let Some(type_info) = field
//                 .supergraph
//                 .types
//                 .get(&unwrap_graphql_type(&field_def.field_type))
//             {
//                 if !type_info.key_fields.is_empty() {
//                     // Generate entities query using the entity_typename
//                     let new_query = generate_entities_query(entity_typename, subfield_selection);
//                     selection_set.add_field(&field_def.source, new_query);
//                     continue;
//                 }
//             }

//             selection_set.add_field(&field_def.source, subfield_selection);

//             // Ensure that key fields are included in the selections if not already present
//             if let Some(graphql_type) =
//                 get_type_of_field(field.field.to_string(), None, &field.supergraph)
//             {
//                 ensure_key_fields_included_for_type(
//                     graphql_type,
//                     selection_set
//                         .fields
//                         .entry(field_def.source.clone())
//                         .or_insert_with(Vec::new),
//                 );
//             }

//             // Add __typename to the selection set for the type
//             if let Some(field_selections) = selection_set.get_fields_for_service(&field_def.source)
//             {
//                 if !field_selections.contains(&"__typename".to_string()) {
//                     selection_set.add_field(&field_def.source, "__typename".to_string());
//                 }
//             }
//         }
//     }
// }

// fn process_field<'a>(subfield: &FieldNode, supergraph: &Supergraph) -> String {
//     if subfield.children.is_empty() {
//         return subfield.field.clone();
//     }

//     let nested_fields = subfield
//         .children
//         .iter()
//         .map(|child| process_field(child, supergraph))
//         .collect::<Vec<String>>()
//         .join(" ");

//     format!("{} {{ {} }}", subfield.field, nested_fields)
// }

// fn ensure_key_fields_included_for_type<'a>(
//     graphql_type: &GraphQLType,
//     current_selections: &mut String,
// ) {
//     // Skip if it's an entities query
//     if contains_entities_query(&current_selections) {
//         return;
//     }

//     // Create a new vector to hold selections in the correct order
//     let mut new_selections = Vec::new();

//     // First, add key fields (if they aren't already in the current selections)
//     for key_field in &graphql_type.key_fields {
//         if !current_selections.contains(key_field) {
//             new_selections.push(key_field.clone());
//         }
//     }

//     // Then, add other fields from current_selections
//     new_selections.extend(current_selections.iter().cloned());

//     // Replace current_selections with the new vector
//     *current_selections = new_selections;
// }

pub fn get_type_of_field(
  field_name: String,
  parent_type: Option<String>,
  supergraph: &Supergraph,
) -> Option<&GraphQLType> {
  for (type_name, type_def) in &supergraph.types {
    // Check if we should restrict by parent type
    if let Some(parent) = &parent_type {
      if parent != type_name {
        continue;
      }
    }

    if let Some(field_def) = type_def.fields.get(&field_name) {
      return supergraph
        .types
        .get(unwrap_graphql_type(&field_def.field_type));
    }
  }

  None
}

pub fn get_type_name_of_field(
  field_name: String,
  parent_type: Option<String>,
  supergraph: &Supergraph,
) -> Option<&str> {
  for (type_name, type_def) in &supergraph.types {
    // Check if we should restrict by parent type
    if let Some(parent) = &parent_type {
      if parent != type_name {
        continue;
      }
    }

    if let Some(field_def) = type_def.fields.get(&field_name) {
      return Some(unwrap_graphql_type(&field_def.field_type));
    }
  }

  None
}

fn unwrap_graphql_type(typename: &str) -> &str {
  let mut unwrapped = typename;
  while unwrapped.ends_with('!') || unwrapped.starts_with('[') || unwrapped.ends_with(']') {
    unwrapped = unwrapped.trim_end_matches('!');
    unwrapped = unwrapped.trim_start_matches('[');
    unwrapped = unwrapped.trim_end_matches(']');
  }
  unwrapped
}
