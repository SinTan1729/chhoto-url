use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    delete, get,
    http::StatusCode,
    middleware, post,
    web::{self, Redirect},
    App, HttpResponse, HttpServer, Responder,
};
use rusqlite::Connection;
use std::env;
mod auth;
mod database;
mod utils;

// This struct represents state
struct AppState {
    db: Connection,
}

// Define the routes

// Add new links
#[post("/api/new")]
async fn add_link(req: String, data: web::Data<AppState>, session: Session) -> HttpResponse {
    if auth::validate(session) {
        let out = utils::add_link(req, &data.db);
        if out.0 {
            HttpResponse::Ok().body(out.1)
        } else {
            HttpResponse::BadRequest().body(out.1)
        }
    } else {
        HttpResponse::Forbidden().body("logged_out")
    }
}

// Return all active links
#[get("/api/all")]
async fn getall(data: web::Data<AppState>, session: Session) -> HttpResponse {
    if auth::validate(session) {
        HttpResponse::Ok().body(utils::getall(&data.db))
    } else {
        HttpResponse::Forbidden().body("logged_out")
    }
}

// Get the site URL
#[get("/api/siteurl")]
async fn siteurl(session: Session) -> HttpResponse {
    if auth::validate(session) {
        let site_url = env::var("site_url").unwrap_or("unset".to_string());
        HttpResponse::Ok().body(site_url)
    } else {
        HttpResponse::Forbidden().body("logged_out")
    }
}

// 404 error page
#[get("/err/404")]
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
    let longlink = utils::get_longurl(shortlink_str, &data.db);
    if longlink == *"" {
        Redirect::to("/err/404")
    } else {
        let redirect_method = env::var("redirect_method").unwrap_or("PERMANENT".to_string());
        database::add_hit(shortlink.as_str(), &data.db);
        if redirect_method == *"TEMPORARY" {
            Redirect::to(longlink)
        } else {
            // Defaults to permanent redirection
            Redirect::to(longlink).permanent()
        }
    }
}

// Handle login
#[post("/api/login")]
async fn login(req: String, session: Session) -> HttpResponse {
    if req == env::var("password").unwrap_or(req.clone()) {
        // If no password was provided, match any password
        session.insert("session-token", auth::gen_token()).unwrap();
        HttpResponse::Ok().body("Correct password!")
    } else {
        eprintln!("Failed login attempt!");
        HttpResponse::Forbidden().body("Wrong password!")
    }
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
async fn delete_link(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
) -> HttpResponse {
    if auth::validate(session) {
        database::delete_link(shortlink.to_string(), &data.db);
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::Forbidden().body("Wrong password!")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));

    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();
    let db_location = env::var("db_url").unwrap_or("/urls.sqlite".to_string());
    let port = env::var("port")
        .unwrap_or("4567".to_string())
        .parse::<u16>()
        .expect("Supplied port is not an integer");

    // Actually start the server
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            // Maintain a single instance of database throughout
            .app_data(web::Data::new(AppState {
                db: database::open_db(env::var("db_url").unwrap_or(db_location.clone())),
            }))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(link_handler)
            .service(error404)
            .service(getall)
            .service(siteurl)
            .service(add_link)
            .service(delete_link)
            .service(login)
            .default_service(Files::new("/", "./resources/").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
