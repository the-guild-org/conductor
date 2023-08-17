pub mod executor;
pub mod graphql_query_builder;
pub mod query_planner;
pub mod response_merge;
pub mod supergraph;
pub mod user_query;

#[tokio::test]
async fn generates_query_plan() {
    use crate::{
        executor::execute_query_plan, query_planner::plan_for_user_query,
        supergraph::parse_supergraph, user_query::parse_user_query,
    };
    use std::fs;

    let query = r#"query AYOOO($age: Int! = 5) {
    locations {
        name
        description:cool_description(wow: $age)
        reviews {
            comment
            rating
        }
        isCool {
            really {
                yes
            }
        }
    }
    location(id: "portugal") {
        photo
    }
}"#;

    let supergraph_schema = fs::read_to_string("./supergraph.graphql").unwrap();

    let supergraph = parse_supergraph(&supergraph_schema).unwrap();
    let user_query = parse_user_query(query);

    let query_plan = plan_for_user_query(&supergraph, &user_query);

    let response_vec = (execute_query_plan(&query_plan, &supergraph).await).unwrap_or_default();

    println!("Query Plan: {:#?}", query_plan);
    // println!("Response Vector: {:#?}", stringified_response_vec);

    insta::assert_json_snapshot!((user_query, supergraph, query_plan, response_vec));
}
