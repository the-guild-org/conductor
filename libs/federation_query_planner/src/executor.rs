use std::pin::Pin;

use crate::constants::CONDUCTOR_INTERNAL_SERVICE_RESOLVER;
use crate::query_planner::Parallel;

use super::query_planner::{QueryPlan, QueryStep};
use super::supergraph::Supergraph;
use anyhow::{anyhow, Result};
use async_graphql::{dynamic::*, Error, Value};
use futures::future::join_all;
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeValue;
use tracing::Instrument;

pub async fn execute_query_plan(
  client: &conductor_tracing::reqwest_utils::TracedHttpClient,
  query_plan: &QueryPlan,
  supergraph: &Supergraph,
) -> Result<Vec<Vec<((String, String), QueryResponse)>>> {
  let mut all_futures = Vec::new();

  for step in &query_plan.parallel_steps {
    match step {
      Parallel::Sequential(query_steps) => {
        let future = execute_sequential(client, query_steps, supergraph);
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
  client: &conductor_tracing::reqwest_utils::TracedHttpClient,
  query_steps: &Vec<QueryStep>,
  supergraph: &Supergraph,
) -> Result<Vec<((String, String), QueryResponse)>> {
  let mut data_vec = vec![];
  let mut entity_arguments: Option<SerdeValue> = None;

  for (i, query_step) in query_steps.iter().enumerate() {
    let data = execute_query_step(client, query_step, supergraph, entity_arguments.clone()).await;

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
                    entity_arguments = Some(SerdeValue::from(y));
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
        extensions: None,
      };
      (plan_meta, new_response)
    })
    .collect::<Vec<((std::string::String, std::string::String), QueryResponse)>>();

  Ok(x)
}

fn find_objects_matching_criteria(
  json: &SerdeValue,
  typename: &str,
  field: &str,
) -> Vec<SerdeValue> {
  let mut matching_objects = Vec::new();

  match json {
    SerdeValue::Object(map) => {
      if let (Some(typename_value), Some(field_value)) = (
        map.get("__typename").and_then(|v| v.as_str()),
        map.get(field),
      ) {
        if typename_value == typename {
          let mut result = serde_json::Map::new();
          result.insert(
            "__typename".to_string(),
            SerdeValue::String(typename.to_string()),
          );
          result.insert(field.to_string(), field_value.clone());
          matching_objects.push(SerdeValue::Object(result));
        }
      }
      for (_, value) in map {
        matching_objects.extend(find_objects_matching_criteria(value, typename, field));
      }
    }
    SerdeValue::Array(arr) => {
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
  pub data: Option<SerdeValue>,
  pub errors: Option<Vec<SerdeValue>>,
  pub extensions: Option<SerdeValue>,
}

fn dynamically_build_schema_from_supergraph(supergraph: &Supergraph) -> Schema {
  let mut query = Object::new("Query");

  // Dynamically create object types and fields
  for (type_name, graphql_type) in &supergraph.types {
    let mut obj = Object::new(type_name);

    for field_name in graphql_type.fields.keys() {
      let field_type = TypeRef::named_nn(TypeRef::STRING); // Adjust based on `field.field_type`
      obj = obj.field(Field::new(field_name, field_type, move |_| {
        let future: Pin<Box<dyn Future<Output = Result<Option<FieldValue>, Error>> + Send>> =
          Box::pin(async move {
            Ok(Some(FieldValue::from(Value::String(
              "Dynamic value".to_string(),
            ))))
          });
        FieldFuture::new(future)
      }));
    }

    // Adjust the creation of Object TypeRef
    // Placeholder logic - replace with the correct object creation
    let obj_type_ref = TypeRef::named(TypeRef::STRING); // This needs to be correctly set
    query = query.field(Field::new(type_name, obj_type_ref, move |_| {
      let future: Pin<Box<dyn Future<Output = Result<Option<FieldValue>, Error>> + Send>> =
        Box::pin(async move { Ok(Some(FieldValue::from(Value::Object(Default::default())))) });
      FieldFuture::new(future)
    }));
  }

  // Construct and return the schema

  Schema::build("Query", None, None)
    .register(query)
    .finish()
    .expect("Introspection schema build failed")
}

async fn execute_query_step(
  client: &conductor_tracing::reqwest_utils::TracedHttpClient,
  query_step: &QueryStep,
  supergraph: &Supergraph,
  entity_arguments: Option<SerdeValue>,
) -> Result<QueryResponse> {
  let is_introspection = query_step.service_name == CONDUCTOR_INTERNAL_SERVICE_RESOLVER;

  if is_introspection {
    let schema = dynamically_build_schema_from_supergraph(supergraph);

    // Execute the introspection query
    // TODO: whenever excuting a query step, we need to take the query out of the step's struct instead of copying it
    let request = async_graphql::Request::new(query_step.query.to_string());
    let response = schema.execute(request).await;

    let data = serde_json::to_value(response.data)?;
    let errors = response
      .errors
      .iter()
      .map(|e| serde_json::to_value(e).unwrap())
      .collect();

    Ok(QueryResponse {
      data: Some(data),
      errors: Some(errors),
      extensions: None,
    })
  } else {
    let span_name = format!("subgraph {}", query_step.service_name);
    let span = tracing::info_span!("subgraph_request", "otel.name" = span_name, service_name = %query_step.service_name, "graphql.document" = query_step.query);
    let url = supergraph.subgraphs.get(&query_step.service_name).unwrap();

    let variables_object = if let Some(arguments) = &entity_arguments {
      serde_json::json!({ "representations": arguments })
    } else {
      SerdeValue::Object(serde_json::Map::new())
    };

    // TODO: improve this by implementing https://github.com/the-guild-org/conductor-t2/issues/205
    let response = match client
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
      .instrument(span)
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
}
