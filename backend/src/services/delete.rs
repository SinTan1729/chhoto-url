// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, CookieJar},
    mime::TEXT_PLAIN,
    web::{self, ServiceConfig, delete},
};
use log::{debug, error, info};
use tokio::task::spawn_blocking;

use crate::{
    AppState,
    auth::{Auth, COOKIE_NAME},
    background::acquire_permit,
    services::{
        types::{
            ChhotoError::{self, ClientError, ServerError},
            JSONResponse,
        },
        utils,
    },
};

pub(super) fn delete_routes_v1(cfg: &mut ServiceConfig) {
    cfg.route("/logout", delete().to(logout))
        .route("/del/{shortlink}", delete().to(delete_link));
}

// Handle logout
// There's no reason to be calling this route with an API key
async fn logout(auth: Auth, req: HttpRequest) -> HttpResponse {
    if matches!(auth, Auth::ValidSession) {
        let mut jar = CookieJar::new();
        let cookie = req
            .cookie(COOKIE_NAME)
            .expect("Valid session must have an auth cookie at this point.");
        jar.add_original(cookie.into_owned());
        jar.remove(Cookie::build(COOKIE_NAME, "").path("/").finish());

        info!("Successful logout.");
        HttpResponse::Ok()
            .cookie(
                jar.delta()
                    .next()
                    .expect("Cookie jar delta should always succeed.")
                    .to_owned(),
            )
            .content_type(TEXT_PLAIN)
            .body("Logged out!")
    } else {
        HttpResponse::Unauthorized()
            .content_type(TEXT_PLAIN)
            .body("You don't seem to be logged in.")
    }
}

// Delete a given shortlink
async fn delete_link(
    shortlink: web::Path<String>,
    auth: Auth,
    data: web::Data<AppState>,
) -> HttpResponse {
    let deleted_text = |link: String| {
        let mut out = String::with_capacity(link.len() + 8);
        out.push_str("Deleted ");
        out.push_str(&link);
        debug!("{}", out);
        out
    };
    match auth {
        Auth::ValidAPIKey => {
            let shortlink_clone = shortlink.clone();
            let server_error = HttpResponse::InternalServerError().json(JSONResponse {
                success: false,
                error: true,
                reason: "Something went wrong when deleting the link.".to_owned(),
            });
            let Ok(permit) = acquire_permit(data.permits.db.clone()).await else {
                return server_error;
            };
            match spawn_blocking(move || {
                let _permit = permit;
                let Ok(writer) = data.writer.lock() else {
                    error!("{}", crate::DB_MUTEX_ERROR);
                    return Err(ChhotoError::ServerError);
                };
                utils::delete_link_helper(
                    &shortlink_clone,
                    &writer,
                    data.config.allow_capital_letters,
                )
            })
            .await
            .map_err(|_| ChhotoError::ServerError)
            .flatten()
            {
                Ok(()) => {
                    let response = JSONResponse {
                        success: true,
                        error: false,
                        reason: deleted_text(shortlink.to_owned()),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(ServerError) => server_error,
                Err(ClientError { reason }) => {
                    let response = JSONResponse {
                        success: false,
                        error: true,
                        reason,
                    };
                    HttpResponse::NotFound().json(response)
                }
            }
        }
        Auth::InvalidAPIKey { result } => HttpResponse::Unauthorized().json(result),
        // If using password - keeps backwards compatibility
        Auth::ValidSession | Auth::NoAuthNeeded => {
            let shortlink_clone = shortlink.clone();
            let server_error = HttpResponse::NotFound()
                .content_type(TEXT_PLAIN)
                .body("Not found!");
            let Ok(permit) = acquire_permit(data.permits.auth.clone()).await else {
                return server_error;
            };
            if spawn_blocking(move || {
                let _permit = permit;
                let Ok(writer) = data.writer.lock() else {
                    error!("{}", crate::DB_MUTEX_ERROR);
                    return Err(ChhotoError::ServerError);
                };
                utils::delete_link_helper(
                    &shortlink_clone,
                    &writer,
                    data.config.allow_capital_letters,
                )
            })
            .await
            .is_ok_and(|r| r.is_ok())
            {
                HttpResponse::Ok()
                    .content_type(TEXT_PLAIN)
                    .body(deleted_text(shortlink.to_string()))
            } else {
                server_error
            }
        }
        Auth::None { result: _ } => HttpResponse::Unauthorized().body("Not logged in!"),
    }
}
