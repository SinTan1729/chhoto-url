// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{HttpResponse, query, web};
use log::debug;

use crate::{
    AppState,
    auth::Auth,
    database,
    services::types::{
        ChhotoError::{ClientError, ServerError},
        JSONResponse, LinkInfo,
    },
};

// Get information about a single shortlink
#[query("/api/expand")]
pub(crate) async fn expand(req: String, auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    expand_helper(req, auth, data).await
}

pub(super) async fn expand_helper(
    req: String,
    auth: Auth,
    data: web::Data<AppState>,
) -> HttpResponse {
    match auth {
        Auth::ValidAPIKey => match database::find_url(&req, &data.reader) {
            Ok(chunks) => {
                let body = LinkInfo {
                    success: true,
                    error: false,
                    longurl: chunks.longlink,
                    hits: chunks.hits,
                    enabled: chunks.enabled,
                    expiry_time: chunks.expiry_time,
                    notes: chunks.notes,
                };
                debug!("Expanded link {}", req);
                HttpResponse::Ok().json(body)
            }
            Err(ServerError) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when expanding the link".to_owned(),
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
