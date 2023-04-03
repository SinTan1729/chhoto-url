use std::env;

use actix_files::{Files, NamedFile};
use actix_web::{
    delete, get, post,
    web::{self, Redirect},
    App, HttpResponse, HttpServer, Responder,
};
use rusqlite::Connection;
mod database;
mod utils;

// This struct represents state
struct AppState {
    db: Connection,
}

// Define the routes

// Add new links
#[post("/api/new")]
async fn add_link(req: String, data: web::Data<AppState>) -> HttpResponse {
    let out = utils::add_link(req, &data.db);
    if out.0 {
        HttpResponse::Ok().body(out.1)
    } else {
        HttpResponse::BadRequest().body(out.1)
    }
}

// Return all active links
#[get("/api/all")]
async fn getall(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body(utils::getall(&data.db))
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
async fn link_handler(shortlink: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let shortlink_str = shortlink.to_string();
    let longlink = utils::get_longurl(shortlink_str, &data.db);
    if longlink == "".to_string() {
        Redirect::to("/err/404")
    } else {
        database::add_hit(shortlink.as_str(), &data.db);
        Redirect::to(longlink).permanent()
    }
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
async fn delete_link(shortlink: web::Path<String>, data: web::Data<AppState>) -> HttpResponse {
    database::delete_link(shortlink.to_string(), &data.db);
    HttpResponse::Ok().body("")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                db: database::open_db(env::var("db_url").unwrap_or("./urls.sqlite".to_string())),
            }))
            .service(link_handler)
            .service(error404)
            .service(getall)
            .service(siteurl)
            .service(add_link)
            .service(delete_link)
            .service(Files::new("/", "./resources/").index_file("index.html"))
    })
    .bind(("0.0.0.0", 2000))?
    .run()
    .await
}
