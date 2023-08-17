use crate::query_planner::Parallel;

use super::query_planner::{QueryPlan, QueryStep};
use super::supergraph::Supergraph;
use anyhow::Result;
use futures::future::join_all;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn execute_query_plan(
    query_plan: &QueryPlan,
    supergraph: &Supergraph,
) -> Result<Vec<QueryResponse>> {
    let mut all_futures = Vec::new();

    for step in &query_plan.parallel_steps {
        match step {
            Parallel::Sequential(query_steps) => {
                let future = execute_sequential(query_steps, supergraph);
                all_futures.push(future);
            }
        }
    }

    let results: Result<Vec<_>, _> = join_all(all_futures).await.into_iter().collect();

    Ok(results.unwrap())
}

async fn execute_sequential(
    query_steps: &Vec<QueryStep>,
    supergraph: &Supergraph,
) -> Result<QueryResponse> {
    let mut entity_arguments = None;
    let mut data_vec = vec![];

    for query_step in query_steps {
        let data = execute_query_step(query_step, supergraph, entity_arguments).await;

        match data {
            Ok(data) => {
                entity_arguments = extract_key_fields_from_response(&data, supergraph);
                data_vec.push(data);
            }
            Err(err) => return Err(err),
        }
    }

    Ok(QueryResponse {
        data: Some(serde_json::Value::Array(
            data_vec
                .into_iter()
                .map(|response| response.data.unwrap_or_default())
                .collect(),
        )),
        errors: None,
    })
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

    let variables_object = if let Some(arguments) = &entity_arguments {
        serde_json::json!({ "representations": arguments })
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    let response = match CLIENT
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
