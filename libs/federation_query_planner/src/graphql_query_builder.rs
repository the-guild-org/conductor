use std::collections::{HashMap, HashSet};

use crate::{
  query_planner::{contains_entities_query, EntityQueryNeeds},
  user_query::{FieldNode, OperationType, UserQuery},
};

pub fn generate_entities_query(typename: &str, selection_set: &str) -> String {
  assert!(
    !typename.is_empty(),
    "Typename of the parent field must not be empty when generating an _entity query!"
  );
  format!(
    "_entities(representations: $representations) {{ ... on {} {{ {} __typename }} }}",
    typename, selection_set
  )
}

pub fn generate_query_for_field(
  operation_type: String,
  sub_query: String,
  // arguments: Vec<QueryDefinedArgument>,
  // fragments: &Fragments,
) -> String {
  if contains_entities_query(&sub_query) {
    // TODO: clean this up
    format!(
      "{} Entity($representations: [_Any!]!) {{ {} }}",
      if operation_type.is_empty() {
        "query"
      } else {
        &operation_type
      },
      sub_query
    )
  } else {
    // let arguments = if !arguments.is_empty() {
    //     format!("({})", stringify_arguments(&arguments))
    // } else {
    //     String::new()
    // };
    // TODO: add arguments
    format!("{}{{ {} }}", operation_type, sub_query)
  }
}

// TODO: VERY EXPENSIVE, NOT GREAT
pub fn batch_subqueries(
  input_structures: Vec<(String, String, EntityQueryNeeds)>,
) -> Vec<(String, String, EntityQueryNeeds)> {
  let mut batched_subqueries: HashMap<String, (HashSet<String>, EntityQueryNeeds)> = HashMap::new();
  let mut order: Vec<String> = Vec::new();

  // Process each structure
  for (service, subquery, entity_map) in input_structures {
    let parts: Vec<&str> = subquery.split('#').collect();

    // If the structure matches the entity#fields format, process it
    if parts.len() == 2 {
      let entity_identifier = parts[0].trim().to_string();
      let fields = parts[1].trim().to_string();

      let key = format!("{}_{}", service, entity_identifier);
      if !batched_subqueries.contains_key(&key) {
        order.push(key.clone());
      }
      batched_subqueries
        .entry(key)
        .or_insert_with(|| (HashSet::new(), entity_map))
        .0
        .insert(fields);
    } else {
      if !batched_subqueries.contains_key(&service) {
        order.push(service.clone());
      }
      batched_subqueries
        .entry(service.clone())
        .or_insert_with(|| (HashSet::new(), entity_map))
        .0
        .insert(subquery);
    }
  }

  // Convert the HashMap into the desired Vec format based on order
  let mut results: Vec<(String, String, EntityQueryNeeds)> = Vec::new();
  for key in order {
    let (fields_set, entity_map) = batched_subqueries.remove(&key).unwrap();

    let service_parts: Vec<&str> = key.split('_').collect();
    let service = service_parts[0];
    let entity = if service_parts.len() > 1 {
      service_parts[1]
    } else {
      ""
    };

    if !entity.is_empty() {
      let batched_query = generate_entities_query(
        entity,
        &fields_set.iter().cloned().collect::<Vec<_>>().join(" "),
      );
      results.push((service.to_string(), batched_query, entity_map));
    } else {
      for field in fields_set {
        results.push((service.to_string(), field, entity_map.clone()));
      }
    }
  }

  results
}

pub fn batch_subqueries_in_user_query(user_query: &mut UserQuery) {
  // Recursively process the fields
  process_and_batch_subqueries(&mut user_query.fields);
}

fn process_and_batch_subqueries(fields: &mut [FieldNode]) {
  for entity_query in fields.iter_mut() {
    if let Some(relevant_sub_queries) = &entity_query.relevant_sub_queries {
      let mut batched_subqueries: HashMap<String, HashSet<String>> = HashMap::new();
      let mut order: Vec<String> = Vec::new();

      for (service, subquery) in relevant_sub_queries {
        let parts: Vec<&str> = subquery.split('#').collect();

        // If the structure matches the entity#fields format, process it
        if parts.len() == 2 {
          let entity_identifier = parts[0].trim().to_string();
          let fields = parts[1].trim().to_string();

          let key = format!("{}_{}", service, entity_identifier);
          if !batched_subqueries.contains_key(&key) {
            order.push(key.clone());
          }
          batched_subqueries.entry(key).or_default().insert(fields);
        } else {
          if !batched_subqueries.contains_key(service) {
            order.push(service.clone());
          }
          batched_subqueries
            .entry(service.clone())
            .or_default()
            .insert(subquery.to_string());
        }
      }

      // Convert the HashMap into the desired Vec format based on order
      let mut results: Vec<(String, String)> = Vec::new();
      for key in order.clone() {
        let fields_set = batched_subqueries.remove(&key).unwrap();

        let service_parts: Vec<&str> = key.split('_').collect();
        let service = service_parts[0];
        let entity = if service_parts.len() > 1 {
          service_parts[1]
        } else {
          ""
        };

        if !entity.is_empty() {
          let batched_query = generate_entities_query(
            entity,
            &fields_set.iter().cloned().collect::<Vec<_>>().join(" "),
          );
          results.push((
            service.to_string(),
            generate_query_for_field(OperationType::Query.to_string(), batched_query),
          ));
        } else {
          for field in fields_set {
            results.push((
              service.to_string(),
              generate_query_for_field(OperationType::Query.to_string(), field),
            ));
          }
        }
      }

      entity_query.relevant_sub_queries = Some(results);

      // Recursively process nested fields
      process_and_batch_subqueries(&mut entity_query.children);
    }
  }
}
