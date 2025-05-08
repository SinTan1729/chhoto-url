// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};
use log::info;
use rusqlite::Connection;
pub(crate) use std::io::Result;
use tokio::{spawn, time};

// Import modules
mod auth;
mod config;
mod database;
mod services;
mod utils;

// This struct represents state
struct AppState {
    db: Connection,
    config: config::Config,
}

#[actix_web::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .parse_filters("warn,chhoto_url=info")
        .init();

    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();

    // Read config from env vars
    let conf = config::read();

    // Tell the user that the server has started, and where it is listening to, rather than simply outputting nothing
    info!("Server has started at 0.0.0.0 on port {}.", conf.port);

    // Do periodic cleanup
    let db_location_clone = conf.db_location.clone();
    info!("Starting cleanup service, will run once every hour.");
    spawn(async move {
        let db = database::open_db(db_location_clone);
        let mut interval = time::interval(time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            database::cleanup(&db);
        }
    });

    let conf_clone = conf.clone();
    // Actually start the server
    HttpServer::new(move || {
        let mut app = App::new()
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
                db: database::open_db(conf_clone.db_location.clone()),
                config: conf_clone.clone(),
            }))
            .wrap(if let Some(header) = &conf.cache_control_header {
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
            .service(services::expand);

        if !conf.disable_frontend {
            app = app.service(Files::new("/", "./resources/").index_file("index.html"));
        }

        app.default_service(actix_web::web::get().to(services::error404))
    })
    // Hardcode the port the server listens to. Allows for more intuitive Docker Compose port management
    .bind(("0.0.0.0", conf.port))?
    .run()
    .await
}
