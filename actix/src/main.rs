use actix_files::{Files, NamedFile};
use actix_web::{
    get,
    web::{self, Redirect},
    App, HttpServer, Responder,
};
mod database;
mod utils;

// Define the routes

// Add new links

// Return all active links

// 404 error page
#[get("/err/404")]
async fn error404() -> impl Responder {
    NamedFile::open_async("./resources/404.html").await
}

// Handle a given shortlink
#[get("/{shortlink}")]
async fn link_handler(shortlink: web::Path<String>) -> impl Responder {
    let longlink = utils::get_longurl(shortlink);
    if longlink == String::from("") {
        Redirect::to("/err/404")
    } else {
        Redirect::to(longlink).permanent()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(link_handler)
            .service(error404)
            .service(Files::new("/", "./resources/").index_file("index.html"))
    })
    .bind(("0.0.0.0", 2000))?
    .run()
    .await
}
