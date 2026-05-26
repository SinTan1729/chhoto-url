// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, post,
    web::{self},
};
use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
use log::{debug, info, warn};

use crate::{
    AppState, auth,
    config::HashAlgorithm,
    database,
    services::types::{
        ChhotoError::{ClientError, ServerError},
        CreatedURL, JSONResponse, LinkInfo,
    },
    utils,
};

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
    let result = auth::is_api_ok(http, config);
    // If success, add new link
    if result.success {
        match utils::add_link_helper(&req, &data.db, config, false) {
            Ok((shorturl, expiry_time)) => {
                let site_url = config.site_url.clone();
                let shorturl = if let Some(url) = site_url {
                    format!("{url}/{shorturl}")
                } else {
                    let protocol = if config.port == 443 { "https" } else { "http" };
                    let port_text = if [80, 443].contains(&config.port) {
                        String::new()
                    } else {
                        format!(":{}", config.port)
                    };
                    format!("{protocol}://localhost{port_text}/{shorturl}")
                };
                let response = CreatedURL {
                    success: true,
                    error: false,
                    shorturl,
                    expiry_time,
                };
                HttpResponse::Created().json(response)
            }
            Err(ServerError) => {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when adding the link.".to_string(),
                };
                HttpResponse::InternalServerError().json(response)
            }
            Err(ClientError { reason }) => {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::Conflict().json(response)
            }
        }
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If password authentication or public mode is used - keeps backwards compatibility
    } else {
        let result = if auth::is_session_valid(session, config) {
            utils::add_link_helper(&req, &data.db, config, false)
        } else if config.public_mode {
            utils::add_link_helper(&req, &data.db, config, true)
        } else {
            return HttpResponse::Unauthorized().body("Not logged in!");
        };
        match result {
            Ok((shorturl, _)) => HttpResponse::Created().body(shorturl),
            Err(ServerError) => HttpResponse::InternalServerError()
                .body("Something went wrong when adding the link.".to_string()),
            Err(ClientError { reason }) => HttpResponse::Conflict().body(reason),
        }
    }
}

// Get information about a single shortlink
#[post("/api/expand")]
pub async fn expand(req: String, data: web::Data<AppState>, http: HttpRequest) -> HttpResponse {
    let result = auth::is_api_ok(http, &data.config);
    if result.success {
        match database::find_url(&req, &data.db) {
            Ok(chunks) => {
                let body = LinkInfo {
                    success: true,
                    error: false,
                    longurl: chunks.longlink,
                    hits: chunks.hits,
                    expiry_time: chunks.expiry_time,
                    notes: chunks.notes,
                };
                HttpResponse::Ok().json(body)
            }
            Err(ServerError) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when finding the link.".to_string(),
                };
                HttpResponse::BadRequest().json(body)
            }
            Err(ClientError { reason }) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::BadRequest().json(body)
            }
        }
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Handle login
#[post("/api/login")]
pub async fn login(req: String, session: Session, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;
    // Check if password is hashed using Argon2. More algorithms maybe added later.
    let authorized = if let Some(password) = &config.password {
        match config.hash_algorithm {
            HashAlgorithm::Argon2 => {
                debug!("Using Argon2 hash for password validation.");
                let hash =
                    PasswordHash::new(password).expect("The provided password hash is invalid.");
                Some(
                    Argon2::default()
                        .verify_password(req.as_bytes(), &hash)
                        .is_ok(),
                )
            }
            HashAlgorithm::None => {
                // If hashing is not enabled, use the plaintext password for matching
                Some(password == &req)
            }
        }
    } else {
        None
    };
    if config.api_key.is_some() {
        if let Some(valid_pass) = authorized
            && !valid_pass
        {
            warn!("Failed login attempt!");
            let response = JSONResponse {
                success: false,
                error: true,
                reason: "Wrong password!".to_string(),
            };
            return HttpResponse::Unauthorized().json(response);
        }
        // Return Ok if no password was set on the server side
        session
            .insert("chhoto-url-auth", auth::gen_token_text())
            .expect("Error inserting auth token.");

        let response = JSONResponse {
            success: true,
            error: false,
            reason: "Correct password!".to_string(),
        };
        info!("Successful login.");
        HttpResponse::Ok().json(response)
    } else {
        // Keep this function backwards compatible
        if let Some(valid_pass) = authorized
            && !valid_pass
        {
            warn!("Failed login attempt!");
            return HttpResponse::Unauthorized().body("Wrong password!");
        }
        // Return Ok if no password was set on the server side
        session
            .insert("chhoto-url-auth", auth::gen_token_text())
            .expect("Error inserting auth token.");

        info!("Successful login.");
        HttpResponse::Ok().body("Correct password!")
    }
}
