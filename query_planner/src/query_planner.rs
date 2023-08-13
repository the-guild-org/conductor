use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
};

use anyhow::Result;
use async_graphql::futures_util::future::join_all;
use linked_hash_map::LinkedHashMap;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    query_builder::stringify_arguments,
    supergraph::{GraphQLType, Supergraph},
    user_query::{FieldNode, UserQuery},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
    pub service_name: String,
    pub query: String,
    pub arguments: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExecutionItem {
    SingleQuery(QueryStep),
    SeqQueries(Vec<QueryStep>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExecutionStep {
    Parallel(Vec<ExecutionItem>),
    Sequential(Vec<QueryStep>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPlan {
    pub steps: Vec<ExecutionStep>,
}

pub async fn plan_for_user_query(supergraph: &Supergraph, user_query: &UserQuery) -> QueryPlan {
    let mut steps_in_parallel: Vec<ExecutionItem> = vec![];

    let operation_name = match &user_query.operation_name {
        Some(v) => v.to_string(),
        None => "Query".to_string(),
    };

    for field in &user_query.fields {
        let mut seq_steps: Vec<QueryStep> = vec![];

        // Main field is not external, so generate a direct query
        let fields = get_direct_local_subfields_as_str(field, supergraph, None);

        for (service_name, fields) in fields {
            let query = if fields
                .join(" ")
                .contains("_entities(representations: $representations)")
            {
                format!(
                    "{} {}($representations: [_Any!]!) {{ {} }} }}",
                    user_query.operation_type,
                    operation_name,
                    fields.join(" ")
                )
            } else {
                let arguments = match !field.arguments.is_empty() {
                    true => format!("({})", stringify_arguments(&field.arguments)),
                    false => "".to_string(),
                };
                format!(
                    "{} {} {{ {}{} {{ {} }} }}",
                    user_query.operation_type,
                    operation_name,
                    field.field,
                    arguments,
                    fields.join(" ")
                )
            };

            let step = QueryStep {
                service_name,
                query,
                arguments: None,
            };

            seq_steps.push(step);
        }

        if !seq_steps.is_empty() {
            steps_in_parallel.push(ExecutionItem::SeqQueries(seq_steps));
        }
    }

    QueryPlan {
        steps: vec![ExecutionStep::Parallel(steps_in_parallel)],
    }
}

fn get_direct_local_subfields_as_str<'a>(
    field: &FieldNode,
    supergraph: &Supergraph,
    parent_type_name: Option<&str>,
) -> LinkedHashMap<String, Vec<String>> {
    let field_type = match get_type_of_field(field.field.to_string(), None, supergraph) {
        Some(ft) => ft,
        None => return LinkedHashMap::new(),
    };

    // Use the provided parent_type_name or try to find one based on the current field
    let field_type_name = parent_type_name
        .or_else(|| get_type_name_of_field(field.field.to_string(), None, supergraph));

    let mut fields: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();

    for subfield in &field.children {
        if let Some(field_def) = field_type.fields.get(&subfield.field) {
            let existing = fields
                .entry(field_def.source.clone())
                .or_insert_with(Vec::new);

            let subfield_selection = if subfield.children.is_empty() {
                subfield.field.clone()
            } else {
                format!(
                    "{} {{ {} }}",
                    subfield.field,
                    get_direct_local_subfields_as_str(
                        subfield,
                        supergraph,
                        Some(&field_type_name.unwrap_or_default())
                    )
                    .values()
                    .flatten()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
                )
            };

            // Check if it's an entity field
            let typename = get_type_name_of_field(subfield.field.clone(), None, supergraph)
                .unwrap_or_default();

            let entity_typename = field_type_name.unwrap();

            // println!("{:?}", entity_typename);

            let key_fields_option = supergraph.types.get(typename);

            if let Some(type_info) = key_fields_option {
                if !type_info.key_fields.is_empty() {
                    // Generate entities query and update selection
                    let new_query = generate_entities_query(
                        subfield,
                        entity_typename.to_string(),
                        subfield_selection,
                    );
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
    for (service_name, field_selections) in &mut fields {
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

fn ensure_key_fields_included_for_type<'a>(
    graphql_type: &GraphQLType,
    current_selections: &mut Vec<String>,
) {
    // Skip if it's an entities query
    if current_selections
        .iter()
        .find(|e| e.contains("_entities(representations: $representations)"))
        .is_some()
    {
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

// Adjusted this function to ignore external fields when gathering direct subfields
fn get_direct_subfields_as_str<'a>(field: &FieldNode, supergraph: &Supergraph) -> String {
    // Determine the type of the field in the GraphQL schema
    let field_type = match supergraph
        .types
        .values()
        .find(|type_def| type_def.fields.contains_key(&field.field))
    {
        Some(type_def) => &type_def.fields[&field.field].field_type,
        None => {
            println!("Unable to determine the type of field: {}", field.field);
            return String::new();
        }
    };

    // Unwrap any list or non-null types to get to the base type
    let unwrapped_type = unwrap_graphql_type(field_type);

    // Fetch the fields of the unwrapped type
    if let Some(type_def) = supergraph.types.get(unwrapped_type) {
        return type_def
            .fields
            .keys()
            .filter(|key_field| {
                !type_def
                    .fields
                    .get(key_field.as_str())
                    .expect("couldn't find field using `key_string`")
                    .external
            })
            .map(|k| k.clone())
            .collect::<Vec<_>>()
            .join(" ");
    } else {
        println!("Unable to find definition for type: {}", unwrapped_type);
        return String::new();
    }
}

// Given a field, determine which service can provide it directly
fn get_service_for_field<'a>(field: &FieldNode, supergraph: &Supergraph) -> Option<String> {
    for (_, type_def) in &supergraph.types {
        if let Some(field_def) = type_def.fields.get(&field.field) {
            if !field_def.external {
                return Some(field_def.source.clone());
            }
        }
    }
    None
}

// For fields that can't be directly queried, get the service that can resolve them via an _entities query
// and also return the primary key field used for querying the _entities
fn get_service_and_key_for_field<'a>(
    field: &FieldNode,
    supergraph: &Supergraph,
) -> Option<(String, Vec<String>)> {
    for (_, type_def) in &supergraph.types {
        if let Some(field_def) = type_def.fields.get(&field.field) {
            if field_def.external || field_def.requires.is_some() {
                return Some((field_def.source.clone(), type_def.key_fields.clone()));
            }
        }
    }
    None
}

fn build_representation_args(typename: &str, id_fields: Vec<String>) -> HashMap<String, String> {
    let mut args = HashMap::new();

    let representations = id_fields
        .iter()
        .map(|id_field| {
            format!(
                "{{ \"__typename\": \"{}\", \"{}\": $id }}",
                typename, id_field
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    args.insert("representations".into(), format!("[{}]", representations));
    args
}

// Given a field and its primary key, generate an _entities query for it
fn generate_entities_query<'a>(
    field: &FieldNode,
    typename: String,
    selection_set: String,
) -> String {
    println!("Generating entities query for {}", field.field.as_str());

    format!(
        "
  _entities(representations: $representations) {{
    ... on {} {{
      {}
    }}
  ",
        typename, selection_set
    )
}

fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(chars).collect(),
    }
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

pub fn get_entity_type_name_of_field<'a>(
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

pub async fn execute_query_plan(
    query_plan: &QueryPlan,
    supergraph: &Supergraph,
) -> Result<Vec<QueryResponse>> {
    let mut all_responses = Vec::new();

    for step in &query_plan.steps {
        let mut responses: Vec<QueryResponse> = Vec::new();

        let step_responses = match step {
            ExecutionStep::Parallel(execution_items) => {
                let futures: Vec<_> = execution_items
                    .iter()
                    .map(|item| async move {
                        match item {
                            ExecutionItem::SingleQuery(query_step) => {
                                execute_query_step(query_step, supergraph, None).await
                            }
                            ExecutionItem::SeqQueries(query_steps) => {
                                let mut entity_arguments = None;

                                let mut data_vec = vec![];
                                for query_step in query_steps {
                                    let data = execute_query_step(
                                        query_step,
                                        supergraph,
                                        entity_arguments.clone(),
                                    )
                                    .await;

                                    match data {
                                        Ok(data) => {
                                            entity_arguments =
                                                extract_key_fields_from_response(&data, supergraph);
                                            data_vec.push(data);
                                        }
                                        Err(err) => return Err(err),
                                    }
                                }

                                Ok(QueryResponse {
                                    data: Some(serde_json::Value::Array(
                                        data_vec
                                            .iter()
                                            .map(|response| {
                                                response.data.clone().unwrap_or_default()
                                            })
                                            .collect(),
                                    )),
                                    errors: None,
                                })
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                for future_response in futures {
                    match future_response.await {
                        Ok(response) => responses.push(response),
                        Err(err) => return Err(err),
                    }
                }

                responses
            }
            ExecutionStep::Sequential(query_steps) => {
                let mut entity_arguments = None;

                let mut data_vec: Vec<QueryResponse> = Vec::new();
                for query_step in query_steps {
                    let data = execute_query_step(query_step, supergraph, entity_arguments.clone())
                        .await?;

                    entity_arguments = extract_key_fields_from_response(&data, supergraph);

                    data_vec.push(data);
                }

                data_vec
            }
        };

        all_responses.extend(step_responses);
    }

    Ok(all_responses)
}

#[derive(Deserialize, Debug, Serialize)]
pub struct QueryResponse {
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<serde_json::Value>>,
}

impl Default for QueryResponse {
    fn default() -> Self {
        QueryResponse {
            data: None,
            errors: None,
        }
    }
}

async fn execute_query_step(
    query_step: &QueryStep,
    supergraph: &Supergraph,
    entity_arguments: Option<serde_json::Value>,
) -> Result<QueryResponse> {
    let url = supergraph.subgraphs.get(&query_step.service_name).unwrap();
    // println!("EXECUTING A QUERY PLAN!!!!!");

    let variables_object = if let Some(arguments) = &entity_arguments {
        serde_json::json!({ "representations": arguments })
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    let response = match reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "query": query_step.query,
                "variables": variables_object
            })
            .to_string(),
        )
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to send request: {}", err);
            return Err(anyhow::anyhow!("Failed to send request: {}", err));
        }
    };

    if !response.status().is_success() {
        eprintln!("Received error response: {:?}", response.status());
        return Err(anyhow::anyhow!(
            "Failed request with status: {}",
            response.status()
        ));
    }

    let response_data = match response.json::<QueryResponse>().await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to parse response: {}", err);
            return Err(anyhow::anyhow!("Failed to parse response: {}", err));
        }
    };

    // Check if there were any GraphQL errors
    if let Some(errors) = &response_data.errors {
        for error in errors {
            eprintln!("Error: {:?}", error);
        }
    }

    // println!(
    //     "Result from service '{}':\n{:?}",
    //     query_step.service_name, response_data.data
    // );

    Ok(response_data)
}

fn extract_key_fields_from_response(
    response: &QueryResponse,
    supergraph: &Supergraph,
) -> Option<serde_json::Value> {
    if let Some(serde_json::Value::Object(data)) = &response.data {
        let mut key_fields_map = vec![];

        for (_, value) in data {
            match value {
                serde_json::Value::Array(array_values) => {
                    for item in array_values {
                        if let Some(object) = item.as_object() {
                            if let Some(serde_json::Value::String(typename)) =
                                object.get("__typename")
                            {
                                if let Some(graphql_type) = supergraph.types.get(typename) {
                                    let mut key_object = serde_json::Map::new();
                                    key_object.insert(
                                        "__typename".to_string(),
                                        serde_json::Value::String(typename.clone()),
                                    );

                                    for key_field in &graphql_type.key_fields {
                                        if let Some(field_value) = object.get(key_field) {
                                            key_object
                                                .insert(key_field.clone(), field_value.clone());
                                        }
                                    }
                                    if !key_object.is_empty() {
                                        key_fields_map.push(serde_json::Value::Object(key_object));
                                    }
                                }
                            }
                        }
                    }
                }
                serde_json::Value::Object(object) => {
                    if let Some(serde_json::Value::String(typename)) = object.get("__typename") {
                        if let Some(graphql_type) = supergraph.types.get(typename) {
                            let mut key_object = serde_json::Map::new();
                            key_object.insert(
                                "__typename".to_string(),
                                serde_json::Value::String(typename.clone()),
                            );

                            for key_field in &graphql_type.key_fields {
                                if let Some(field_value) = object.get(key_field) {
                                    key_object.insert(key_field.clone(), field_value.clone());
                                }
                            }
                            if !key_object.is_empty() {
                                key_fields_map.push(serde_json::Value::Object(key_object));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !key_fields_map.is_empty() {
            return Some(serde_json::Value::Array(key_fields_map));
        }
    }

    None
}
