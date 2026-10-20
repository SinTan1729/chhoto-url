// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use actix_session::Session;
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, post,
    web::{self},
};
use argon2::{Argon2, PasswordVerifier, password_hash::phc::PasswordHash};
use log::{debug, info, warn};

use crate::{
    AppState,
    auth::{self, Auth},
    config::{Config, HashAlgorithm},
    database,
    services::{
        types::{
            AddLinkResponse,
            ChhotoError::{self, ClientError, ServerError},
            CreatedURL, JSONResponse, LinkInfo, LoginReq,
        },
        utils,
    },
};

const SERVER_ERROR_RES: &str = "Something went wrong when adding the link.";
// Add new links
#[post("/api/new")]
pub(crate) async fn add_links(
    req: HttpRequest,
    payload: String,
    auth: Auth,
    data: web::Data<AppState>,
) -> HttpResponse {
    let config = &data.config;
    let cookie_response = async |public_mode| {
        let result = utils::add_links_helper(
            &payload,
            &mut *data.writer.lock().await,
            config,
            public_mode,
        )
        .and_then(|(v, _)| v.into_iter().next().unwrap_or(Err(ServerError)));
        match result {
            Ok((shorturl, _)) => HttpResponse::Created()
                .content_type("text/plain")
                .body(shorturl),
            Err(ClientError { reason }) => HttpResponse::Conflict()
                .content_type("text/plain")
                .body(reason),
            Err(ServerError) => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(SERVER_ERROR_RES.to_owned()),
        }
    };
    match auth {
        Auth::ValidAPIKey => {
            let to_response = |res: Result<(String, i64), ChhotoError>| match res {
                Ok((shortlink, expiry_time)) => {
                    let origin = if let Some(url) = &config.site_url {
                        url.to_owned()
                    } else {
                        let info = req.connection_info();
                        let scheme = info.scheme();
                        let host = info.host();
                        let mut out = String::with_capacity(scheme.len() + host.len() + 3);
                        out.push_str(scheme);
                        out.push_str("://");
                        out.push_str(host);
                        out
                    };
                    let mut shorturl = String::with_capacity(origin.len() + shortlink.len() + 1);
                    shorturl.push_str(&origin);
                    shorturl.push('/');
                    shorturl.push_str(&shortlink);

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
                        reason: SERVER_ERROR_RES.to_owned(),
                    }),
                ),
            };

            match utils::add_links_helper(&payload, &mut *data.writer.lock().await, config, false) {
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
        Auth::ValidSession | Auth::NoAuthNeeded => cookie_response(false).await,
        Auth::None { result: _ } => {
            if data.config.public_mode {
                cookie_response(true).await
            } else {
                HttpResponse::Unauthorized()
                    .content_type("text/plain")
                    .body("Not logged in!")
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
                    reason: SERVER_ERROR_RES.to_owned(),
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
        Auth::ValidSession | Auth::NoAuthNeeded => {
            HttpResponse::Unauthorized().json(JSONResponse {
                success: false,
                error: true,
                reason: "This route needs API auth.".to_owned(),
            })
        }
        Auth::None { result } | Auth::InvalidAPIKey { result } => {
            HttpResponse::Unauthorized().json(result)
        }
    }
}

// Handle login
#[post("/api/login")]
pub(crate) async fn login(
    auth: Auth,
    req: HttpRequest,
    body: String,
    session: Session,
    data: web::Data<AppState>,
) -> HttpResponse {
    let config = &data.config;
    if matches!(auth, Auth::ValidSession) {
        return HttpResponse::Ok().body("Already authorized.");
    }

    fn is_authorized(provided_pass: &str, config: &Config, auth: Auth) -> (bool, bool) {
        if let Some(password) = &config.password {
            (
                match config.hash_algorithm {
                    HashAlgorithm::Argon2 => {
                        debug!("Using Argon2 hash for password validation.");
                        let hash = PasswordHash::new(password)
                            .expect("The provided password hash is invalid.");
                        Argon2::default()
                            .verify_password(provided_pass.as_bytes(), &hash)
                            .is_ok()
                    }
                    HashAlgorithm::None => {
                        // If hashing is not enabled, use the plaintext password for matching
                        subtle::ConstantTimeEq::ct_eq(password.as_bytes(), provided_pass.as_bytes())
                            .into()
                    }
                },
                true,
            )
        } else {
            (matches!(auth, Auth::NoAuthNeeded), false)
        }
    }
    let failure_text = |pass_needed| {
        if pass_needed {
            warn!("Failed login attempt: Wrong password.");
            "Wrong password!"
        } else {
            warn!("Failed login attempt: Password login disabled.");
            "Password login disabled."
        }
    };

    match req.content_type() {
        "application/json" => {
            let Ok(req): Result<LoginReq, _> = serde_json::from_str(&body) else {
                return HttpResponse::BadRequest().body("Bad payload.");
            };
            let (authorized, password_needed) = is_authorized(&req.password, config, auth);
            if authorized {
                session
                    .insert("chhoto-url-auth", auth::gen_token_text(req.remember, false))
                    .expect("Error inserting auth token.");

                let response = JSONResponse {
                    success: true,
                    error: false,
                    reason: "Correct password!".to_owned(),
                };
                info!("Successful login. Remember: {}", req.remember);
                HttpResponse::Ok().json(response)
            } else {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason: failure_text(password_needed).to_owned(),
                };
                HttpResponse::Unauthorized().json(response)
            }
        }
        // Legacy path, might be removed in a future release
        "text/plain" => {
            let (authorized, password_needed) = is_authorized(&body, config, auth);
            if authorized {
                session
                    .insert("chhoto-url-auth", auth::gen_token_text(false, false))
                    .expect("Error inserting auth token.");

                info!("Successful login. Remember: {}", false);
                HttpResponse::Ok()
                    .content_type("text/plain")
                    .body("Correct password!")
            } else {
                HttpResponse::Unauthorized()
                    .content_type("text/plain")
                    .body(failure_text(password_needed))
            }
        }
        _ => HttpResponse::BadRequest().body("Bad content type."),
    }
}
