use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn baseline_handler() -> impl Responder {
  HttpResponse::Ok()
    .content_type("application/json")
    .body(r#"{ "data": { "hello": "world" } }"#)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| App::new().route("/baseline", web::get().to(baseline_handler)))
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
