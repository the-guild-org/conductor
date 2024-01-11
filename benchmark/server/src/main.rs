use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn hello() -> impl Responder {
  HttpResponse::Ok().body("Hello!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
