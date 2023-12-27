use actix_web::http::StatusCode;
use actix_web::{guard, web, App, HttpResponse, HttpServer, Result};
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription};
use async_graphql::{Context, Object, Schema, ID};
use async_graphql_actix_web::{GraphQL, GraphQLRequest, GraphQLResponse};
use std::sync::Arc;

#[derive(Clone)]
struct Book {
  id: ID,
  name: String,
  num_pages: i32,
}

#[Object]
impl Book {
  async fn id(&self) -> &ID {
    &self.id
  }

  async fn name(&self) -> &str {
    &self.name
  }

  async fn num_pages(&self) -> i32 {
    self.num_pages
  }
}

#[derive(Clone)]
struct Author {
  id: ID,
  name: String,
  company: String,
  books: Vec<Book>,
}

#[Object]
impl Author {
  async fn id(&self) -> &ID {
    &self.id
  }

  async fn name(&self) -> &str {
    &self.name
  }

  async fn company(&self) -> &str {
    &self.company
  }

  async fn books(&self) -> &Vec<Book> {
    &self.books
  }
}

struct Query;

#[Object]
impl Query {
  async fn authors(&self, ctx: &Context<'_>) -> Vec<Author> {
    vec![Author {
      id: ID::from("1"),
      name: "someone".to_string(),
      company: "the-guild".to_string(),
      books: vec![Book {
        id: ID::from("101"),
        name: "Rust lang book".to_string(),
        num_pages: 223,
      }],
    }]
  }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(move || {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    App::new()
      .service(
        web::resource("/")
          .guard(guard::Post())
          .to(GraphQL::new(schema)),
      )
      .service(web::resource("/").guard(guard::Get()).to(index_graphiql))
      .route("/_health", web::get().to(health_check))
  })
  .bind("127.0.0.1:4000")?
  .run()
  .await
}

async fn index_graphiql() -> Result<HttpResponse> {
  Ok(
    HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(GraphiQLSource::build().endpoint("/").finish()),
  )
}

async fn health_check() -> Result<HttpResponse> {
  Ok(HttpResponse::new(StatusCode::OK))
}
