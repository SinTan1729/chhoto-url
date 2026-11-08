// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    mime::TEXT_PLAIN,
    web::{self, ServiceConfig, post},
};
use log::{error, info, warn};
use std::rc::Rc;
use tokio::task::spawn_blocking;

use crate::{
    AppState,
    auth::{Auth, gen_cookie},
    background::acquire_permit,
    services::{
        types::{
            AddLinkResponse,
            ChhotoError::{self, ClientError, ServerError},
            CreatedURL, JSONResponse, LoginReq,
        },
        utils,
    },
};

pub(super) fn post_routes_v1(cfg: &mut ServiceConfig) {
    cfg.route("/new", post().to(add_links))
        .route("/login", post().to(login));
}

// Add new links
async fn add_links(
    req: HttpRequest,
    payload: String,
    auth: Auth,
    data: web::Data<AppState>,
) -> HttpResponse {
    const SERVER_ERROR_RES: &str = "Something went wrong when adding the link.";
    let config = &data.config;
    let cookie_response = async |public_mode| {
        let Ok(mut writer) = data.writer.lock() else {
            error!("{}", crate::DB_MUTEX_ERROR);
            return HttpResponse::InternalServerError()
                .content_type(TEXT_PLAIN)
                .body("Internal server error.");
        };
        let result = utils::add_links_helper(&payload, &mut writer, config, public_mode, true)
            .and_then(|(v, _)| v.into_iter().next().unwrap_or(Err(ServerError)));
        match result {
            Ok((shorturl, _)) => HttpResponse::Created()
                .content_type(TEXT_PLAIN)
                .body(shorturl),
            Err(ClientError { reason }) => HttpResponse::Conflict()
                .content_type(TEXT_PLAIN)
                .body(reason),
            Err(ServerError) => HttpResponse::InternalServerError()
                .content_type(TEXT_PLAIN)
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
                            enabled: true,
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

            let data_clone = data.clone();
            let Ok(permit) = acquire_permit(data.permits.db.clone()).await else {
                let res = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong while adding links".to_owned(),
                };
                return HttpResponse::InternalServerError().json(res);
            };
            match spawn_blocking(move || {
                let _permit = permit;
                let Ok(mut writer) = data_clone.writer.lock() else {
                    error!("{}", crate::DB_MUTEX_ERROR);
                    return Err(ServerError);
                };
                utils::add_links_helper(&payload, &mut writer, &data_clone.config, false, false)
            })
            .await
            .map_err(|_| ServerError)
            .flatten()
            {
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
                    .content_type(TEXT_PLAIN)
                    .body("Not logged in!")
            }
        }
    }
}

// Handle login
async fn login(
    auth: Auth,
    req: HttpRequest,
    body: String,
    data: web::Data<AppState>,
) -> HttpResponse {
    let config = &data.config;

    if matches!(auth, Auth::ValidSession) {
        return HttpResponse::Ok().body("Already authorized.");
    }

    if config.disable_frontend {
        return HttpResponse::Forbidden().body("Frontend is disabled.");
    }

    // Normalize both supported request formats into the same inputs.
    let (password, remember, is_json) = match req.content_type() {
        "application/json" => {
            let Ok(req) = serde_json::from_str::<LoginReq>(&body) else {
                return HttpResponse::BadRequest().body("Bad payload.");
            };

            (req.password, req.remember, true)
        }
        "text/plain" => (body, false, false),
        _ => return HttpResponse::BadRequest().body("Bad content type."),
    };

    let (authorized, password_needed) = match utils::verify_login(
        password,
        config.password.clone(),
        auth,
        data.permits.auth.clone(),
    )
    .await
    {
        Ok(result) => result,
        Err(err) => {
            error!("Password verification worker failed: {err}");
            return HttpResponse::InternalServerError()
                .body("Authentication temporarily unavailable.");
        }
    };

    if authorized {
        let cookie = gen_cookie(remember, false, &data.secret_key);
        info!("Successful login. Remember: {remember}");

        if is_json {
            HttpResponse::Ok()
                .cookie(cookie.to_owned())
                .json(JSONResponse {
                    success: true,
                    error: false,
                    reason: "Correct password!".to_owned(),
                })
        } else {
            HttpResponse::Ok()
                .cookie(cookie.to_owned())
                .content_type(TEXT_PLAIN)
                .body("Correct password!")
        }
    } else {
        let reason = if password_needed {
            warn!("Failed login attempt: Wrong password.");
            "Wrong password!"
        } else {
            warn!("Failed login attempt: Password login disabled.");
            "Password login disabled."
        };

        if is_json {
            let response = JSONResponse {
                success: false,
                error: true,
                reason: reason.to_owned(),
            };

            if password_needed {
                HttpResponse::Unauthorized().json(response)
            } else {
                HttpResponse::Forbidden().json(response)
            }
        } else {
            let mut res = if password_needed {
                HttpResponse::Unauthorized()
            } else {
                HttpResponse::Forbidden()
            };

            res.content_type(TEXT_PLAIN).body(reason)
        }
    }
}
