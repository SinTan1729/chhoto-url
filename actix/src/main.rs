use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    delete, get,
    http::StatusCode,
    middleware, post,
    web::{self, Redirect},
    App, Either, HttpResponse, HttpServer, Responder,
};
use rusqlite::Connection;
use std::{env, io::Result};
mod auth;
mod database;
mod utils;

// This struct represents state
struct AppState {
    db: Connection,
}

// Store the version number
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Define the routes

// Add new links
#[post("/api/new")]
async fn add_link(req: String, data: web::Data<AppState>, session: Session) -> HttpResponse {
    if auth::validate(session) {
        let out = utils::add_link(req, &data.db);
        if out.0 {
            HttpResponse::Created().body(out.1)
        } else {
            HttpResponse::Conflict().body(out.1)
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Return all active links
#[get("/api/all")]
async fn getall(data: web::Data<AppState>, session: Session) -> HttpResponse {
    if auth::validate(session) {
        HttpResponse::Ok().body(utils::getall(&data.db))
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Get the site URL
#[get("/api/siteurl")]
async fn siteurl(session: Session) -> HttpResponse {
    if auth::validate(session) {
        let site_url = env::var("site_url").unwrap_or(String::from("unset"));
        HttpResponse::Ok().body(site_url)
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Get the version number
#[get("/api/version")]
async fn version() -> HttpResponse {
    HttpResponse::Ok().body(VERSION)
}

// 404 error page
async fn error404() -> impl Responder {
    NamedFile::open_async("./resources/static/404.html")
        .await
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}

// Handle a given shortlink
#[get("/{shortlink}")]
async fn link_handler(shortlink: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let shortlink_str = shortlink.to_string();
    if let Some(longlink) = utils::get_longurl(shortlink_str, &data.db) {
        let redirect_method = env::var("redirect_method").unwrap_or(String::from("PERMANENT"));
        database::add_hit(shortlink.as_str(), &data.db);
        if redirect_method == "TEMPORARY" {
            Either::Left(Redirect::to(longlink))
        } else {
            // Defaults to permanent redirection
            Either::Left(Redirect::to(longlink).permanent())
        }
    } else {
        Either::Right(
            NamedFile::open_async("./resources/static/404.html")
                .await
                .customize()
                .with_status(StatusCode::NOT_FOUND),
        )
    }
}

// Handle login
#[post("/api/login")]
async fn login(req: String, session: Session) -> HttpResponse {
    if let Ok(password) = env::var("password") {
        if password != req {
            eprintln!("Failed login attempt!");
            return HttpResponse::Unauthorized().body("Wrong password!");
        }
    }
    // Return Ok if no password was set on the server side
    session
        .insert("chhoto-url-auth", auth::gen_token())
        .expect("Error inserting auth token.");
    HttpResponse::Ok().body("Correct password!")
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
async fn delete_link(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
) -> HttpResponse {
    if auth::validate(session) {
        if utils::delete_link(shortlink.to_string(), &data.db) {
            HttpResponse::Ok().body(format!("Deleted {shortlink}"))
        } else {
            HttpResponse::NotFound().body("Not found!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));

    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();
    let db_location = env::var("db_url").unwrap_or(String::from("/urls.sqlite"));
    let port = env::var("port")
        .unwrap_or(String::from("4567"))
        .parse::<u16>()
        .expect("Supplied port is not an integer");

    // Actually start the server
    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .cookie_secure(false)
                    .build(),
            )
            // Maintain a single instance of database throughout
            .app_data(web::Data::new(AppState {
                db: database::open_db(env::var("db_url").unwrap_or(db_location.clone())),
            }))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(link_handler)
            .service(getall)
            .service(siteurl)
            .service(version)
            .service(add_link)
            .service(delete_link)
            .service(login)
            .service(Files::new("/", "./resources/").index_file("index.html"))
            .default_service(web::get().to(error404))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
