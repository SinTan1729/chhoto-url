// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, CookieJar},
    delete,
    mime::TEXT_PLAIN,
    web,
};
use log::{debug, info};

use crate::{
    AppState,
    auth::Auth,
    services::{
        types::{
            ChhotoError::{ClientError, ServerError},
            JSONResponse,
        },
        utils,
    },
};

// Handle logout
// There's no reason to be calling this route with an API key
#[delete("/api/logout")]
pub(crate) async fn logout(auth: Auth, req: HttpRequest) -> HttpResponse {
    if matches!(auth, Auth::ValidSession) {
        let mut jar = CookieJar::new();
        let cookie = req
            .cookie("chhoto-url-auth")
            .expect("Valid session must have an auth cookie at this point.");
        jar.add_original(cookie.into_owned());
        jar.remove(Cookie::build("chhoto-url-auth", "").path("/").finish());

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
#[delete("/api/del/{shortlink}")]
pub(crate) async fn delete_link(
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
            match utils::delete_link_helper(
                &shortlink,
                &*data.writer.lock().await,
                data.config.allow_capital_letters,
            ) {
                Ok(()) => {
                    let response = JSONResponse {
                        success: true,
                        error: false,
                        reason: deleted_text(shortlink.to_string()),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(ServerError) => {
                    let response = JSONResponse {
                        success: false,
                        error: true,
                        reason: "Something went wrong when deleting the link.".to_owned(),
                    };
                    HttpResponse::InternalServerError().json(response)
                }
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
            if utils::delete_link_helper(
                &shortlink,
                &*data.writer.lock().await,
                data.config.allow_capital_letters,
            )
            .is_ok()
            {
                HttpResponse::Ok()
                    .content_type(TEXT_PLAIN)
                    .body(deleted_text(shortlink.to_string()))
            } else {
                HttpResponse::NotFound()
                    .content_type(TEXT_PLAIN)
                    .body("Not found!")
            }
        }
        Auth::None { result: _ } => HttpResponse::Unauthorized().body("Not logged in!"),
    }
}
