use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
};

use crate::{
    supergraph::{GraphQLType, Supergraph},
    user_query::{FieldNode, UserQuery},
};

#[derive(Debug, Clone)]
pub struct QueryStep {
    pub service_name: String,
    pub query: String,
    pub arguments: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub enum ExecutionItem {
    SingleQuery(QueryStep),
    SeqQueries(Vec<QueryStep>),
}

#[derive(Debug)]
pub enum ExecutionStep {
    Parallel(Vec<ExecutionItem>),
    Sequential(Vec<QueryStep>),
}

#[derive(Debug)]
pub struct QueryPlan {
    pub steps: Vec<ExecutionStep>,
}

pub fn plan_for_user_query<'a>(supergraph: &Supergraph, user_query: &UserQuery<'a>) -> QueryPlan {
    let mut steps_in_parallel: Vec<ExecutionItem> = vec![];

    println!("{:#?}", user_query.fields);
    for field in &user_query.fields {
        let mut seq_steps: Vec<QueryStep> = vec![];

        println!("Processing field: {}", field.field);

        // // Check if the main field is external or requires another service
        // if let Some((service_name, id_field)) = get_service_and_key_for_field(field, supergraph) {
        //     let entities_query = generate_entities_query(field, id_field, supergraph);
        //     seq_steps.push(QueryStep {
        //         service_name,
        //         query: entities_query,
        //         arguments: None,
        //     });
        // } else {
        // Main field is not external, so generate a direct query
        let fields = get_direct_local_subfields_as_str(field, supergraph);

        for (service_name, fields) in fields {
            seq_steps.push(QueryStep {
                service_name,
                query: format!("{} {{ {} }}", field.field, fields.join(" ")),
                arguments: None,
            });
        }

        // println!("Main query for field {}: {}", field.field, main_query());

        // if let Some(service_name) = get_service_for_field(field, supergraph) {
        //     println!("Determined service for {}: {}", field.field, service_name);
        //     seq_steps.push(QueryStep {
        //         service_name,
        //         query: main_query(),
        //         arguments: None,
        //     });
        // }
        // }

        // Handle subfields
        let mut service_to_entities_query: HashMap<String, Vec<String>> = HashMap::new();

        for subfield in &field.children {
            println!("Processing subfield: {}", subfield.field.to_string());

            if let Some((service_name, id_field)) =
                get_service_and_key_for_field(subfield, supergraph)
            {
                // Check if this subfield is external or requires another service
                let query = generate_entities_query(subfield, id_field, supergraph);
                println!(
                    "Generated query for {}: {}",
                    subfield.field.to_string(),
                    query
                );
                service_to_entities_query
                    .entry(service_name)
                    .or_default()
                    .push(query);
            } else {
                println!(
                    "Could not determine service for {}",
                    subfield.field.to_string()
                );
            }
        }

        for (service, queries) in service_to_entities_query {
            for query in queries {
                seq_steps.push(QueryStep {
                    service_name: service.clone(),
                    query,
                    arguments: None,
                });
            }
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
    field: &FieldNode<'a>,
    supergraph: &Supergraph,
) -> HashMap<String, Vec<String>> {
    let field_type = match get_type_of_field(&field.field, supergraph) {
        Some(ft) => ft,
        None => return HashMap::new(),
    };

    let mut fields: HashMap<String, Vec<String>> = HashMap::new();

    // TODO: construct graphql queries to each service for each field
    // TODO: nested fields are still an issue to be implemented
    for subfield in &field.children {
        if let Some(field_def) = field_type.fields.get(&subfield.field) {
            // if !field_def.external {
            let existing = fields.get_mut(&field_def.source);
            let val = {
                if subfield.children.is_empty() {
                    subfield.field.clone()
                } else {
                    format!(
                        "{} {{ {} }}",
                        subfield.field,
                        get_direct_subfields_as_str(subfield, supergraph)
                    )
                }
            };

            if existing.is_some() {
                existing.unwrap().push(val);
            } else {
                fields.insert(field_def.source.clone(), vec![val]);
            }
            // }
        }
    }

    fields
}

// Adjusted this function to ignore external fields when gathering direct subfields
fn get_direct_subfields_as_str<'a>(field: &FieldNode<'a>, supergraph: &Supergraph) -> String {
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
fn get_service_for_field<'a>(field: &FieldNode<'a>, supergraph: &Supergraph) -> Option<String> {
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
    field: &FieldNode<'a>,
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

// Given a field and its primary key, generate an _entities query for it
fn generate_entities_query<'a>(
    field: &FieldNode<'a>,
    id_fields: Vec<String>,
    supergraph: &Supergraph,
) -> String {
    let subfields_str = get_direct_subfields_as_str(field, supergraph);

    println!("Generating entities query for {}", field.field.as_str());

    format!(
        "query Query($representations: [_Any!]!) {{
        _entities(representations: $representations) {{
            ... on {} {{
                {}
            }}
        }}
    }}",
        capitalize_first_letter(&field.field),
        subfields_str
    )
}

fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(chars).collect(),
    }
}

fn get_type_of_field<'a>(
    field_name: &'a str,
    supergraph: &'a Supergraph,
) -> Option<&'a GraphQLType> {
    for (_, type_def) in &supergraph.types {
        if let Some(field_def) = type_def.fields.get(field_name) {
            return supergraph
                .types
                .get(unwrap_graphql_type(&field_def.field_type));
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

pub fn execute_query_plan(query_plan: &QueryPlan) {
    for step in &query_plan.steps {
        match step {
            ExecutionStep::Sequential(query_steps) => {
                for query_step in query_steps {
                    execute_query_step(query_step);
                }
            } // If there are other variants of ExecutionStep, handle them here.
            ExecutionStep::Parallel(step) => {}
        }
    }
}

fn execute_query_step(query_step: &QueryStep) {
    // Here's where the actual execution happens. For now, I'm just printing it out.
    // You'd replace this with an HTTP request or whatever your execution mechanism is.
    println!(
        "Executing query for service '{}':\n{}",
        query_step.service_name, query_step.query
    );

    // Mocking the result for now
    let result = "Some mocked result data...";

    println!(
        "Result from service '{}':\n{}",
        query_step.service_name, result
    );
}
