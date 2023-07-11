use std::{
    collections::{hash_map::RandomState, HashMap},
    sync::{Arc, Mutex},
};

use async_graphql::{
    dynamic::Schema as DynamicSchema,
    http::parse_query_string,
    parser::{parse_query, parse_schema},
    EmptyMutation, EmptySubscription, Schema, SchemaBuilder,
};
use async_graphql_axum::GraphQLRequest;
use hyper::{body::to_bytes, http, Body, Client, Request, Response};

pub async fn fetch_from_source(
    source_url: String,
    headers: Option<HashMap<String, String, RandomState>>,
    // introspection_results: Arc<Mutex<HashMap<String, DynamicSchema>>>,
) {
    // Create a new Hyper client
    let http_client = Client::new();

    // Prepare the request
    let mut request_builder = Request::builder()
        .method(http::Method::POST)
        .uri(source_url);

    // Add headers if provided
    if let Some(headers) = headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }

    let request = request_builder.body(Body::empty()).unwrap(); // You can handle unwrap errors as per your requirement

    // Send the request and await the response
    let response = http_client.request(request).await.unwrap(); // Handle request errors as per your requirement

    // Read the response body
    let response_bytes = to_bytes(response.into_body()).await.unwrap(); // Handle read errors as per your requirement

    // Convert the response body to a string
    let response_string = String::from_utf8(response_bytes.to_vec()).unwrap(); // Handle conversion errors as per your requirement

    println!("{}", response_string);
    // Build the schema from the response string (assuming it's a valid GraphQL SDL)
    // let schema = ;

    // introspection_results
    //     .lock()
    //     .unwrap()
    //     .insert(source_url.to_string(), schema);
}

pub async fn fetch_from_json(client: &str, config: &str) {
    // Your code here.
}

async fn body_to_string(req: Response<Body>) -> String {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    String::from_utf8(body_bytes.to_vec()).unwrap()
}
