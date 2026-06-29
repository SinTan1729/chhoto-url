// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::ops::Deref;

use actix_session::Session;
use actix_web::{HttpResponse, delete, web};
use log::info;

use crate::{
    AppState,
    auth::Auth,
    services::types::{
        ChhotoError::{ClientError, ServerError},
        JSONResponse,
    },
    utils,
};

// Handle logout
// There's no reason to be calling this route with an API key
#[delete("/api/logout")]
pub(crate) async fn logout(session: Session) -> HttpResponse {
    if session.remove("chhoto-url-auth").is_some() {
        info!("Successful logout.");
        HttpResponse::Ok().body("Logged out!")
    } else {
        HttpResponse::Unauthorized().body("You don't seem to be logged in.")
    }
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
pub(crate) async fn delete_link(
    shortlink: web::Path<String>,
    auth: Auth,
    data: web::Data<AppState>,
) -> HttpResponse {
    match auth {
        Auth::ValidAPIKey => {
            match utils::delete_link_helper(
                &shortlink,
                data.writer.lock().await.deref(),
                data.config.allow_capital_letters,
            ) {
                Ok(()) => {
                    let response = JSONResponse {
                        success: true,
                        error: false,
                        reason: format!("Deleted {shortlink}"),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(ServerError) => {
                    let response = JSONResponse {
                        success: false,
                        error: true,
                        reason: "Something went wrong when deleting the link.".to_string(),
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
        Auth::ValidSession => {
            if utils::delete_link_helper(
                &shortlink,
                data.writer.lock().await.deref(),
                data.config.allow_capital_letters,
            )
            .is_ok()
            {
                HttpResponse::Ok().body(format!("Deleted {shortlink}"))
            } else {
                HttpResponse::NotFound().body("Not found!")
            }
        }
        Auth::None { result: _ } => HttpResponse::Unauthorized().body("Not logged in!"),
    }
}
