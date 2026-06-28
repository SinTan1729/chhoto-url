// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{HttpResponse, put, web};

use crate::{
    AppState,
    auth::Auth,
    services::types::{
        ChhotoError::{ClientError, ServerError},
        JSONResponse,
    },
    utils,
};

// Edit a shortlink
#[put("/api/edit")]
pub(crate) async fn edit_link(req: String, auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;
    match auth {
        Auth::ValidAPIKey | Auth::ValidSession => {
            match utils::edit_link_helper(&req, &data.db.borrow(), config) {
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
        }
        Auth::None { result } | Auth::InvalidAPIKey { result } => {
            HttpResponse::Unauthorized().json(result)
        }
    }
}
