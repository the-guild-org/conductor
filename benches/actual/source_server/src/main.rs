use std::sync::Arc;

use actix_web::{guard, web, web::Data, App, HttpServer};
use async_graphql::{EmptyMutation, EmptySubscription, Schema, SimpleObject};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref COUNTRIES: Arc<Vec<Country>> = Arc::new(vec![
        Country {
            id: 1,
            code: String::from("US"),
            name: String::from("United States"),
            foundation_date: String::from("1776-07-04"),
            language: String::from("English"),
            avg_wage: 50000,
            most_popular_jobs: vec![
                String::from("Software Developer"),
                String::from("Physician"),
                String::from("Nurse"),
            ],
            popular_dishes: vec![
                PopularDish {
                    id: 1,
                    ingredients: vec![
                        String::from("Bread"),
                        String::from("Cheese"),
                        String::from("Ham"),
                    ],
                    name: String::from("Hamburger"),
                    price: 5,
                }
            ],
        },
        Country {
            id: 2,
            code: String::from("CA"),
            name: String::from("Canada"),
            foundation_date: String::from("1867-07-01"),
            language: String::from("English, French"),
            avg_wage: 52000,
            most_popular_jobs: vec![
                String::from("Software Developer"),
                String::from("Engineer"),
                String::from("Nurse"),
            ],
            popular_dishes: vec![
                PopularDish {
                    id: 2,
                    ingredients: vec![
                        String::from("Potato"),
                        String::from("Cheese"),
                        String::from("Gravy"),
                    ],
                    name: String::from("Poutine"),
                    price: 7,
                }
            ],
        },
        Country {
            id: 3,
            code: String::from("PT"),
            name: String::from("Portugal"),
            foundation_date: String::from("1143-10-05"),
            language: String::from("Portuguese"),
            avg_wage: 18000,
            most_popular_jobs: vec![
                String::from("Doctor"),
                String::from("Teacher"),
                String::from("Engineer"),
            ],
            popular_dishes: vec![
                PopularDish {
                    id: 3,
                    ingredients: vec![
                        String::from("Egg"),
                        String::from("Sugar"),
                        String::from("Cream"),
                    ],
                    name: String::from("Pastel de nata"),
                    price: 3,
                }
            ],
        },
    ]);
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct PopularDish {
    id: u32,
    name: String,
    ingredients: Vec<String>,
    price: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
struct Country {
    id: u32,
    name: String,
    code: String,
    language: String,
    foundation_date: String,
    avg_wage: u32,
    most_popular_jobs: Vec<String>,
    popular_dishes: Vec<PopularDish>,
}

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn country(&self, code: String) -> Option<Country> {
        // Access the shared list of countries
        let countries = COUNTRIES.clone();

        countries
            .iter()
            .find(|country| country.code == code)
            .cloned()
    }

    async fn countries(&self) -> Vec<Country> {
        // Access the shared list of countries
        let countries = COUNTRIES.clone();

        // Create a copy of the countries vector
        countries.to_vec()
    }
}

async fn index(
    schema: web::Data<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    println!("GraphiQL IDE: http://localhost:4000");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}
