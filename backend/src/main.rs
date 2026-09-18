// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::Files;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpServer,
    cookie::{self, Key},
    middleware,
    web::{self, Redirect},
};
use log::{info, warn};
use rusqlite::Connection;
use std::{
    io::Result,
    sync::{Arc, Once},
};
use tokio::sync::{Mutex, mpsc};

// Import modules
mod auth;
mod background;
mod config;
mod database;
mod services;
mod utils;

// Tests
#[cfg(test)]
mod tests;

// This struct represents state
struct AppState {
    hits_tx: mpsc::Sender<(String, bool)>,
    reader: Connection,
    writer: Arc<Mutex<Connection>>,
    config: config::Config,
}

static LOGGER: Once = Once::new();

#[actix_web::main]
async fn main() -> Result<()> {
    utils::init_logger();
    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();

    eprintln!("----------------------------------------------------------------------");
    info!(
        "Starting Chhoto URL Server v{}",
        services::utils::get_version()
    );
    info!("Source: https://github.com/SinTan1729/chhoto-url");
    eprintln!("----------------------------------------------------------------------");

    // Read config from env vars
    let conf = config::read();
    // ArcMutex is necessary since the writer is shared across threads
    let writer = Arc::new(Mutex::new(database::open_db(&conf.db_location, false)));

    // Initialize the database and perform migrations
    database::init_db(
        &mut *writer.lock().await,
        conf.use_wal_mode,
        conf.ensure_acid,
        conf.disable_backups,
    );
    // Spawn cleaner
    background::spawn_cleaner(Arc::clone(&writer), conf.use_wal_mode, conf.disable_backups);
    // Spawn hit updater
    let (hits_tx, hits_rx) = mpsc::channel::<(String, bool)>(1024);
    background::spawn_hits_worker(Arc::clone(&writer), hits_rx);
    // Apply custom title if needed
    let frontend_dir = match utils::apply_custom_title(&conf.custom_site_title) {
        Ok(dir) => dir,
        Err(e) => {
            warn!("Issue while applying custom title: {}", e);
            utils::get_frontend_location().0
        }
    };

    let port = conf.port;
    let addr = conf.listen_address.clone();
    let db_location_for_health = conf.db_location.clone();
    // Actually start the server
    let default_app = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::MergeOnly,
            ))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
                    )
                    .cookie_secure(false)
                    .build(),
            )
            // Maintain a single instance of database throughout
            .app_data(web::Data::new(AppState {
                hits_tx: hits_tx.clone(),
                reader: database::open_db(&conf.db_location, true),
                writer: Arc::clone(&writer),
                config: conf.clone(),
            }))
            .wrap(if let Some(header) = &conf.cache_control_header {
                middleware::DefaultHeaders::new().add(("Cache-Control", header.to_owned()))
            } else {
                middleware::DefaultHeaders::new()
            })
            .service(services::link_handler)
            .service(services::edit_link)
            .service(services::getall)
            .service(services::siteurl)
            .service(services::version)
            .service(services::getconfig)
            .service(services::add_links)
            .service(services::delete_link)
            .service(services::login)
            .service(services::logout)
            .service(services::expand)
            .service(services::whoami);

        if !conf.disable_frontend {
            if let Some(dir) = &conf.custom_landing_directory {
                app = app
                    .service(Redirect::new("/admin/manage", "/admin/manage/"))
                    .service(Files::new("/admin/manage/", &frontend_dir).index_file("index.html"))
                    .service(Files::new("/", dir).index_file("index.html"));
            } else {
                app = app.service(Files::new("/", &frontend_dir).index_file("index.html"));
            }
        }

        app.default_service(actix_web::web::get().to(services::utils::error404))
    })
    // Hardcode the port the server listens to. Allows for more intuitive Docker Compose port management
    .bind((&*addr, port))
    .inspect(|_| {
        LOGGER.call_once(|| {
            info!("Server has started listening to {} on port {}.", addr, port);
        })
    })?
    .run();

    let health_app = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database::open_db(
                &db_location_for_health,
                true,
            )))
            .service(services::health_handler)
    })
    .workers(1)
    .bind(("127.0.0.1", 1729))?
    .run();

    tokio::try_join!(default_app, health_app)?;
    Ok(())
}
