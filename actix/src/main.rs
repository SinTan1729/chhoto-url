// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};
use rusqlite::Connection;
use std::{env, io::Result};

// Import modules
mod auth;
mod database;
mod services;
mod utils;

// This struct represents state
struct AppState {
    db: Connection,
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));

    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();

    let db_location = env::var("db_url")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(String::from("urls.sqlite"));

    // Get the port environment variable
    let port = env::var("port")
        .unwrap_or(String::from("4567"))
        .parse::<u16>()
        .expect("Supplied port is not an integer");

    let cache_control_header = env::var("cache_control_header")
        .ok()
        .filter(|s| !s.trim().is_empty());

    // If an API key is set, check the security
    if let Ok(key) = env::var("api_key") {
        if !auth::is_key_secure() {
            eprintln!("WARN: API key is insecure! Please change it. Current key is: {}. Generated secure key which you may use: {}", key, auth::gen_key())
        } else {
            eprintln!("Secure API key was provided.")
        }
    }

    // If the site_url env variable exists
    if let Some(site_url) = env::var("site_url").ok().filter(|s| !s.trim().is_empty()) {
        // Get first and last characters of the site_url
        let mut chars = site_url.chars();
        let first = chars.next();
        let last = chars.next_back();
        let url = chars.as_str();
        // If the site_url is encapsulated by quotes (i.e. invalid)
        if first == Option::from('"') || first == Option::from('\'') && first == last {
            // Set the site_url without the quotes
            env::set_var("site_url", url);
            eprintln!("WARN: The site_url environment variable is encapsulated by quotes. Automatically adjusting to {}", url);

            // Tell the user what URI the server will respond with
            eprintln!("INFO: Public URI is: {url}:{port}.")
        } else {
            // No issues
            eprintln!("INFO: Configured Site URL is: {site_url}.");

            // Tell the user what URI the server will respond with
            eprintln!("INFO: Public URI is: {site_url}:{port}.")
        }
    } else {
        // Site URL is not configured
        eprintln!("WARN: The site_url environment variable is not configured. Defaulting to http://localhost");
        eprintln!("INFO: Public URI is: http://localhost:{port}.")
    }

    // Tell the user that the server has started, and where it is listening to, rather than simply outputting nothing
    eprintln!("Server has started at 0.0.0.0 on port 4567.");

    // Actually start the server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .cookie_secure(false)
                    .build(),
            )
            // Maintain a single instance of database throughout
            .app_data(web::Data::new(AppState {
                db: database::open_db(db_location.clone()),
            }))
            .wrap(if let Some(header) = &cache_control_header {
                middleware::DefaultHeaders::new().add(("Cache-Control", header.to_owned()))
            } else {
                middleware::DefaultHeaders::new()
            })
            .service(services::link_handler)
            .service(services::getall)
            .service(services::siteurl)
            .service(services::version)
            .service(services::add_link)
            .service(services::delete_link)
            .service(services::login)
            .service(services::logout)
            .service(services::expand)
            .service(Files::new("/", "./resources/").index_file("index.html"))
            .default_service(actix_web::web::get().to(services::error404))
    })
    // Hardcode the port the server listens to. Allows for more intuitive Docker Compose port management
    .bind(("0.0.0.0", 4567))?
    .run()
    .await
}
