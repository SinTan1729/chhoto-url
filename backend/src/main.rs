// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::Files;
use actix_web::{
    App, HttpServer,
    cookie::Key,
    http::header::CACHE_CONTROL,
    middleware,
    web::{self, Redirect},
};
use log::{error, info, warn};
use r2d2::Pool;
use rusqlite::Connection;
use std::{
    collections::HashMap,
    io::Result,
    process::exit,
    sync::{Arc, Mutex, Once},
    thread::available_parallelism,
    time::Duration,
};
use tokio::sync::{Semaphore, mpsc};

use crate::database::pool::{ReaderDbManager, open_pool};

// Import modules
mod auth;
mod background;
mod config;
mod database;
mod logging;
mod oidc;
mod services;
mod utils;

// Tests
#[cfg(test)]
mod tests;

pub(crate) const RO_DB_OPEN_ERROR: &str = "Unable to open reader database connection!";
pub(crate) const RW_DB_OPEN_ERROR: &str = "Unable to open writer database connection!";
pub(crate) const DB_POOL_ERROR: &str = "Unable to get a reader database form the pool!";
pub(crate) const DB_MUTEX_ERROR: &str = "Mutex for Database writer is poisoned!";
pub(crate) const OIDC_MUTEX_ERROR: &str = "Mutex for OIDC Data is poisoned!";

struct Semaphores {
    auth: Arc<Semaphore>,
    db: Arc<Semaphore>,
}

// This struct represents state
struct AppState {
    hits_tx: mpsc::Sender<(String, bool)>,
    readers: Pool<ReaderDbManager>,
    writer: Mutex<Connection>,
    config: config::Config,
    frontend_dir: String,
    permits: Semaphores,
    oidc_data: Option<Mutex<oidc::OidcData>>,
    secret_key: Key,
}

static LOGGER: Once = Once::new();

#[actix_web::main]
async fn main() -> Result<()> {
    eprintln!("----------------------------------------------------------------------");
    eprintln!(
        "Starting Chhoto URL Server v{}",
        services::utils::get_version()
    );
    eprintln!("Source: https://github.com/SinTan1729/chhoto-url");
    eprintln!("----------------------------------------------------------------------");

    logging::init_async(Duration::from_secs(1), 8 * 1024, 512);

    // Read config from env vars
    let init_config = config::init_config();
    let config = config::read();
    // ArcMutex is necessary since the writer is shared across threads
    let writer = Mutex::new(database::open_db(&config.db_location, false).expect(RW_DB_OPEN_ERROR));

    // Initialize the database and perform migrations
    database::init_db(
        &mut writer.lock().expect(DB_MUTEX_ERROR),
        init_config.use_wal_mode,
        init_config.ensure_acid,
        init_config.disable_backups,
    );
    // Apply custom title if needed
    let frontend_dir = match utils::apply_custom_title(&init_config.custom_site_title) {
        Ok(dir) => dir,
        Err(e) => {
            warn!("Issue while applying custom title: {e}");
            utils::get_frontend_location().0
        }
    };

    let oidc_client = oidc::create_oidc_client(&config).await;
    let oidc_data = if let Some((http_client, oidc_client)) = oidc_client {
        info!("OIDC was successfully enabled.");
        Some(Mutex::new(oidc::OidcData {
            http_client,
            oidc_client,
            states: HashMap::new(),
        }))
    } else {
        if config.password.is_none() {
            warn!("No auth is set up. Chhoto URL will be exposed to the public.");
            if init_config.exit_on_no_auth {
                error!("Exiting Chhoto URL.");
                exit(1);
            }
        }
        None
    };

    // Generate or load session master key
    let secret_key = utils::generate_key();
    let (hits_tx, hits_rx) = mpsc::channel::<(String, bool)>(1024);

    let data = web::Data::new(AppState {
        hits_tx: hits_tx.clone(),
        readers: open_pool(&config.db_location).expect("Unable to open database reader pool."),
        writer,
        config,
        frontend_dir,
        permits: {
            let n = available_parallelism().map(usize::from).unwrap_or(2);
            Semaphores {
                auth: Arc::new(Semaphore::new(n)),
                db: Arc::new(Semaphore::new(4 * n)),
            }
        },
        oidc_data,
        secret_key,
    });
    // Spawn cleaner
    background::spawn_cleaner(
        data.clone(),
        init_config.use_wal_mode,
        init_config.disable_backups,
    );
    // Spawn hit updater
    background::spawn_hits_worker(data.clone(), hits_rx);

    // Actually start the server
    let data_clone = data.clone();
    let default_app = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::MergeOnly,
            ))
            // Maintain a single instance of database throughout
            .app_data(data.clone())
            .wrap(if let Some(header) = &init_config.cache_control_header {
                middleware::DefaultHeaders::new().add((CACHE_CONTROL, header.to_owned()))
            } else {
                middleware::DefaultHeaders::new()
            })
            .service(web::scope("/api").configure(services::api_routes_v1))
            .service(web::scope("/oidc").configure(services::oidc_routes))
            .service(services::link_handler());

        if !data.config.disable_frontend {
            if let Some(dir) = data.config.custom_landing_directory.clone() {
                app = app
                    .service(Redirect::new("/admin/manage", "/admin/manage/"))
                    .service(
                        Files::new("/admin/manage/", data.frontend_dir.clone())
                            .index_file("index.html"),
                    )
                    .service(Files::new("/", dir).index_file("index.html"));
            } else {
                app = app
                    .service(Files::new("/", data.frontend_dir.clone()).index_file("index.html"));
            }
        }

        app.default_service(services::error_route())
    })
    // Hardcode the port the server listens to. Allows for more intuitive Docker Compose port management
    .bind((&*init_config.listen_address, init_config.port))
    .inspect(|_| {
        LOGGER.call_once(|| {
            info!(
                "Server has started listening to {} on port {}.",
                init_config.listen_address, init_config.port
            );
        })
    })?
    .run();

    if let Some(port) = init_config.health_port {
        let health_app = HttpServer::new(move || {
            App::new()
                .app_data(data_clone.clone())
                .route("/healthz", services::health_route())
        })
        .workers(1)
        .bind(("127.0.0.1", port))?
        .run();
        info!("Health server listening to port {}.", port);
        tokio::try_join!(default_app, health_app)?;
    } else {
        default_app.await?;
    }

    Ok(())
}
