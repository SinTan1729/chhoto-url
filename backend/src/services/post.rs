// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use actix_session::Session;
use actix_web::{
    HttpResponse, post,
    web::{self},
};
use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
use log::{debug, info, warn};

use crate::{
    AppState,
    auth::{self, Auth},
    config::HashAlgorithm,
    database,
    services::types::{
        AddLinkResponse,
        ChhotoError::{ClientError, ServerError},
        CreatedURL, JSONResponse, LinkInfo,
    },
    utils,
};

const SERVER_ERROR_RES: &str = "Something went wrong when adding the link.";
// Add new links
#[post("/api/new")]
pub(crate) async fn add_links(req: String, auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;
    let cookie_response = async |public_mode| {
        let result =
            utils::add_links_helper(&req, &mut *data.writer.lock().await, config, public_mode)
                .and_then(|(v, _)| v.into_iter().next().unwrap_or(Err(ServerError)));
        match result {
            Ok((shorturl, _)) => HttpResponse::Created().body(shorturl),
            Err(ClientError { reason }) => HttpResponse::Conflict().body(reason),
            Err(ServerError) => {
                HttpResponse::InternalServerError().body(SERVER_ERROR_RES.to_string())
            }
        }
    };
    match auth {
        Auth::ValidAPIKey => {
            let to_response = |res| match res {
                Ok((shortlink, expiry_time)) => {
                    let site_url = config.site_url.to_owned();
                    let shorturl = if let Some(url) = &site_url {
                        format!("{url}/{shortlink}")
                    } else {
                        let protocol = if config.port == 443 { "https" } else { "http" };
                        let port_text = if [80, 443].contains(&config.port) {
                            String::new()
                        } else {
                            format!(":{}", config.port)
                        };
                        format!("{protocol}://localhost{port_text}/{shortlink}")
                    };

                    (
                        actix_web::http::StatusCode::OK,
                        AddLinkResponse::Success(CreatedURL {
                            success: true,
                            error: false,
                            shorturl,
                            expiry_time,
                        }),
                    )
                }
                Err(ClientError { reason }) => (
                    actix_web::http::StatusCode::BAD_REQUEST,
                    AddLinkResponse::Error(JSONResponse {
                        success: false,
                        error: true,
                        reason,
                    }),
                ),
                Err(ServerError) => (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    AddLinkResponse::Error(JSONResponse {
                        success: false,
                        error: true,
                        reason: SERVER_ERROR_RES.to_string(),
                    }),
                ),
            };

            match utils::add_links_helper(&req, &mut *data.writer.lock().await, config, false) {
                Ok((reply, single_request)) => {
                    if single_request {
                        let (status, response) = to_response(
                            reply
                                .into_iter()
                                .next()
                                .expect("There should be one response here."),
                        );
                        HttpResponse::build(status).json(response)
                    } else {
                        let response: Rc<_> =
                            reply.into_iter().map(to_response).map(|(_, r)| r).collect();
                        HttpResponse::Ok().json(response)
                    }
                }
                Err(error) => {
                    let (status, response) = to_response(Err(error));
                    HttpResponse::build(status).json(response)
                }
            }
        }
        Auth::InvalidAPIKey { result } => HttpResponse::Unauthorized().json(result),
        // If password authentication or public mode is used - keeps backwards compatibility
        Auth::ValidSession => cookie_response(false).await,
        Auth::None { result: _ } => {
            if data.config.public_mode {
                cookie_response(true).await
            } else {
                HttpResponse::Unauthorized().body("Not logged in!")
            }
        }
    }
}

// Get information about a single shortlink
#[post("/api/expand")]
pub(crate) async fn expand(req: String, auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    match auth {
        Auth::ValidAPIKey => match database::find_url(&req, &data.reader) {
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
                    reason: SERVER_ERROR_RES.to_string(),
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
        },
        Auth::ValidSession => HttpResponse::Unauthorized().json(JSONResponse {
            success: false,
            error: true,
            reason: "This route needs API auth.".to_string(),
        }),
        Auth::None { result } | Auth::InvalidAPIKey { result } => {
            HttpResponse::Unauthorized().json(result)
        }
    }
}

// Handle login
#[post("/api/login")]
pub(crate) async fn login(
    auth: Auth,
    req: String,
    session: Session,
    data: web::Data<AppState>,
) -> HttpResponse {
    let config = &data.config;
    if matches!(auth, Auth::ValidSession) {
        return HttpResponse::Ok().body("Already authorized.");
    }

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
