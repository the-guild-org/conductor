mod query_builder;
mod query_planner;
mod supergraph;
mod user_query;

use std::fs;

use crate::{
    query_planner::plan_for_user_query, supergraph::parse_supergraph, user_query::parse_user_query,
};

fn main() {
    let query = fs::read_to_string("./query.graphql").unwrap();
    let supergraph_schema = fs::read_to_string("./supergraph.graphql").unwrap();

    let user_query = parse_user_query(&query);
    let supergraph = parse_supergraph(&supergraph_schema).unwrap();

    let plan = plan_for_user_query(&supergraph, &user_query);
    println!("Final QueryPlan: {:#?}", plan);

    // println!("Supergraph {:#?}", supergraph);
    // println!("User query: {:#?}", user_query);
}
