// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{
    delete, get,
    http::StatusCode,
    post,
    web::{self, Redirect},
    Either, HttpRequest, HttpResponse, Responder,
};
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use log::{info, warn};
use serde::Serialize;
use std::env;

use crate::utils;
use crate::AppState;
use crate::{auth, database};

// Store the version number
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Define JSON struct for returning JSON data
#[derive(Serialize)]
struct Response {
    success: bool,
    error: bool,
    reason: String,
}

// Needed to return the short URL to make it easier for programs leveraging the API
#[derive(Serialize)]
struct CreatedURL {
    success: bool,
    error: bool,
    shorturl: String,
    expiry_time: i64,
}

// Struct for returning information about a shortlink
#[derive(Serialize)]
struct LinkInfo {
    success: bool,
    error: bool,
    longurl: String,
    hits: i64,
    expiry_time: i64,
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
        let (success, reply, expiry_time) = utils::add_link(req, &data.db, config);
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
    } else if config.public_mode || auth::validate(session, config) {
        let (success, reply, _) = utils::add_link(req, &data.db, config);
        if success {
            HttpResponse::Created().body(reply)
        } else {
            HttpResponse::Conflict().body(reply)
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Return all active links
#[get("/api/all")]
pub async fn getall(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = utils::is_api_ok(http, config);
    // If success, return all links
    if result.success {
        HttpResponse::Ok().body(utils::getall(&data.db))
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If password authentication is used - keeps backwards compatibility
    } else if auth::validate(session, config) {
        HttpResponse::Ok().body(utils::getall(&data.db))
    } else {
        let body = if config.public_mode {
            format!("Using public mode. {}", config.public_mode_expiry_delay)
        } else {
            String::from("Not logged in!")
        };
        HttpResponse::Unauthorized().body(body)
    }
}

// Get information about a single shortlink
#[post("/api/expand")]
pub async fn expand(req: String, data: web::Data<AppState>, http: HttpRequest) -> HttpResponse {
    let result = utils::is_api_ok(http, &data.config);
    if result.success {
        let (longurl, hits, expiry_time) = utils::get_longurl(req, &data.db, true);
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

// Get the site URL
#[get("/api/siteurl")]
pub async fn siteurl(data: web::Data<AppState>) -> HttpResponse {
    if let Some(url) = &data.config.site_url {
        HttpResponse::Ok().body(url.clone())
    } else {
        HttpResponse::Ok().body("unset")
    }
}

// Get the version number
#[get("/api/version")]
pub async fn version() -> HttpResponse {
    HttpResponse::Ok().body(VERSION)
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
    if let Some(longlink) = utils::get_longurl(shortlink_str, &data.db, false).0 {
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
        if utils::delete_link(shortlink.to_string(), &data.db) {
            let response = Response {
                success: true,
                error: false,
                reason: format!("Deleted {}", shortlink),
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
        if utils::delete_link(shortlink.to_string(), &data.db) {
            HttpResponse::Ok().body(format!("Deleted {shortlink}"))
        } else {
            HttpResponse::NotFound().body("Not found!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}
