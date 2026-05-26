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
use log::info;
use rusqlite::Connection;
use std::{io::Result, sync::Once};
use tokio::{spawn, time};

// Import modules
mod auth;
mod config;
mod database;
mod services;
mod utils;

// Tests
#[cfg(test)]
mod tests;

// This struct represents state
struct AppState {
    db: Connection,
    config: config::Config,
}

static LOGGER: Once = Once::new();

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .parse_filters(
            std::env::var("RUST_LOG")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or("warn,chhoto_url=info,actix_session::middleware=error".to_string())
                .as_str(),
        )
        .format(|buf, record| {
            use chrono::Local;
            use env_logger::fmt::style::{AnsiColor, Style};
            use std::io::Write;

            let subtle = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
            let level_style = buf.default_level_style(record.level());

            writeln!(
                buf,
                "{subtle}[{subtle:#}{} {level_style}{:<6}{level_style:#}{}{subtle}]{subtle:#} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%Z"),
                record.level(),
                record.module_path().unwrap_or_default(),
                record.args()
            )
        })
        .init();

    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();

    eprintln!("----------------------------------------------------------------------");
    info!("Starting Chhoto URL Server v{}", utils::get_version());
    info!("Source: https://github.com/SinTan1729/chhoto-url");
    eprintln!("----------------------------------------------------------------------");

    // Read config from env vars
    let conf = config::read();

    // Do periodic cleanup
    let db_location = conf.db_location.clone();
    database::initialize_db(&db_location, conf.use_wal_mode, conf.ensure_acid);

    spawn(async move {
        info!("Starting database cleanup service, will run once every hour.");
        let db = database::open_db(&db_location);
        let mut interval = time::interval(time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            database::cleanup(&db, conf.use_wal_mode);
        }
    });

    let conf_clone = conf.clone();
    // Actually start the server
    HttpServer::new(move || {
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
                db: database::open_db(&conf.db_location),
                config: conf_clone.clone(),
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
            .service(services::add_link)
            .service(services::delete_link)
            .service(services::login)
            .service(services::logout)
            .service(services::expand)
            .service(services::whoami);

        if !conf.disable_frontend {
            if let Some(dir) = &conf.custom_landing_directory {
                app = app
                    .service(Redirect::new("/admin/manage", "/admin/manage/"))
                    .service(Files::new("/admin/manage/", "./resources/").index_file("index.html"))
                    .service(Files::new("/", dir).index_file("index.html"));
            } else {
                app = app.service(Files::new("/", "./resources/").index_file("index.html"));
            }
        }

        app.default_service(actix_web::web::get().to(services::error404))
    })
    // Hardcode the port the server listens to. Allows for more intuitive Docker Compose port management
    .bind((conf.listen_address.clone(), conf.port))
    .inspect(|_| {
        LOGGER.call_once(|| {
            info!(
                "Server has started listening to {} on port {}.",
                &conf.listen_address, conf.port
            );
        })
    })?
    .run()
    .await
}
