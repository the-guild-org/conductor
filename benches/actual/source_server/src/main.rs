use actix_web::{guard, web, web::Data, App, HttpServer};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Country {
    name: String,
    code: String,
}

#[async_graphql::Object]
impl Country {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn code(&self) -> &str {
        &self.code
    }
}

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn country(&self) -> Country {
        Country {
            code: String::from("EG"),
            name: String::from("Egypt"),
        }
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
