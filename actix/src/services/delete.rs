// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, delete, web};
use log::info;

use crate::{
    AppState, auth,
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
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = auth::is_api_ok(http, config);
    // If success, delete shortlink
    if result.success {
        match utils::delete_link_helper(&shortlink, &data.db, data.config.allow_capital_letters) {
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
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If using password - keeps backwards compatibility
    } else if auth::is_session_valid(session, config) {
        if utils::delete_link_helper(&shortlink, &data.db, data.config.allow_capital_letters)
            .is_ok()
        {
            HttpResponse::Ok().body(format!("Deleted {shortlink}"))
        } else {
            HttpResponse::NotFound().body("Not found!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}
