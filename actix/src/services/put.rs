// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, put, web};

use crate::{
    AppState,
    auth::{self, is_session_valid},
    services::types::{
        ChhotoError::{ClientError, ServerError},
        JSONResponse,
    },
    utils,
};

// Get information about a single shortlink
#[put("/api/edit")]
pub(crate) async fn edit_link(
    req: String,
    session: Session,
    data: web::Data<AppState>,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    if result.success || is_session_valid(session, config) {
        match utils::edit_link_helper(&req, &data.db, config) {
            Ok(()) => {
                let body = JSONResponse {
                    success: true,
                    error: false,
                    reason: String::from("Edit was successful."),
                };
                HttpResponse::Created().json(body)
            }
            Err(ServerError) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when editing the link.".to_string(),
                };
                HttpResponse::InternalServerError().json(body)
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
