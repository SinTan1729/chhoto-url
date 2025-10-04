// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{
    delete, get,
    http::StatusCode,
    post, put,
    web::{self, Redirect},
    Either, HttpRequest, HttpResponse, Responder,
};
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::env;

use crate::AppState;
use crate::{auth, database};
use crate::{auth::validate, utils};

// Store the version number
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Define JSON struct for returning success/error data
#[derive(Serialize)]
struct Response {
    success: bool,
    error: bool,
    reason: String,
}

// Define JSON struct for returning backend config
#[derive(Serialize)]
struct BackendConfig {
    version: String,
    site_url: Option<String>,
    allow_capital_letters: bool,
    public_mode: bool,
    public_mode_expiry_delay: i64,
    slug_style: String,
    slug_length: usize,
    try_longer_slug: bool,
}

// Needed to return the short URL to make it easier for programs leveraging the API
#[derive(Serialize)]
struct CreatedURL {
    success: bool,
    error: bool,
    shorturl: String,
    expiry_time: i64,
}

// Struct for returning information about a shortlink in expand
#[derive(Serialize)]
struct LinkInfo {
    success: bool,
    error: bool,
    longurl: String,
    hits: i64,
    expiry_time: i64,
}

// Struct for query params in /api/all
#[derive(Deserialize)]
pub struct GetReqParams {
    pub page_no: Option<i64>,
    pub page_size: Option<i64>,
}

// Define the routes

// Add new links
#[post("/api/new")]
pub async fn add_link(
    req: String,
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = utils::is_api_ok(http, config);
    // If success, add new link
    if result.success {
        let (success, reply, expiry_time) = utils::add_link(req, &data.db, config, false);
        if success {
            let site_url = config.site_url.clone();
            let shorturl = if let Some(url) = site_url {
                format!("{url}/{reply}")
            } else {
                let protocol = if config.port == 443 { "https" } else { "http" };
                let port_text = if [80, 443].contains(&config.port) {
                    String::new()
                } else {
                    format!(":{}", config.port)
                };
                format!("{protocol}://localhost{port_text}/{reply}")
            };
            let response = CreatedURL {
                success: true,
                error: false,
                shorturl,
                expiry_time,
            };
            HttpResponse::Created().json(response)
        } else {
            let response = Response {
                success: false,
                error: true,
                reason: reply,
            };
            HttpResponse::Conflict().json(response)
        }
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If password authentication or public mode is used - keeps backwards compatibility
    } else {
        let (success, reply, _) = if auth::validate(session, config) {
            utils::add_link(req, &data.db, config, false)
        } else if config.public_mode {
            utils::add_link(req, &data.db, config, true)
        } else {
            return HttpResponse::Unauthorized().body("Not logged in!");
        };
        if success {
            HttpResponse::Created().body(reply)
        } else {
            HttpResponse::Conflict().body(reply)
        }
    }
}

// Return all active links
#[get("/api/all")]
pub async fn getall(
    data: web::Data<AppState>,
    session: Session,
    params: web::Query<GetReqParams>,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = utils::is_api_ok(http, config);
    // If success, return all links
    if result.success {
        HttpResponse::Ok().body(utils::getall(&data.db, params.into_inner()))
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If password authentication is used - keeps backwards compatibility
    } else if auth::validate(session, config) {
        HttpResponse::Ok().body(utils::getall(&data.db, params.into_inner()))
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Get information about a single shortlink
#[post("/api/expand")]
pub async fn expand(req: String, data: web::Data<AppState>, http: HttpRequest) -> HttpResponse {
    let result = utils::is_api_ok(http, &data.config);
    if result.success {
        let (longurl, hits, expiry_time) =
            utils::get_longurl(req, &data.db, true, data.config.allow_capital_letters);
        if let Some(longlink) = longurl {
            let body = LinkInfo {
                success: true,
                error: false,
                longurl: longlink,
                hits: hits.expect("Error getting hit count for existing shortlink."),
                expiry_time: expiry_time
                    .expect("Error getting expiry time for existing shortlink."),
            };
            HttpResponse::Ok().json(body)
        } else {
            let body = Response {
                success: false,
                error: true,
                reason: "The shortlink does not exist on the server.".to_string(),
            };
            HttpResponse::BadRequest().json(body)
        }
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Get information about a single shortlink
#[put("/api/edit")]
pub async fn edit_link(
    req: String,
    session: Session,
    data: web::Data<AppState>,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = utils::is_api_ok(http, config);
    if result.success || validate(session, config) {
        if let Some((server_error, error_msg)) = utils::edit_link(req, &data.db, config) {
            let body = Response {
                success: false,
                error: true,
                reason: error_msg,
            };
            if server_error {
                HttpResponse::InternalServerError().json(body)
            } else {
                HttpResponse::BadRequest().json(body)
            }
        } else {
            let body = Response {
                success: true,
                error: false,
                reason: String::from("Edit was successful."),
            };
            HttpResponse::Created().json(body)
        }
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Get the site URL
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/siteurl")]
pub async fn siteurl(data: web::Data<AppState>) -> HttpResponse {
    if let Some(url) = &data.config.site_url {
        HttpResponse::Ok().body(url.clone())
    } else {
        HttpResponse::Ok().body("unset")
    }
}

// Get the version number
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/version")]
pub async fn version() -> HttpResponse {
    HttpResponse::Ok().body(format!("Chhoto URL v{VERSION}"))
}

// Get the user's current role
#[get("/api/whoami")]
pub async fn whoami(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = utils::is_api_ok(http, config);
    let acting_user = if result.success || validate(session, config) {
        "admin"
    } else if config.public_mode {
        "public"
    } else {
        "nobody"
    };
    HttpResponse::Ok().body(acting_user)
}

// Get some useful backend config
#[get("/api/getconfig")]
pub async fn getconfig(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = utils::is_api_ok(http, config);
    if result.success || validate(session, config) || data.config.public_mode {
        let backend_config = BackendConfig {
            version: VERSION.to_string(),
            allow_capital_letters: config.allow_capital_letters,
            public_mode: config.public_mode,
            public_mode_expiry_delay: config.public_mode_expiry_delay,
            site_url: config.site_url.clone(),
            slug_style: config.slug_style.clone(),
            slug_length: config.slug_length,
            try_longer_slug: config.try_longer_slug,
        };
        HttpResponse::Ok().json(backend_config)
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// 404 error page
pub async fn error404() -> impl Responder {
    NamedFile::open_async("./resources/static/404.html")
        .await
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}

// Handle a given shortlink
#[get("/{shortlink}")]
pub async fn link_handler(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let shortlink_str = shortlink.to_string();
    if let Some(longlink) = utils::get_longurl(
        shortlink_str,
        &data.db,
        false,
        data.config.allow_capital_letters,
    )
    .0
    {
        database::add_hit(shortlink.as_str(), &data.db);
        if data.config.use_temp_redirect {
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
pub async fn login(req: String, session: Session, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;
    // Check if password is hashed using Argon2. More algorithms maybe added later.
    let authorized = if let Some(password) = &config.password {
        if config.hash_algorithm.is_some() {
            info!("Using Argon2 hash for password validation.");
            let hash = PasswordHash::new(password).expect("The provided password hash is invalid.");
            Some(
                Argon2::default()
                    .verify_password(req.as_bytes(), &hash)
                    .is_ok(),
            )
        } else {
            // If hashing is not enabled, use the plaintext password for matching
            Some(password == &req)
        }
    } else {
        None
    };
    if config.api_key.is_some() {
        if let Some(valid_pass) = authorized {
            if !valid_pass {
                warn!("Failed login attempt!");
                let response = Response {
                    success: false,
                    error: true,
                    reason: "Wrong password!".to_string(),
                };
                return HttpResponse::Unauthorized().json(response);
            }
        }
        // Return Ok if no password was set on the server side
        session
            .insert("chhoto-url-auth", auth::gen_token())
            .expect("Error inserting auth token.");

        let response = Response {
            success: true,
            error: false,
            reason: "Correct password!".to_string(),
        };
        info!("Successful login.");
        HttpResponse::Ok().json(response)
    } else {
        // Keep this function backwards compatible
        if let Some(valid_pass) = authorized {
            if !valid_pass {
                warn!("Failed login attempt!");
                return HttpResponse::Unauthorized().body("Wrong password!");
            }
        }
        // Return Ok if no password was set on the server side
        session
            .insert("chhoto-url-auth", auth::gen_token())
            .expect("Error inserting auth token.");

        info!("Successful login.");
        HttpResponse::Ok().body("Correct password!")
    }
}

// Handle logout
// There's no reason to be calling this route with an API key
#[delete("/api/logout")]
pub async fn logout(session: Session) -> HttpResponse {
    if session.remove("chhoto-url-auth").is_some() {
        info!("Successful logout.");
        HttpResponse::Ok().body("Logged out!")
    } else {
        HttpResponse::Unauthorized().body("You don't seem to be logged in.")
    }
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
pub async fn delete_link(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = utils::is_api_ok(http, config);
    // If success, delete shortlink
    if result.success {
        if utils::delete_link(
            shortlink.to_string(),
            &data.db,
            data.config.allow_capital_letters,
        ) {
            let response = Response {
                success: true,
                error: false,
                reason: format!("Deleted {shortlink}"),
            };
            HttpResponse::Ok().json(response)
        } else {
            let response = Response {
                success: false,
                error: true,
                reason: "The short link was not found, and could not be deleted.".to_string(),
            };
            HttpResponse::NotFound().json(response)
        }
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If "pass" is true - keeps backwards compatibility
    } else if auth::validate(session, config) {
        if utils::delete_link(
            shortlink.to_string(),
            &data.db,
            data.config.allow_capital_letters,
        ) {
            HttpResponse::Ok().body(format!("Deleted {shortlink}"))
        } else {
            HttpResponse::NotFound().body("Not found!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}
