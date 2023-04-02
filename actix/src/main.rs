use actix_files::Files;
use actix_web::{get, web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(greet)
            .service(Files::new("/", "./resources/").index_file("index.html"))
    })
    .bind(("127.0.0.1", 2000))?
    .run()
    .await
}
