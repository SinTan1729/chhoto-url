use std::env;

use actix_files::{Files, NamedFile};
use actix_web::{
    get, post,
    web::{self, Redirect},
    App, HttpResponse, HttpServer, Responder,
};
mod database;
mod utils;

// Define the routes

// Add new links
#[post("/api/new")]
async fn add_link(req: String) -> HttpResponse {
    let out = utils::add_link(req);
    if out.0 {
        println!("ok{}", out.1);
        HttpResponse::Ok().body(out.1)
    } else {
        println!("bad{}", out.1);
        HttpResponse::BadRequest().body(out.1)
    }
}

// Return all active links
#[get("/api/all")]
async fn getall() -> HttpResponse {
    HttpResponse::Ok().body(utils::getall())
}

// Get the site URL
#[get("/api/siteurl")]
async fn siteurl() -> HttpResponse {
    let site_url = env::var("site_url").unwrap_or("unset".to_string());
    HttpResponse::Ok().body(site_url)
}

// 404 error page
#[get("/err/404")]
async fn error404() -> impl Responder {
    NamedFile::open_async("./resources/404.html").await
}

// Handle a given shortlink
#[get("/{shortlink}")]
async fn link_handler(shortlink: web::Path<String>) -> impl Responder {
    let shortlink_str = shortlink.to_string();
    let longlink = utils::get_longurl(shortlink_str);
    if longlink == "".to_string() {
        database::add_hit(shortlink.as_str());
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
            .service(getall)
            .service(siteurl)
            .service(add_link)
            .service(Files::new("/", "./resources/").index_file("index.html"))
    })
    .bind(("0.0.0.0", 2000))?
    .run()
    .await
}
