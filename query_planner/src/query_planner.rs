use std::collections::HashMap;

use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    graphql_query_builder::{generate_entities_query, generate_query_for_field},
    supergraph::{GraphQLType, Supergraph},
    user_query::{FieldNode, Fragments, UserQuery},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
    pub service_name: String,
    pub query: String,
    pub arguments: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Parallel {
    Sequential(Vec<QueryStep>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPlan {
    pub parallel_steps: Vec<Parallel>,
}

pub fn plan_for_user_query(supergraph: &Supergraph, user_query: &UserQuery) -> QueryPlan {
    let parallel_steps: Vec<Parallel> = user_query
        .fields
        .iter()
        .filter_map(|field| {
            let fields = build_queries_services_map(field, supergraph, None, &user_query.fragments);

            if fields.is_empty() {
                None
            } else {
                Some(Parallel::Sequential(
                    fields
                        .into_iter()
                        .map(|(service_name, field_strings)| QueryStep {
                            service_name,
                            query: generate_query_for_field(
                                &user_query.operation_type.to_string(),
                                field,
                                &field_strings,
                            ),
                            arguments: None,
                        })
                        .collect(),
                ))
            }
        })
        .collect();

    QueryPlan { parallel_steps }
}

pub fn contains_entities_query(field_strings: &[String]) -> bool {
    field_strings
        .iter()
        .any(|s| s.contains("_entities(representations: $representations)"))
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

fn build_queries_services_map<'a>(
    field: &FieldNode,
    supergraph: &Supergraph,
    parent_type_name: Option<&str>,
    fragments: &Fragments,
) -> LinkedHashMap<String, Vec<String>> {
    let (field_type, field_type_name) = get_type_info_of_field(&field.field, supergraph);
    let field_type = match field_type {
        Some(ft) => ft,
        None => return LinkedHashMap::new(),
    };

    // Use the provided parent_type_name or the one from get_type_info_of_field
    let field_type_name = parent_type_name.or(field_type_name);

    let mut fields: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();

    for subfield in &field.children {
        let is_fragment = subfield.field.starts_with("...");

        if is_fragment {
            let fragment_name = subfield
                .field
                .split("...")
                .nth(1)
                .expect("Incorrect fragment usage!");
            let fragment_fields = fragments.get(fragment_name).expect(&format!(
                "The used \"{}\" Fragment is not defined!",
                &fragment_name
            ));

            for frag_field in fragment_fields {
                let fragment_query = process_field(frag_field, supergraph, fragments);

                if let Some(field_def) = field_type.fields.get(&frag_field.field) {
                    let existing = fields
                        .entry(field_def.source.clone())
                        .or_insert_with(Vec::new);

                    if !existing.contains(&fragment_query) {
                        existing.push(fragment_query);
                    }
                }
            }
        } else if let Some(field_def) = field_type.fields.get(&subfield.field) {
            let existing = fields
                .entry(field_def.source.clone())
                .or_insert_with(Vec::new);

            let subfield_selection = if subfield.children.is_empty() {
                subfield.field.clone()
            } else {
                format!(
                    "{} {{ {} }}",
                    subfield.field,
                    build_queries_services_map(
                        subfield,
                        supergraph,
                        Some(&field_type_name.unwrap_or_default()),
                        fragments
                    )
                    .values()
                    .flatten()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
                )
            };

            // Get the actual type name of the field.
            let actual_typename =
                get_type_name_of_field(subfield.field.to_string(), None, supergraph)
                    .unwrap_or_default();

            let entity_typename = get_type_name_of_field(field.field.to_string(), None, supergraph)
                .unwrap_or_default()
                .to_string();

            let key_fields_option = supergraph.types.get(actual_typename);

            if let Some(type_info) = key_fields_option {
                if !type_info.key_fields.is_empty() {
                    // Generate entities query using the entity_typename
                    let new_query = generate_entities_query(entity_typename, subfield_selection);
                    existing.push(new_query);

                    continue;
                }
            }
            // Add to existing selections
            if !existing.contains(&subfield_selection) {
                existing.push(subfield_selection);
            }
        }
    }

    // Ensure that key fields are included in the selections if not already present
    for (_service_name, field_selections) in &mut fields {
        if let Some(graphql_type) = get_type_of_field(field.field.to_string(), None, &supergraph) {
            ensure_key_fields_included_for_type(graphql_type, field_selections);
        }

        // Add __typename to the selection set for the type
        if !field_selections.contains(&"__typename".to_string()) {
            field_selections.push("__typename".to_string());
        }
    }

    fields
}

fn process_field<'a>(
    subfield: &FieldNode,
    supergraph: &Supergraph,
    fragments: &Fragments,
) -> String {
    if subfield.children.is_empty() {
        return subfield.field.clone();
    }

    let nested_fields = subfield
        .children
        .iter()
        .map(|child| process_field(child, supergraph, fragments))
        .collect::<Vec<String>>()
        .join(" ");

    format!("{} {{ {} }}", subfield.field, nested_fields)
}

fn ensure_key_fields_included_for_type<'a>(
    graphql_type: &GraphQLType,
    current_selections: &mut Vec<String>,
) {
    // Skip if it's an entities query
    if contains_entities_query(current_selections) {
        return;
    }

    // Create a new vector to hold selections in the correct order
    let mut new_selections = Vec::new();

    // First, add key fields (if they aren't already in the current selections)
    for key_field in &graphql_type.key_fields {
        if !current_selections.contains(key_field) {
            new_selections.push(key_field.clone());
        }
    }

    // Then, add other fields from current_selections
    new_selections.extend(current_selections.iter().cloned());

    // Replace current_selections with the new vector
    *current_selections = new_selections;
}

pub fn get_type_of_field<'a>(
    field_name: String,
    parent_type: Option<String>,
    supergraph: &'a Supergraph,
) -> Option<&'a GraphQLType> {
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

pub fn get_type_name_of_field<'a>(
    field_name: String,
    parent_type: Option<String>,
    supergraph: &'a Supergraph,
) -> Option<&'a str> {
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
