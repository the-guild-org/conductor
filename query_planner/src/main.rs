use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    sync::Arc,
    vec,
};

use async_graphql::{indexmap::Equivalent, InputType};
use graphql_parser::{
    parse_query, parse_schema,
    query::{Definition, Field, OperationDefinition, Selection},
    schema::{Definition as SchemaDefinition, TypeDefinition, Value},
};
use query_planner::{parse_supergraph, Supergraph};

fn main() {
    let query = fs::read_to_string("./query.graphql").unwrap();
    let supergraph_schema = fs::read_to_string("./supergraph.graphql").unwrap();

    let user_query = parse_user_query(&query);
    let supergraph = parse_supergraph(&supergraph_schema).unwrap();

    let plan = plan_for_user_query(&supergraph, &user_query);
    println!("Final QueryPlan: {:#?}", plan);

    // println!("Supergraph {:#?}", supergraph);
    // println!("User query: {:#?}", user_query);
    // query_planner.plan_query();
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

fn plan_for_field<'a, 'b>(
    supergraph: &'a Supergraph,
    field: &'b FieldNode<'a>,
    service_fields_map: &mut HashMap<String, Vec<&'b FieldNode<'a>>>,
    parent_type: &'a str,
) {
    // Debugging output
    println!(
        "Processing field: {} with parent type: {}",
        field.field, parent_type
    );

    match supergraph.types.get(parent_type) {
        Some(parent_data) => {
            if let Some(field_data) = parent_data.fields.get(&field.field) {
                if field.field == "reviews" {
                    // For the "reviews" field, we need to handle it differently.
                    if let Some(review_type_data) = supergraph.types.get("Review") {
                        if let Some(review_fields) = review_type_data.fields.get(&field.field) {
                            service_fields_map
                                .entry(review_fields.source.clone())
                                .or_insert_with(Vec::new)
                                .push(field);
                            return;
                        }
                    }
                    println!(
                        "Failed to find the source service for field {}",
                        field.field
                    );
                    return;
                } else {
                    service_fields_map
                        .entry(field_data.source.clone())
                        .or_insert_with(Vec::new)
                        .push(field);
                }

                // For object fields, fetch the unwrapped type for potential children
                let child_parent_type = unwrap_graphql_type(&field_data.field_type);

                // Recurse for nested fields
                for child_field in &field.children {
                    plan_for_field(
                        supergraph,
                        child_field,
                        service_fields_map,
                        child_parent_type,
                    );
                }
            }
        }
        None => {
            println!("Parent type: {} not found in supergraph.", parent_type);
        }
    }
}

fn plan_for_user_query<'a>(supergraph: &'a Supergraph, user_query: &'a UserQuery<'a>) -> QueryPlan {
    let mut service_fields_map: HashMap<String, Vec<&FieldNode>> = HashMap::new();

    // Process the fields and map them to their services.
    for field in &user_query.fields {
        plan_for_field(supergraph, field, &mut service_fields_map, "Query");
    }

    // Convert the service_fields_map into QuerySteps.
    let mut steps: Vec<QueryStep> = Vec::new();

    for (_, fields) in service_fields_map {
        let field_name = &fields[0].field;
        let field_type = match supergraph.types.get("Query") {
            Some(query_type) => match query_type.fields.get(field_name) {
                Some(field_data) => &field_data.field_type,
                None => match &fields[0].alias {
                    Some(alias) => match query_type.fields.get(alias) {
                        Some(field_data) => &field_data.field_type,
                        None => {
                            println!("Field {} not found in supergraph's Query type", field_name);
                            continue;
                        }
                    },
                    None => {
                        println!("Field {} not found in supergraph's Query type", field_name);
                        continue;
                    }
                },
            },
            None => {
                println!("Query type not found in supergraph");
                continue;
            }
        };

        let additional_steps = plan_for_fields_of_type(supergraph, field_type, fields);
        steps.extend(additional_steps);
    }

    // Create the final QueryPlan.
    QueryPlan {
        steps: vec![ExecutionStep::Sequential(steps)],
    }
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

fn plan_for_fields_of_type<'a>(
    supergraph: &'a Supergraph,
    field_type: &'a str,
    fields: Vec<&'a FieldNode<'a>>,
) -> Vec<QueryStep> {
    let unwrapped_type = unwrap_graphql_type(field_type);

    let mut service_to_fields: HashMap<String, Vec<&FieldNode>> = HashMap::new();

    for field in fields {
        let service_name = match supergraph.types.get(unwrapped_type) {
            Some(type_data) => match type_data.fields.get(&field.field) {
                Some(field_data) => field_data.source.clone(),
                None => {
                    println!(
                        "Failed to find the source service for field {}",
                        field.field
                    );
                    continue;
                }
            },
            None => {
                println!("{} type not found in supergraph", unwrapped_type);
                continue;
            }
        };

        service_to_fields
            .entry(service_name)
            .or_default()
            .push(field);
    }

    let mut sequential_steps = Vec::new();

    for (service_name, fields) in service_to_fields {
        let query_content: String = fields
            .iter()
            .map(|&field| field_node_to_string(field))
            .collect::<Vec<String>>()
            .join(" ");

        let has_root_query_field = fields.iter().any(|&field| {
            let field_name = match &field.alias {
                Some(alias) => alias,
                None => &field.field,
            };
            supergraph.types.get("Query").map_or(false, |query_type| {
                query_type.fields.contains_key(field_name)
            })
        });

        let full_query = if has_root_query_field {
            format!("query {{ {} }}", query_content.trim())
        } else if service_name == "REVIEWS" {
            format!(
                "query($representations: [_Any!]!) {{ _entities(representations: $representations) {{ ... on Review {{ {} }} }} }}",
                query_content
            )
        } else {
            format!(
                "query($representations: [_Any!]!) {{ _entities(representations: $representations) {{ ... on {} {{ {} }} }} }}",
                unwrapped_type,
                query_content
            )
        };

        let step = QueryStep {
            service_name,
            query: full_query,
        };

        sequential_steps.push(step);
    }

    sequential_steps
}

fn stringify_arguments<'a>(arguments: &Vec<(String, Value<'a, String>)>) -> String {
    let mut result = String::new();
    for (name, value) in arguments {
        result.push_str(&format!("{}: {}, ", name, value));
    }
    result.trim_end_matches(", ").to_string()
}

// Recursive function to convert FieldNode to a GraphQL query string
fn field_node_to_string<'a>(field_node: &FieldNode<'a>) -> String {
    let mut result = String::new();
    if let Some(alias) = &field_node.alias {
        result.push_str(&format!("{}: ", alias));
    }
    result.push_str(&field_node.field);
    if !field_node.arguments.is_empty() {
        result.push_str(&format!("({})", stringify_arguments(&field_node.arguments)));
    }
    if !field_node.children.is_empty() {
        result.push_str(" {");
        for child in &field_node.children {
            result.push_str(&field_node_to_string(child));
        }
        result.push_str("}");
    }
    result.push_str(" ");
    result
}

// Function to convert UserQuery to a GraphQL query string
pub fn user_query_to_string<'a>(user_query: &UserQuery<'a>) -> String {
    let operation_type_str = match user_query.operation_type {
        OperationType::Query => "query",
        OperationType::Mutation => "mutation",
        OperationType::Subscription => "subscription",
    };

    let mut result = String::new();
    if let Some(operation_name) = &user_query.operation_name {
        result.push_str(&format!("{} {} ", operation_type_str, operation_name));
    } else {
        result.push_str(&format!("{} ", operation_type_str));
    }

    // if !user_query.arguments.is_empty() {
    //     result.push_str(&format!("({})", stringify_arguments(&user_query.arguments)));
    // }

    result.push_str(" {");
    for field_node in &user_query.fields {
        result.push_str(&field_node_to_string(field_node));
    }
    result.push_str("}");

    result
}

#[derive(Debug, Clone)]
pub struct FieldNode<'a> {
    pub field: String,
    pub alias: Option<String>,
    pub arguments: Vec<(String, Value<'a, String>)>,
    pub children: Vec<FieldNode<'a>>,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug)]
pub struct UserQuery<'a> {
    pub operation_type: OperationType,
    pub operation_name: Option<String>,
    pub arguments: Vec<(String, String, Option<Value<'a, String>>)>,
    pub fields: Vec<FieldNode<'a>>,
}

fn parse_user_query<'a>(query: &'a str) -> UserQuery<'a> {
    let parsed_query = parse_query::<String>(&query);

    let mut user_query = UserQuery {
        operation_name: None,
        operation_type: OperationType::Query,
        arguments: vec![],
        fields: vec![],
    };

    match parsed_query {
        Ok(query) => {
            for definition in &query.definitions {
                match definition {
                    Definition::Operation(OperationDefinition::Query(q)) => {
                        user_query.operation_type = OperationType::Query;
                        user_query.operation_name = q.name.clone();

                        user_query.arguments = q
                            .variable_definitions
                            .iter()
                            .map(|e| {
                                (
                                    e.name.to_string(),
                                    e.var_type.to_string(),
                                    e.default_value.to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(q.selection_set.clone()));
                    }
                    Definition::Operation(OperationDefinition::Mutation(m)) => {
                        user_query.operation_type = OperationType::Mutation;
                        user_query.operation_name = m.name.clone();

                        user_query.arguments = m
                            .variable_definitions
                            .iter()
                            .map(|e| {
                                (
                                    e.name.to_string(),
                                    e.var_type.to_string(),
                                    e.default_value.to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(m.selection_set.clone()));
                    }
                    Definition::Operation(OperationDefinition::Subscription(s)) => {
                        user_query.operation_type = OperationType::Subscription;
                        user_query.operation_name = s.name.clone();

                        user_query.arguments = s
                            .variable_definitions
                            .iter()
                            .map(|e| {
                                (
                                    e.name.to_string(),
                                    e.var_type.to_string(),
                                    e.default_value.to_owned(),
                                )
                            })
                            .collect::<Vec<_>>();

                        user_query
                            .fields
                            .extend(handle_selection_set(s.selection_set.clone()));
                    }
                    Definition::Operation(OperationDefinition::SelectionSet(q)) => {
                        user_query.fields = handle_selection_set(q.clone());
                    }
                    _ => {}
                }
            }
        }
        Err(e) => println!("Failed to parse the query: {:#?}", e),
    }

    user_query
}

fn handle_selection_set<'a>(
    selection_set: graphql_parser::query::SelectionSet<'a, String>,
) -> Vec<FieldNode<'a>> {
    let mut fields = Vec::new();

    for selection in &selection_set.items {
        if let Selection::Field(Field {
            name,
            selection_set: field_selection_set,
            arguments,
            alias,
            ..
        }) = selection
        {
            let children = handle_selection_set(field_selection_set.clone());

            let x = arguments
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            fields.push(FieldNode {
                field: name.to_string(),
                children,
                alias: alias.to_owned(),
                arguments: x,
            });
        } else if let Selection::Field(Field { name, .. }) = selection {
            fields.push(FieldNode {
                field: name.to_string(),
                children: Vec::new(),
                alias: None,
                arguments: Vec::new(),
            });
        }
    }

    fields
}

#[derive(Debug, Clone)]
pub struct QueryStep {
    pub service_name: String,
    pub query: String,
}

#[derive(Debug)]
pub enum ExecutionStep {
    Parallel(Vec<QueryStep>),
    Sequential(Vec<QueryStep>),
}

#[derive(Debug)]
pub struct QueryPlan {
    pub steps: Vec<ExecutionStep>,
}

pub struct QueryPlanner<'a> {
    supergraph: &'a Supergraph,
    plan: QueryPlan,
    user_query: &'a [FieldNode<'a>], // Store the user_query here
}

#[derive(Debug)]
pub struct QueryNode<'a> {
    pub fields: Vec<FieldNode<'a>>,
}

// impl<'a> QueryPlanner<'a> {
//     pub fn new(supergraph: &'a HashMap<String, Subgraph>, user_query: &'a [FieldNode<'a>]) -> Self {
//         QueryPlanner {
//             supergraph,
//             plan: QueryPlan { steps: Vec::new() },
//             user_query, // Store the user_query here
//         }
//     }

//     fn plan_field(
//         &mut self,
//         node: &'a FieldNode<'a>,
//         parent_service: Option<&str>,
//     ) -> Option<QueryStep<'a>> {
//         if let Some((service_name, _)) = self.supergraph.iter().find(|(name, subgraph)| {
//             if let Some(types) = subgraph.types.get("Query") {
//                 types.fields.contains_key(&node.field)
//             } else {
//                 false
//             }
//         }) {
//             // Create a new QueryStep for this field
//             let mut step = QueryStep {
//                 service_name: service_name.to_string(),
//                 field: Arc::new(node.clone()),
//                 sub_query: Vec::new(),
//             };

//             // Plan each nested field
//             for child in &node.children {
//                 if let Some(sub_step) = self.plan_field(child, Some(service_name)) {
//                     step.sub_query.push(sub_step);
//                 }
//             }

//             // Plan required fields for this service
//             self.plan_required_fields(service_name, node);

//             // Determine whether this step can be executed in parallel or sequentially
//             if let Some(parent) = parent_service {
//                 if parent == service_name {
//                     if let Some(ExecutionStep::Parallel(steps))
//                     | Some(ExecutionStep::Sequential(steps)) = self.plan.steps.last_mut()
//                     {
//                         steps.push(step);
//                     } else {
//                         self.plan.steps.push(ExecutionStep::Parallel(vec![step]));
//                     }
//                     return None;
//                 }
//             }

//             Some(step)
//         } else {
//             // Handle the field not found case here, such as returning an error or skipping the field.
//             // For demonstration purposes, let's skip the field.
//             None
//         }
//     }

//     fn plan_query(&mut self) {
//         for field_node in self.user_query {
//             self.plan_field(field_node, None);
//         }
//     }

//     fn plan_required_fields(&mut self, service: &'a str, node: &'a FieldNode<'a>) {
//         if let Some(subgraph) = self.supergraph.get(service) {
//             if let Some(types) = subgraph.types.get("Query") {
//                 if let Some(field_details) = types.fields.get(&node.field) {
//                     if let Some(required_field_name) = &field_details.requires {
//                         if let Some(required_field_node) = self
//                             .user_query
//                             .iter()
//                             .find(|n| n.field == *required_field_name)
//                         {
//                             if let Some(step) = self.plan_field(required_field_node, Some(service))
//                             {
//                                 // If the step already exists in the plan, add the required field to its sub-query
//                                 if let Some(ExecutionStep::Parallel(steps))
//                                 | Some(ExecutionStep::Sequential(steps)) =
//                                     self.plan.steps.last_mut()
//                                 {
//                                     if let Some(existing_step) = steps
//                                         .iter_mut()
//                                         .find(|s| s.service_name == step.service_name)
//                                     {
//                                         existing_step.sub_query.push(step);
//                                         return;
//                                     }
//                                 }

//                                 // If the step doesn't exist in the plan, create a new step with the required field
//                                 let new_step = ExecutionStep::Parallel(vec![step]);
//                                 self.plan.steps.push(new_step);
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// fn generate_query_plan<'a>(
//     parsed_query: &[FieldNode<'a>],
//     parsed_supergraph: &Supergraph,
// ) -> QueryPlan<'a> {
//     let mut query_plan = QueryPlan { steps: Vec::new() };

//     // Iterate over all top-level fields in the query
//     for field_node in parsed_query {
//         // Recursively process the field and its children
//         process_field_node(
//             field_node.clone(),
//             &parsed_supergraph,
//             &mut query_plan,
//             None,
//         );
//     }

//     query_plan
// }

// fn process_field_node<'a>(
//     field_node: FieldNode<'a>,
//     parsed_supergraph: &Supergraph,
//     query_plan: &mut QueryPlan<'a>,
//     parent_service: Option<&String>,
// ) {
//     let field_name = field_node.field.clone();

//     // Determine which service is responsible for this field
//     let service_name_for_field =
//         parsed_supergraph
//             .iter()
//             .find_map(|(service_name, service_details)| {
//                 for (_type_name, type_fields) in &service_details.types {
//                     if type_fields.fields.contains(&field_name) {
//                         return Some(service_name);
//                     }
//                 }
//                 None
//             });

//     match service_name_for_field {
//         Some(service_name) if Some(service_name) == parent_service => {
//             // If the service for this field is the same as the parent field's service,
//             // just add it to the last query step
//             if let Some(ExecutionStep::Parallel(steps)) | Some(ExecutionStep::Sequential(steps)) =
//                 query_plan.steps.last_mut()
//             {
//                 if let Some(last_step) = steps.last_mut() {
//                     last_step.query_fields.push(field_node.clone());
//                 }
//             }
//         }
//         Some(service_name) => {
//             // If the service is different, or if this is the first field, create a new query step
//             let service_details = &parsed_supergraph[service_name];
//             let new_step = QueryStep {
//                 service_name: service_name.clone(),
//                 service_url: service_details.url.clone(),
//                 query_fields: vec![field_node.clone()],
//             };

//             match query_plan.steps.last_mut() {
//                 Some(ExecutionStep::Parallel(steps)) => {
//                     if steps.last().unwrap().service_name == new_step.service_name {
//                         steps.push(new_step);
//                     } else {
//                         query_plan
//                             .steps
//                             .push(ExecutionStep::Parallel(vec![new_step]));
//                     }
//                 }
//                 Some(ExecutionStep::Sequential(steps)) => {
//                     if steps.last().unwrap().service_name == new_step.service_name {
//                         steps.push(new_step);
//                     } else {
//                         query_plan
//                             .steps
//                             .push(ExecutionStep::Sequential(vec![new_step]));
//                     }
//                 }
//                 None => {
//                     query_plan
//                         .steps
//                         .push(ExecutionStep::Parallel(vec![new_step]));
//                 }
//             }
//         }
//         None => {
//             // If we couldn't find a service for this field, we'll skip it for now
//             // (you might want to handle this case differently, e.g., by returning an error)
//         }
//     }

//     // Process children fields
//     for child_node in &field_node.children {
//         process_field_node(
//             child_node.clone(),
//             parsed_supergraph,
//             query_plan,
//             service_name_for_field,
//         );
//     }
// }
