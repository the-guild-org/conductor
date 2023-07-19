use crate::query_planner::Parallel;

use super::query_planner::{QueryPlan, QueryStep};
use super::supergraph::Supergraph;
use anyhow::{anyhow, Result};
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
) -> Result<Vec<Vec<((String, String), QueryResponse)>>> {
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

    match results {
        Ok(val) => Ok(val),
        Err(e) => Err(anyhow!(e)),
    }
}

async fn execute_sequential(
    query_steps: &Vec<QueryStep>,
    supergraph: &Supergraph,
) -> Result<Vec<((String, String), QueryResponse)>> {
    let mut data_vec = vec![];
    let mut entity_arguments: Option<serde_json::Value> = None;

    for (i, query_step) in query_steps.iter().enumerate() {
        let data = execute_query_step(query_step, supergraph, entity_arguments.clone()).await;

        match data {
            Ok(data) => {
                data_vec.push((
                    (query_step.service_name.clone(), query_step.query.clone()),
                    data,
                ));

                if i + 1 < query_steps.len() {
                    let next_step = &query_steps[i + 1];
                    match &next_step.entity_query_needs {
                        Some(needs) => {
                            data_vec.iter().find(|&data| {
                                if let Some(x) = data.1.data.as_ref() {
                                    // recursively search and find match
                                    let y = find_objects_matching_criteria(
                                        x,
                                        &needs.__typename,
                                        &needs.fields.clone().into_iter().next().unwrap(),
                                    );

                                    if y.is_empty() {
                                        return false;
                                    } else {
                                        entity_arguments = Some(Value::from(y));
                                        return true;
                                    }
                                }

                                false
                            });

                            Some(serde_json::json!({ "representations": entity_arguments }))
                        }
                        None => None,
                    }
                } else {
                    None
                };
            }
            Err(err) => return Err(err),
        }
    }

    let x: Vec<((String, String), QueryResponse)> = data_vec
        .into_iter()
        .map(|(plan_meta, response)| {
            let new_response = QueryResponse {
                data: response.data,
                // Initialize other fields of QueryResponse as needed
                errors: response.errors,
            };
            (plan_meta, new_response)
        })
        .collect::<Vec<((std::string::String, std::string::String), QueryResponse)>>();

    Ok(x)
}

fn find_objects_matching_criteria(json: &Value, typename: &str, field: &str) -> Vec<Value> {
    let mut matching_objects = Vec::new();

    match json {
        Value::Object(map) => {
            if let (Some(typename_value), Some(field_value)) = (
                map.get("__typename").and_then(|v| v.as_str()),
                map.get(field),
            ) {
                if typename_value == typename {
                    let mut result = serde_json::Map::new();
                    result.insert(
                        "__typename".to_string(),
                        Value::String(typename.to_string()),
                    );
                    result.insert(field.to_string(), field_value.clone());
                    matching_objects.push(Value::Object(result));
                }
            }
            for (_, value) in map {
                matching_objects.extend(find_objects_matching_criteria(value, typename, field));
            }
        }
        Value::Array(arr) => {
            for element in arr {
                matching_objects.extend(find_objects_matching_criteria(element, typename, field));
            }
        }
        _ => {}
    }

    matching_objects
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct QueryResponse {
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<serde_json::Value>>,
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
