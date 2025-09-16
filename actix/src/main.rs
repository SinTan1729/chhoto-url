// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware,
    web::{self, Redirect},
    App, HttpServer,
};
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

// Tests
#[cfg(test)]
mod tests;

// This struct represents state
struct AppState {
    db: Connection,
    config: config::Config,
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .parse_filters("warn,chhoto_url=info,actix_session::middleware=error")
        .format(|buf, record| {
            use chrono::Local;
            use env_logger::fmt::style::{AnsiColor, Style};
            use std::io::Write;

            let subtle = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
            let level_style = buf.default_level_style(record.level());

            writeln!(
                buf,
                "{subtle}[{subtle:#}{} {level_style}{:<5}{level_style:#}{}{subtle}]{subtle:#} {}",
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
    info!("Starting Chhoto URL Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Source: https://github.com/SinTan1729/chhoto-url");
    eprintln!("----------------------------------------------------------------------");

    // Read config from env vars
    let conf = config::read();

    // Tell the user that the server has started, and where it is listening to, rather than simply outputting nothing
    info!("Server has started at {} on port {}.", conf.address, conf.port);

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
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::MergeOnly,
            ))
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
    .bind((conf.address, conf.port))?
    .run()
    .await
}
