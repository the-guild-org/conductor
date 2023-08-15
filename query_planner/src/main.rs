mod executor;
mod graphql_query_builder;
mod query_planner;
mod supergraph;
mod type_merge;
mod user_query;
use std::fs;

use crate::executor::execute_query_plan;

use crate::{
    query_planner::plan_for_user_query,
    supergraph::parse_supergraph,
    // type_merge::merge_data_based_on_query,
    user_query::parse_user_query,
};

#[tokio::main]
async fn main() {
    let query = fs::read_to_string("./query.graphql").unwrap();
    let supergraph_schema = fs::read_to_string("./supergraph.graphql").unwrap();

    let supergraph = parse_supergraph(&supergraph_schema).unwrap();
    let user_query = parse_user_query(&query);

    let query_plan = plan_for_user_query(&supergraph, &user_query);
    println!("{:#?}", query_plan);

    let response_vec = execute_query_plan(&query_plan, &supergraph)
        .await
        .unwrap_or_default();

    // println!("Response Vector: {:#?}", response_vec);

    // let mut final_response = merge_data_based_on_query(&user_query, &response_vec);

    // println!("Final Merged Response: {:#?}", final_response);

    // println!("Supergraph {:#?}", supergraph);
    // println!("User query: {:#?}", user_query);
}
